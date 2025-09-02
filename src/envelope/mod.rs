use crate::filter::ModulationSource;

#[derive(Clone)]
pub struct Envelope {
    config: EnvelopeConfig,
    current_value: f32,
    current_state: EnvelopeState,
    sample_rate: f32,
    attack_increment: f32,
    decay_increment: f32,
    release_increment: f32,
}

impl Envelope {
    pub fn new(config: EnvelopeConfig, sample_rate: f32) -> Self {
        let attack_increment = 1.0 / (config.attack_time * sample_rate);
        let decay_increment = (1.0 - config.sustain_level) / (config.decay_time * sample_rate);
        let release_increment = config.sustain_level / (config.release_time * sample_rate);

        Self {
            config,
            current_value: 0.0,
            current_state: EnvelopeState::Idle,
            sample_rate,
            attack_increment,
            decay_increment,
            release_increment,
        }
    }

    pub fn current_value(&self) -> f32 {
        self.current_value
    }

    pub fn trigger(&mut self, other_value: Option<f32>) {
        if !self.config.retrigger && other_value.is_some() {
            self.current_value = other_value.unwrap();
            self.attack_increment = (1.0 - self.current_value) / (self.config.attack_time * self.sample_rate);
        } else {
            self.current_value = 0.0;
        }
        self.current_state = EnvelopeState::Attack;
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f32) {
        self.sample_rate = new_sample_rate;
        self.attack_increment = 1.0 / (self.config.attack_time * new_sample_rate);
        self.decay_increment = (1.0 - self.config.sustain_level) / (self.config.decay_time * new_sample_rate);
        self.release_increment = self.config.sustain_level / (self.config.release_time * new_sample_rate);
    }

    pub fn is_active(&self) -> bool {
        match self.current_state {
            EnvelopeState::Idle => false,
            EnvelopeState::Release => self.current_value > 0.00001, // Consider envelope done when nearly silent
            _ => true
        }
    }

    pub fn release(&mut self) {
        if self.current_state != EnvelopeState::Idle {
            self.current_state = EnvelopeState::Release;
        }
    }

    pub fn next_value(&mut self) -> f32 {
        match self.current_state {
            EnvelopeState::Idle => 0.0,

            EnvelopeState::Attack => {
                self.current_value = (self.current_value + self.attack_increment)
                    .clamp(0.0, 1.0);

                if self.current_value >= 1.0 {
                    self.current_state = EnvelopeState::Decay;
                }
                self.current_value
            }

            EnvelopeState::Decay => {
                self.current_value = (self.current_value - self.decay_increment)
                    .clamp(self.config.sustain_level, 1.0);

                if self.current_value <= self.config.sustain_level {
                    self.current_state = EnvelopeState::Sustain;
                }
                self.current_value
            }

            EnvelopeState::Sustain => {
                self.current_value = self.config.sustain_level;
                self.current_value
            }

            EnvelopeState::Release => {
                self.current_value = (self.current_value - self.release_increment)
                    .clamp(0.0, 1.0);

                if self.current_value <= 0.001 {
                    self.current_state = EnvelopeState::Idle;
                    self.current_value = 0.0;
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

impl ModulationSource for Envelope {
    fn next_value(&mut self) -> f32 {
        self.next_value()
    }

    fn is_active(&self) -> bool {
        self.is_active()
    }

    fn reset(&mut self) {
        self.trigger(None);
    }
}

#[derive(Clone)]
pub struct EnvelopeConfig {
    pub attack_time: f32,
    pub decay_time: f32,
    pub sustain_level: f32,
    pub release_time: f32,
    pub retrigger: bool,
}

impl EnvelopeConfig {
    pub fn new(attack_time: f32, decay_time: f32, sustain_level: f32, release_time: f32, retrigger: bool) -> Self {
        Self {
            attack_time,
            decay_time,
            sustain_level,
            release_time,
            retrigger,
        }
    }
}