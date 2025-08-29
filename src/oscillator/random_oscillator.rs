use super::{OscillatorConfig, WaveformGenerator};
use std::f64::consts::PI;
use crate::voice_configuration::Waveform;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct RandomOscillator {
    config: OscillatorConfig,
    sample_rate: f64,
    frequency: f64,
    phase: f64,
    wavetable: Vec<f64>,
    wavetable_size: usize,
}

impl RandomOscillator {
    pub fn new(sample_rate: f64, base_frequency: f64, config: OscillatorConfig) -> Self {
        const WAVETABLE_SIZE: usize = 4096; // Increased size for more detail
        let mut wavetable = Vec::with_capacity(WAVETABLE_SIZE);
        
        // Get a seed based on system time for randomness
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        // Simple random number generator
        let mut rng = seed;
        let mut random = move || {
            rng = rng.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((rng >> 32) as f64) / ((u32::MAX as f64) + 1.0)
        };

        // Generate base harmonics with random phases and amplitudes
        let num_harmonics = 12;
        let mut harmonic_phases: Vec<f64> = (0..num_harmonics)
            .map(|_| random() * 2.0 * PI)
            .collect();
        let mut harmonic_amps: Vec<f64> = (0..num_harmonics)
            .map(|i| {
                let base_amp = 1.0 / (i as f64 + 1.0).powf(0.7); // Less steep falloff
                base_amp * (0.5 + 0.5 * random()) // Random amplitude variation
            })
            .collect();

        // Generate the wavetable
        for i in 0..WAVETABLE_SIZE {
            let phase = (i as f64 / WAVETABLE_SIZE as f64) * 2.0 * PI;
            let mut sample = 0.0;

            // Add harmonics with random phase and amplitude modulation
            for h in 0..num_harmonics {
                let harmonic = h + 1;
                let harmonic_phase = harmonic_phases[h];
                let mut amplitude = harmonic_amps[h];

                // Add some random wobble to the amplitude
                amplitude *= 1.0 + 0.1 * (phase * 4.37 + harmonic_phase).sin();

                sample += amplitude * (phase * harmonic as f64 + harmonic_phase).sin();

                // Add some frequency modulation for more character
                if h < 3 {  // Only for first few harmonics
                    sample += 0.1 * amplitude * 
                        (phase * harmonic as f64 * (1.0 + 0.02 * (phase * 3.17).sin())).sin();
                }
            }

            // Add some noise components
            for _ in 0..3 {
                let noise_freq = 1.0 + random() * 10.0;
                let noise_phase = random() * 2.0 * PI;
                sample += 0.05 * (phase * noise_freq + noise_phase).sin();
            }

            wavetable.push(sample);
        }

        // Smooth the waveform slightly to reduce aliasing
        let mut smoothed = vec![0.0; WAVETABLE_SIZE];
        for i in 0..WAVETABLE_SIZE {
            let prev = wavetable[(i + WAVETABLE_SIZE - 1) % WAVETABLE_SIZE];
            let next = wavetable[(i + 1) % WAVETABLE_SIZE];
            smoothed[i] = 0.25 * prev + 0.5 * wavetable[i] + 0.25 * next;
        }

        // Normalize
        let max_amplitude = smoothed.iter().map(|x| x.abs()).fold(0.0, f64::max);
        for sample in &mut smoothed {
            *sample /= max_amplitude;
        }

        Self {
            config,
            sample_rate,
            frequency: base_frequency * (2.0f64.powf(config.detune_semitones / 12.0)),
            phase: 0.0,
            wavetable: smoothed,
            wavetable_size: WAVETABLE_SIZE,
        }
    }
}

impl WaveformGenerator for RandomOscillator {
    fn next_sample(&mut self) -> f64 {
        // Cubic interpolation for smoother playback
        let index_f = self.phase * self.wavetable_size as f64;
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