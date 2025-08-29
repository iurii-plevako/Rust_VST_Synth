// oscillator.rs
use dasp::{signal, Signal};
use crate::voice_configuration::Waveform;

#[derive(Clone, Copy)]
pub struct OscillatorConfig {
    pub waveform: Waveform,
    pub detune_semitones: f64,
    pub volume: f64,
}

#[derive(Clone)]
pub struct Oscillator {
    config: OscillatorConfig,
    sample_rate: f64,
    frequency: f64,
    phase: f64,

}

impl Oscillator {
    pub fn new(sample_rate: f64, base_frequency: f64, config: OscillatorConfig) -> Self {
        Self {
            config,
            sample_rate,
            frequency: base_frequency * (2.0f64.powf(config.detune_semitones / 12.0)),
            phase: 0.0,
        }
    }



    pub fn next_sample(&mut self) -> f64 {
        let value = match self.config.waveform {
            Waveform::SINE => (self.phase * 2.0 * std::f64::consts::PI).sin(),
            Waveform::SAW => 2.0 * (self.phase - 0.5),
            Waveform::SQUARE => if self.phase < 0.5 { 1.0 } else { -1.0 },
        };

        self.phase = (self.phase + self.frequency / self.sample_rate) % 1.0;
        value * self.config.volume
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f64) {
        self.sample_rate = new_sample_rate;
    }
}