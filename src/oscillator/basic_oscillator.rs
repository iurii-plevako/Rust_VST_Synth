use super::{OscillatorConfig, WaveformGenerator};
use crate::voice_configuration::Waveform;

#[derive(Clone)]
pub struct BasicOscillator {
    config: OscillatorConfig,
    sample_rate: f32,
    frequency: f32,
    phase: f32,
    rng: u64,
}

impl BasicOscillator {
    pub fn new(sample_rate: f32, base_frequency: f32, config: OscillatorConfig) -> Self {
        Self {
            config,
            sample_rate,
            frequency: base_frequency * (2.0f32.powf(config.detune_semitones / 12.0)),
            phase: 0.0,
            rng: 12345,
        }
    }

    fn next_random(&mut self) -> f32 {
        self.rng = self.rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.rng >> 32) as f32) / ((u32::MAX as f32) + 1.0) * 2.0 - 1.0
    }
}

impl WaveformGenerator for BasicOscillator {
    fn next_sample(&mut self) -> f32 {
        let value = match self.config.waveform {
            Waveform::SINE => (self.phase * 2.0 * std::f32::consts::PI).sin(),
            Waveform::SAW => 2.0 * (self.phase - 0.5),
            Waveform::SQUARE => if self.phase < 0.5 { 1.0 } else { -1.0 },
            Waveform::RANDOM => (self.phase * 2.0 * std::f32::consts::PI).sin(),
            Waveform::WHITE_NOISE => self.next_random(),
        };

        self.phase = (self.phase + self.frequency / self.sample_rate) % 1.0;
        value * self.config.volume
    }

    fn update_sample_rate(&mut self, new_sample_rate: f32) {
        self.sample_rate = new_sample_rate;
    }

    fn set_frequency(&mut self, freq_hz: f32) {
        self.frequency = freq_hz * 2.0f32.powf(self.config.detune_semitones / 12.0);
    }

    fn volume(&self) -> f32 {
        self.config.volume
    }

    fn box_clone(&self) -> Box<dyn WaveformGenerator> {
        Box::new(self.clone())
    }
}
