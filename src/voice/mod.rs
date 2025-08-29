// voice.rs
use std::sync::{Arc, Mutex};

use crate::oscillator::{Oscillator,OscillatorConfig};
use crate::envelope::Envelope;

#[derive(Clone)]
pub struct Voice {
    oscillators: Vec<Oscillator>,
    envelope: Envelope,
    frequency: f64,
    is_note_on: bool,
    sample_rate: f64,
}

impl Voice {

    pub fn new(
        sample_rate: f64,
        oscillator_configs: Vec<OscillatorConfig>,
        envelope: Arc<Mutex<Envelope>>,
        base_frequency: f64
    ) -> Self {
        let oscillators = oscillator_configs.into_iter()
            .map(|config| Oscillator::new(sample_rate, base_frequency, config))
            .collect();

        // Create a new envelope by cloning the values from the locked envelope
        let envelope_guard = envelope.lock().unwrap();
        let envelope = envelope_guard.clone();
        drop(envelope_guard); // Explicitly drop the guard

        Voice {
            oscillators,
            envelope,
            frequency: base_frequency,
            is_note_on: false,
            sample_rate,
        }
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f64) {
        self.sample_rate = new_sample_rate;
        self.envelope.update_sample_rate(new_sample_rate);
        for osc in &mut self.oscillators {
            osc.update_sample_rate(new_sample_rate);
        }
    }

    pub fn is_active(&self) -> bool {
        // Keep the voice active until the envelope completely finishes its release
        self.envelope.is_active()  // Remove the || self.is_note_on part since the envelope handles the state
    }

    pub fn trigger_note(&mut self, frequency: f64) {
        self.frequency = frequency;  // Store the frequency
        self.is_note_on = true;
        self.envelope.trigger();
    }

    pub fn release_note(&mut self) {
        self.is_note_on = false;
        self.envelope.release();
    }

    pub fn next_sample(&mut self) -> f64 {
        let envelope_value = self.envelope.next_value();
        
        let sample = self.oscillators.iter_mut()
            .map(|osc| osc.next_sample())
            .sum::<f64>();
            
        sample * envelope_value
    }
}