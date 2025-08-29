// voice.rs
use std::sync::{Arc, Mutex};

use crate::oscillator::{Oscillator,OscillatorConfig};
use crate::envelope::Envelope;
use crate::filter::{Filter, FilterParameters};

#[derive(Clone)]
pub struct Voice {
    oscillators: Vec<Oscillator>,
    envelope: Arc<Mutex<Envelope>>,
    filter: Filter,
    filter_envelope: Arc<Mutex<Envelope>>,
    frequency: f64,
    is_note_on: bool,
    sample_rate: f64,
}

impl Voice {

    pub fn new(
        sample_rate: f64,
        oscillator_configs: Vec<OscillatorConfig>,
        envelope: Arc<Mutex<Envelope>>,
        base_frequency: f64,
        mut filter: Filter,
        filter_envelope: Arc<Mutex<Envelope>>,
    ) -> Self {
        let oscillators = oscillator_configs.into_iter()
            .map(|config| Oscillator::new(sample_rate, base_frequency, config))
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

    pub fn update_sample_rate(&mut self, new_sample_rate: f64) {
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

    pub fn trigger_note(&mut self, frequency: f64) {
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

    pub fn next_sample(&mut self) -> f64 {
        let envelope_value = if let Ok(mut env) = self.envelope.lock() {
            env.next_value()
        } else {
            0.0
        };

        let sample = self.oscillators.iter_mut()
            .map(|osc| osc.next_sample())
            .sum::<f64>();

        let enveloped_sample = sample * envelope_value;

        self.filter.process_sample(enveloped_sample)
    }
}