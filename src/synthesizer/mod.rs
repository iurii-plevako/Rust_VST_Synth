use std::collections::HashMap;
use crate::envelope::{Envelope, EnvelopeConfig};
use crate::filter::Filter;
use crate::oscillator::OscillatorConfig;
use crate::voice::{Voice, VoiceConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct Synthesizer {
    active_notes: HashMap<u32, Vec<usize>>,
    next_voice: usize,
    config: SynthesizerConfig,
    shared_state: Arc<Mutex<SharedState>>,
    stream: Option<cpal::Stream>,
}

struct SharedState {
    voices: Vec<Voice>,
    sample_rate: f32,
    next_voice: usize,
}

impl SharedState {
    fn find_free_voice(&mut self) -> Option<usize> {
        if self.voices.is_empty() { return None; }
        if let Some(i) = self.voices.iter().position(|v| !v.is_active) {
            Some(i)
        } else {
            let i = self.next_voice;
            let len = self.voices.len();
            self.next_voice = (self.next_voice + 1) % len;
            Some(i)
        }
    }
}

// Add Send marker for the Synthesizer
unsafe impl Send for Synthesizer {}

impl Synthesizer {
    pub fn new(config: SynthesizerConfig) -> Self {
        let voice_cfg = VoiceConfig {
            oscillator_configs: config.oscillator_configs.clone(),
            filter: config.filter.clone(),
        };

        let voice_count = config.max_voices.max(1);
        let voices = (0..voice_count)
            .map(|_| Voice::new(&voice_cfg, &config.envelope_config, config.sample_rate))
            .collect::<Vec<_>>();

        let shared_state = Arc::new(Mutex::new(SharedState {
            voices,
            sample_rate: config.sample_rate,
            next_voice: 0,
        }));

        Self {
            active_notes: HashMap::new(),
            next_voice: 0,
            config,
            shared_state,
            stream: None,
        }
    }
    pub fn note_on(&mut self, frequency: f32) {
        let note_id = self.frequency_to_note_id(frequency);

        let mut state = self.shared_state.lock()
            .unwrap_or_else(|e| e.into_inner());

        let existing_env_value = state.voices.iter()
            .find(|v| v.is_active && v.note_id == note_id)
            .map(|v| v.get_envelope_value());

        let other_env_value = if !self.config.envelope_config.retrigger { existing_env_value } else { None };

        let Some(voice_idx) = state.find_free_voice() else {
            eprintln!("No voices configured; ignoring note_on for {}", note_id);
            return;
        };

        state.voices[voice_idx].trigger(frequency, note_id, other_env_value);
        self.active_notes.entry(note_id).or_default().push(voice_idx);
    }

    pub fn note_off(&mut self, frequency: f32) {
        let note_id = self.frequency_to_note_id(frequency);

        let mut state = self.shared_state.lock()
            .unwrap_or_else(|e| e.into_inner());

        if let Some(indices) = self.active_notes.remove(&note_id) {
            for idx in indices {
                if let Some(v) = state.voices.get_mut(idx) {
                    v.release(note_id);
                }
            }
        }
    }

    fn frequency_to_note_id(&self, frequency: f32) -> u32 {
        // Convert frequency to a unique identifier
        // This could be as simple as rounding the frequency to the nearest integer
        frequency.round() as u32
    }


    pub fn start_audio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting audio...");
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("no output device available")?;
        println!("Using audio device: {}", device.name()?);

        let config = device.default_output_config()?;
        println!("Sample rate: {}", config.sample_rate().0);

        {
            let mut state = self.shared_state.lock().unwrap_or_else(|e| e.into_inner());
            state.sample_rate = config.sample_rate().0 as f32;
        }

        let shared_state = self.shared_state.clone();
        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                if let Ok(mut state) = shared_state.lock() {
                    Self::process_audio(&mut state, data);
                }
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None
        )?;

        println!("Playing stream...");
        stream.play()?;
        self.stream = Some(stream);
        println!("Audio started successfully");
        Ok(())
    }

    fn process_audio(state: &mut SharedState, buffer: &mut [f32]) {
        // Keep the pool intact
        for voice in &mut state.voices {
            voice.update_sample_rate(state.sample_rate);
        }

        for sample in buffer.iter_mut() {
            let mut sum = 0.0;
            let mut count = 0;

            for v in &mut state.voices {
                if v.is_active() {
                    sum += v.next_sample();
                    count += 1;
                }
            }

            *sample = if count > 0 { sum / count as f32 } else { 0.0 };
        }
    }
}

#[derive(Clone)]
pub struct SynthesizerConfig {
    pub oscillator_configs: Vec<OscillatorConfig>,
    pub envelope_config: EnvelopeConfig,
    pub filter: Filter,
    pub filter_envelope_config: EnvelopeConfig,
    pub max_voices: usize,
    pub sample_rate: f32,
}
