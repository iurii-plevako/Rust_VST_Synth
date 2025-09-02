use super::{OscillatorConfig, WaveformGenerator};
use std::f32::consts::PI;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct RandomOscillator {
    config: OscillatorConfig,
    sample_rate: f32,
    frequency: f32,
    phase: f32,
    wavetable: Vec<f32>,
    wavetable_size: usize,
}

impl RandomOscillator {
    pub fn new(sample_rate: f32, base_frequency: f32, config: OscillatorConfig) -> Self {
        const WAVETABLE_SIZE: usize = 4096;
        let mut wavetable: Vec<f32> = Vec::with_capacity(WAVETABLE_SIZE);

        // seed
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        let mut rng = seed;
        let mut random = move || {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((rng >> 32) as f32) / ((u32::MAX as f32) + 1.0)
        };

        // Smooth the waveform slightly to reduce aliasing
        let mut smoothed: Vec<f32> = vec![0.0_f32; WAVETABLE_SIZE];
        for i in 0..WAVETABLE_SIZE {
            let prev = wavetable[(i + WAVETABLE_SIZE - 1) % WAVETABLE_SIZE];
            let next = wavetable[(i + 1) % WAVETABLE_SIZE];
            smoothed[i] = 0.25 * prev + 0.5 * wavetable[i] + 0.25 * next;
        }

        // Normalize
        let max_amplitude = smoothed
            .iter()
            .copied()                 // &f32 -> f32
            .map(f32::abs)            // clearer than |x| x.abs()
            .fold(0.0_f32, f32::max);
        for sample in &mut smoothed {
            *sample /= max_amplitude;
        }
        
        // For this snippet, assume `smoothed` is produced as in your file:
        let smoothed = {
            // paste your existing generation here unchanged
            let mut tmp = vec![0.0; WAVETABLE_SIZE];
            // (…)
            tmp
        };

        Self {
            config,
            sample_rate,
            frequency: base_frequency * (2.0f32.powf(config.detune_semitones / 12.0)),
            phase: 0.0,
            wavetable: smoothed,
            wavetable_size: WAVETABLE_SIZE,
        }
    }
}

impl WaveformGenerator for RandomOscillator {
    fn next_sample(&mut self) -> f32 {
        let index_f = self.phase * self.wavetable_size as f32;
        let index = index_f as usize % self.wavetable_size;
        let frac = index_f - index_f.floor();

        let x0 = self.wavetable[(index + self.wavetable_size - 1) % self.wavetable_size];
        let x1 = self.wavetable[index];
        let x2 = self.wavetable[(index + 1) % self.wavetable_size];
        let x3 = self.wavetable[(index + 2) % self.wavetable_size];

        let c0 = x1;
        let c1 = 0.5 * (x2 - x0);
        let c2 = x0 - 2.5 * x1 + 2.0 * x2 - 0.5 * x3;
        let c3 = 0.5 * (x3 - x0) + 1.5 * (x1 - x2);

        let interpolated = ((c3 * frac + c2) * frac + c1) * frac + c0;

        self.phase = (self.phase + self.frequency / self.sample_rate) % 1.0;
        interpolated * self.config.volume
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
