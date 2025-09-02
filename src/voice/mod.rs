use crate::envelope::{Envelope, EnvelopeConfig};
use crate::filter::Filter;
use crate::oscillator::{make_oscillator, OscillatorConfig, WaveformGenerator};

pub struct VoiceConfig {
    pub oscillator_configs: Vec<OscillatorConfig>,
    pub filter: Filter,
}

pub struct Voice {
    frequency: f32,
    oscillators: Vec<Box<dyn WaveformGenerator>>, // polymorphic oscillators
    envelope: Envelope,
    filter: Filter,
    pub(crate) is_active: bool,
    pub(crate) note_id: u32,
}

impl Voice {
    pub fn new(config: &VoiceConfig, envelope_config: &EnvelopeConfig, sample_rate: f32) -> Self {
        // Create polymorphic oscillators with a harmless initial frequency (will be set on trigger)
        let init_freq = 440.0;
        let oscillators = config
            .oscillator_configs
            .iter()
            .cloned()
            .map(|cfg| make_oscillator(cfg, sample_rate, init_freq))
            .collect::<Vec<_>>();

        Self {
            frequency: 0.0,
            oscillators,
            envelope: Envelope::new(envelope_config.clone(), sample_rate),
            filter: config.filter.clone(),
            is_active: false,
            note_id: 0,
        }
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f32) {
        self.envelope.update_sample_rate(new_sample_rate);
        for osc in &mut self.oscillators {
            osc.update_sample_rate(new_sample_rate);
        }
        self.filter.update_sample_rate(new_sample_rate);
    }

    pub fn trigger(&mut self, frequency: f32, note_id: u32, other_env_value: Option<f32>) {
        self.frequency = frequency;
        self.note_id = note_id;
        self.is_active = true;

        // Retrigger or continue from current env value depending on config
        self.envelope.trigger(other_env_value);

        // Retune all oscillators for this note
        for osc in &mut self.oscillators {
            osc.set_frequency(frequency);
        }
    }

    pub fn release(&mut self, note_id: u32) {
        if self.note_id == note_id {
            self.envelope.release();
        }
    }

    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }

    pub fn next_sample(&mut self) -> f32 {
        let env = self.envelope.next_value();

        let osc_sum = self
            .oscillators
            .iter_mut()
            .map(|osc| osc.next_sample())
            .sum::<f32>();

        let enveloped = osc_sum * env;
        self.filter.process_sample(enveloped)
    }

    pub fn get_envelope_value(&self) -> f32 {
        self.envelope.current_value()
    }
}

// Clone via box_clone() for the oscillators
impl Clone for Voice {
    fn clone(&self) -> Self {
        Self {
            frequency: self.frequency,
            oscillators: self.oscillators.iter().map(|o| o.box_clone()).collect(),
            envelope: self.envelope.clone(),
            filter: self.filter.clone(),
            is_active: self.is_active,
            note_id: self.note_id,
        }
    }
}
