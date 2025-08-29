use super::{OscillatorConfig, WaveformGenerator};
use crate::voice_configuration::Waveform;

#[derive(Clone)]
pub struct BasicOscillator {
    config: OscillatorConfig,
    sample_rate: f64,
    frequency: f64,
    phase: f64,
}

impl BasicOscillator {
    pub fn new(sample_rate: f64, base_frequency: f64, config: OscillatorConfig) -> Self {
        println!("Creating basic oscillator");
        Self {
            config,
            sample_rate,
            frequency: base_frequency * (2.0f64.powf(config.detune_semitones / 12.0)),
            phase: 0.0,
        }
    }
}

impl WaveformGenerator for BasicOscillator {
    fn next_sample(&mut self) -> f64 {
        let value = match self.config.waveform {
            Waveform::SINE => (self.phase * 2.0 * std::f64::consts::PI).sin(),
            Waveform::SAW => 2.0 * (self.phase - 0.5),
            Waveform::SQUARE => if self.phase < 0.5 { 1.0 } else { -1.0 },
            Waveform::RANDOM => (self.phase * 2.0 * std::f64::consts::PI).sin(), // Fall back to sine wave

        };

        self.phase = (self.phase + self.frequency / self.sample_rate) % 1.0;
        value * self.config.volume
    }

    fn update_sample_rate(&mut self, new_sample_rate: f64) {
        self.sample_rate = new_sample_rate;
    }

    fn volume(&self) -> f64 {
        self.config.volume
    }

    fn box_clone(&self) -> Box<dyn WaveformGenerator> {
        Box::new(self.clone())
    }
}