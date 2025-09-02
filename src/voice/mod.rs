use std::sync::{Arc, Mutex};
use crate::oscillator::{OscillatorConfig, WaveformGenerator, BasicOscillator, RandomOscillator};
use crate::envelope::Envelope;
use crate::filter::Filter;
use crate::voice_configuration::Waveform;

pub struct Voice {
    oscillators: Vec<Box<dyn WaveformGenerator>>,
    envelope: Arc<Mutex<Envelope>>,
    filter: Filter,
    filter_envelope: Arc<Mutex<Envelope>>,
    frequency: f32,
    is_note_on: bool,
    sample_rate: f32,
}

// Manual Clone implementation for Voice
impl Clone for Voice {
    fn clone(&self) -> Self {
        Voice {
            oscillators: self.oscillators.iter()
                .map(|osc| osc.box_clone())
                .collect(),
            envelope: self.envelope.clone(),
            filter: self.filter.clone(),
            filter_envelope: self.filter_envelope.clone(),
            frequency: self.frequency,
            is_note_on: self.is_note_on,
            sample_rate: self.sample_rate,
        }
    }
}

impl Voice {

    pub fn new(
        sample_rate: f32,
        oscillator_configs: Vec<OscillatorConfig>,
        envelope: Arc<Mutex<Envelope>>,
        base_frequency: f32,
        mut filter: Filter,
        filter_envelope: Arc<Mutex<Envelope>>,
    ) -> Self {
        let oscillators = oscillator_configs.into_iter()
            .map(|config| match config.waveform {
                Waveform::RANDOM => Box::new(RandomOscillator::new(sample_rate, base_frequency, config)) as Box<dyn WaveformGenerator>,
                _ => Box::new(BasicOscillator::new(sample_rate, base_frequency, config)) as Box<dyn WaveformGenerator>,
            })
            .collect();

        // Add the filter envelope as a modulation source
        filter.add_modulation_source(filter_envelope.clone());

        Voice {
            oscillators,
            envelope,
            frequency: base_frequency,
            filter,
            filter_envelope,
            is_note_on: false,
            sample_rate,
        }
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f32) {
        self.sample_rate = new_sample_rate;
        self.envelope.lock().unwrap().update_sample_rate(new_sample_rate);
        for osc in &mut self.oscillators {
            osc.update_sample_rate(new_sample_rate);
        }
    }

    pub fn is_active(&self) -> bool {
        if let Ok(env) = self.envelope.lock() {
            env.is_active()
        } else {
            false
        }

    }

    pub fn trigger_note(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.is_note_on = true;
        if let Ok(mut env) = self.envelope.lock() {
            env.trigger();
        }
        if let Ok(mut filter_env) = self.filter_envelope.lock() {
            filter_env.trigger();
        }
    }

    pub fn release_note(&mut self) {
        self.is_note_on = false;
        if let Ok(mut env) = self.envelope.lock() {
            env.release();
        }
        if let Ok(mut filter_env) = self.filter_envelope.lock() {
            filter_env.release();
        }

    }

    pub fn next_sample(&mut self) -> f32 {
        let envelope_value = if let Ok(mut env) = self.envelope.lock() {
            env.next_value()
        } else {
            0.0
        };

        let sample = self.oscillators.iter_mut()
            .map(|osc| osc.next_sample())
            .sum::<f32>();

        let enveloped_sample = sample * envelope_value;

        self.filter.process_sample(enveloped_sample)
    }
}