use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Clone)]
pub struct Envelope {
    pub attack_time: f64,
    pub decay_time: f64,
    pub sustain_level: f64,
    pub release_time: f64,
    current_value: f64,
    current_state: EnvelopeState,
    target_value: f64,
    rate: f64,
    sample_rate: f64,
}

impl Envelope {
    pub fn new(attack_time: f64, decay_time: f64, sustain_level: f64, release_time: f64, sample_rate: f64) -> Self {
        Envelope {
            attack_time,
            decay_time,
            sustain_level,
            release_time,
            current_value: 0.0,
            current_state: EnvelopeState::Idle,
            target_value: 0.0,
            rate: 0.0,
            sample_rate,
        }
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f64) {
        // Store the new sample rate
        self.sample_rate = new_sample_rate;

        // Recalculate the current rate based on the current state
        match self.current_state {
            EnvelopeState::Attack => {
                self.rate = 1.0 / (self.attack_time * new_sample_rate);
            }
            EnvelopeState::Decay => {
                self.rate = (1.0 - self.sustain_level) / (self.decay_time * new_sample_rate);
            }
            EnvelopeState::Release => {
                self.rate = self.current_value / (self.release_time * new_sample_rate);
            }
            _ => {}
        }
    }

    pub fn is_active(&self) -> bool {
        // Envelope is active until it fully completes the release phase
        match self.current_state {
            EnvelopeState::Idle => false,
            EnvelopeState::Release => self.current_value > 0.00001, // Consider envelope done when nearly silent
            _ => true
        }
    }

    pub fn trigger(&mut self) {
        self.current_state = EnvelopeState::Attack;
        self.target_value = 1.0;
        self.rate = 1.0 / (self.attack_time * self.sample_rate);
    }

    pub fn release(&mut self) {
        if self.current_state != EnvelopeState::Idle {
            self.current_state = EnvelopeState::Release;
            self.target_value = 0.0;
            let old_rate = self.rate;
            self.rate = self.current_value / (self.release_time * self.sample_rate);
            println!("Release triggered:");
            println!("- Current value: {}", self.current_value);
            println!("- Sample rate: {}", self.sample_rate);
            println!("- Release time: {}", self.release_time);
            println!("- Calculated rate: {}", self.rate);
            println!("- Samples to process: {}", self.release_time * self.sample_rate);

        }
    }


    pub fn next_value(&mut self) -> f64 {
        match self.current_state {
            EnvelopeState::Idle => 0.0,
            EnvelopeState::Attack => {
                self.current_value += self.rate;
                if self.current_value >= 1.0 {
                    self.current_value = 1.0;
                    self.current_state = EnvelopeState::Decay;
                    self.target_value = self.sustain_level;
                    self.rate = (1.0 - self.sustain_level) / (self.decay_time * self.sample_rate);
                }
                self.current_value
            }
            EnvelopeState::Decay => {
                self.current_value -= self.rate;
                if self.current_value <= self.sustain_level {
                    self.current_value = self.sustain_level;
                    self.current_state = EnvelopeState::Sustain;
                }
                self.current_value
            }
            EnvelopeState::Sustain => self.sustain_level,
            EnvelopeState::Release => {
                self.current_value -= self.rate;
                if self.current_value <= 0.00001 {
                    self.current_value = 0.0;
                    self.current_state = EnvelopeState::Idle;
                }
                self.current_value
            }
        }
    }
}

#[derive(PartialEq, Clone)]
enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}