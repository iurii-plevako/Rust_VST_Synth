use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use crate::envelope::Envelope;
use crate::filter::{Filter, FilterParameters};
use crate::oscillator::OscillatorConfig;
use crate::voice::Voice;

pub struct Synthesizer {
    shared_state: Arc<Mutex<SharedState>>,
    config: SynthesizerConfig,
    stream: Option<cpal::Stream>,
}

struct SharedState {
    voices: Vec<Voice>,
    sample_rate: f64,
}


// Add Send marker for the Synthesizer
unsafe impl Send for Synthesizer {}

impl Synthesizer {
    pub fn new(config: SynthesizerConfig) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            voices: Vec::new(),
            sample_rate: 44100.0,
        }));

        Self {
            shared_state,
            config,
            stream: None,
        }
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
            let mut state = self.shared_state.lock().unwrap();
            state.sample_rate = config.sample_rate().0 as f64;
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
        for voice in &mut state.voices {
            // This ensures each voice has the correct sample rate
            voice.update_sample_rate(state.sample_rate);
        }

        for sample in buffer.iter_mut() {
            let mixed_sample = state.voices.iter_mut()
                .map(|voice| voice.next_sample())
                .sum::<f64>();

            let num_voices = state.voices.len().max(1) as f64;
            *sample = (mixed_sample / num_voices) as f32;
        }

        state.voices.retain(|voice| voice.is_active());
    }

    pub fn note_on(&mut self, frequency: f64) {
        let mut state = self.shared_state.lock().unwrap();
        if let Some(voice) = state.voices.iter_mut().find(|v| !v.is_active()) {
            voice.trigger_note(frequency);
        } else if state.voices.len() < self.config.max_voices {
            let mut voice = Voice::new(
                state.sample_rate,
                self.config.oscillator_configs.clone(),
                self.config.envelope.clone(),
                frequency,
                self.config.filter.clone(),
                self.config.filter_envelope.clone(),
            );
            voice.trigger_note(frequency);
            state.voices.push(voice);
        }
    }

    pub fn note_off(&mut self, frequency: f64) {
        let mut state = self.shared_state.lock().unwrap();
        if let Some(voice) = state.voices.iter_mut().find(|v| v.is_active()) {
            voice.release_note();
        }
    }
}

#[derive(Clone)]
pub struct SynthesizerConfig {
    pub oscillator_configs: Vec<OscillatorConfig>,
    pub envelope: Arc<Mutex<Envelope>>,
    pub filter: Filter,
    pub filter_envelope: Arc<Mutex<Envelope>>,
    pub max_voices: usize,
}