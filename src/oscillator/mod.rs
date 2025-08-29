pub mod basic_oscillator;
pub mod random_oscillator;

pub use basic_oscillator::BasicOscillator;
pub use random_oscillator::RandomOscillator;

use crate::voice_configuration::Waveform;

pub trait WaveformGenerator: Send + Sync {
    fn next_sample(&mut self) -> f64;
    fn update_sample_rate(&mut self, new_sample_rate: f64);
    fn volume(&self) -> f64;
    fn box_clone(&self) -> Box<dyn WaveformGenerator>;
}

#[derive(Clone, Copy)]
pub struct OscillatorConfig {
    pub waveform: Waveform,
    pub detune_semitones: f64,
    pub volume: f64,
}