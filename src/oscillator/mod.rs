pub mod basic_oscillator;
pub mod random_oscillator;

pub use basic_oscillator::BasicOscillator;
pub use random_oscillator::RandomOscillator;

use crate::voice_configuration::Waveform;

pub trait WaveformGenerator: Send + Sync {
    fn next_sample(&mut self) -> f32;
    fn update_sample_rate(&mut self, new_sample_rate: f32);
    fn set_frequency(&mut self, freq_hz: f32);          // NEW: allow retuning on note-on
    fn volume(&self) -> f32;
    fn box_clone(&self) -> Box<dyn WaveformGenerator>;
}

#[derive(Clone, Copy)]
pub struct OscillatorConfig {
    pub waveform: Waveform,
    pub detune_semitones: f32,
    pub volume: f32,
}

/// Small factory so Voice can construct polymorphic oscillators cleanly.
pub fn make_oscillator(
    cfg: OscillatorConfig,
    sample_rate: f32,
    init_freq_hz: f32,
) -> Box<dyn WaveformGenerator> {
    match cfg.waveform {
        Waveform::RANDOM => Box::new(RandomOscillator::new(sample_rate, init_freq_hz, cfg)),
        _ => Box::new(BasicOscillator::new(sample_rate, init_freq_hz, cfg)),
    }
}
