use std::sync::{atomic::Ordering, Arc};
use std::time::Instant;
use atomic_float::AtomicF64;
use crate::filter::ModulationSource;

#[derive(Clone)]
pub struct Envelope {
    pub attack_time: f64,
    pub decay_time: f64,
    pub sustain_level: f64,
    pub release_time: f64,
    current_value: Arc<AtomicF64>,
    current_state: EnvelopeState,
    rate: f64,
    sample_rate: f64,
    release_start: Option<Instant>,
    attack_start: Option<Instant>,
    decay_start: Option<Instant>,
    sample_count: Option<u32>,
}

impl Envelope {
    pub fn new(attack_time: f64, decay_time: f64, sustain_level: f64, release_time: f64, sample_rate: f64) -> Self {
        Envelope {
            attack_time,
            decay_time,
            sustain_level,
            release_time,
            current_value: Arc::new(AtomicF64::new(0.0)),
            current_state: EnvelopeState::Idle,
            rate: 0.0,
            sample_rate,
            release_start: None,
            attack_start: None,
            decay_start: None,
            sample_count: None,
        }
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f64) {
        self.sample_rate = new_sample_rate;

        match self.current_state {
            EnvelopeState::Attack => {
                let samples_for_attack = self.attack_time * new_sample_rate;
                self.rate = (1.0 - self.current_value.load(Ordering::Relaxed)) / samples_for_attack;
            }
            EnvelopeState::Decay => {
                let samples_for_decay = self.decay_time * new_sample_rate;
                self.rate = (self.sustain_level - self.current_value.load(Ordering::Relaxed)) / samples_for_decay;
            }
            EnvelopeState::Release => {
                let samples_for_release = self.release_time * new_sample_rate;
                self.rate = -self.current_value.load(Ordering::Relaxed) / samples_for_release;
            }
            _ => {}
        }

    }

    pub fn is_active(&self) -> bool {
        // Envelope is active until it fully completes the release phase
        match self.current_state {
            EnvelopeState::Idle => false,
            EnvelopeState::Release => self.current_value.load(Ordering::Relaxed) > 0.00001, // Consider envelope done when nearly silent
            _ => true
        }
    }

    pub fn trigger(&mut self) {
        self.sample_count = Option::Some(0);
        self.current_state = EnvelopeState::Attack;
        let samples_for_attack = (self.attack_time * self.sample_rate).ceil() as u32;
        self.current_value = Arc::new(AtomicF64::new(0.000));

        // Calculate rate to reach 1.0 over exact number of samples
        self.rate = (1.0 - self.current_value.load(Ordering::Relaxed)) / samples_for_attack as f64;

        self.attack_start = Some(Instant::now());
        println!("Attack started:");
        println!("- Initial value: {}", self.current_value.load(Ordering::Relaxed));
        println!("- Samples to process: {}", samples_for_attack);
        println!("- Rate (increase per sample): {}", self.rate);
    }

    pub fn release(&mut self) {
        if self.current_state != EnvelopeState::Idle {
            self.current_state = EnvelopeState::Release;
            let samples_for_release = (self.release_time * self.sample_rate).ceil() as u32;

            // Calculate rate to reach 0.0 over exact number of samples
            self.rate = -self.current_value.load(Ordering::Relaxed) / samples_for_release as f64;

            self.release_start = Some(Instant::now());
            println!("Release started:");
            println!("- From value: {}", self.current_value.load(Ordering::Relaxed));
            println!("- Samples to process: {}", samples_for_release);
            println!("- Rate (change per sample): {}", self.rate);
        }
    }



    pub fn next_value(&mut self) -> f64 {
        match self.current_state {
            EnvelopeState::Idle => 0.0,
            EnvelopeState::Attack => {
                let new_value = (self.current_value.load(Ordering::Relaxed) + self.rate)
                    .clamp(0.0, 1.0);  // Clamp to valid range
                self.current_value.store(new_value, Ordering::Relaxed);

                if let Some(count) = self.sample_count.as_mut() {
                    *count += 1;
                }

                // Transition based on value, not time
                if new_value >= 1.0 {
                    println!("Attack completed after {} samples", self.sample_count.unwrap());
                    self.attack_start = None;
                    self.current_state = EnvelopeState::Decay;

                    let samples_for_decay = (self.decay_time * self.sample_rate).ceil() as u32;
                    self.rate = (self.sustain_level - new_value) / samples_for_decay as f64;
                    self.decay_start = Some(Instant::now());

                    println!("Decay started:");
                    println!("- From value: {}", new_value);
                    println!("- Target sustain: {}", self.sustain_level);
                    println!("- Samples to process: {}", samples_for_decay);
                    println!("- Rate (change per sample): {}", self.rate);
                }
                new_value
            }
            EnvelopeState::Decay => {
                let new_value = (self.current_value.load(Ordering::Relaxed) + self.rate)
                    .clamp(self.sustain_level, 1.0);
                self.current_value.store(new_value, Ordering::Relaxed);

                // Transition based on value
                if new_value <= self.sustain_level {
                    println!("Decay completed");
                    self.decay_start = None;
                    self.current_state = EnvelopeState::Sustain;
                    println!("Sustain reached at level: {}", new_value);
                }
                new_value
            }
            EnvelopeState::Sustain => self.sustain_level,
            EnvelopeState::Release => {
                let new_value = (self.current_value.load(Ordering::Relaxed) + self.rate)
                    .clamp(0.0, 1.0);  // Allow going down to silence
                self.current_value.store(new_value, Ordering::Relaxed);

                // Transition based on value
                if new_value <= 0.001 {  // Small threshold for silence
                    println!("Release completed at value {:.4}", new_value);
                    self.release_start = None;
                    self.current_state = EnvelopeState::Idle;
                    self.current_value.store(0.0, Ordering::Relaxed);
                }
                new_value
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
    fn next_value(&mut self) -> f64 {
        self.next_value()  // reuse existing next_value method
    }

    fn is_active(&self) -> bool {
        self.is_active()   // reuse existing is_active method
    }

    fn reset(&mut self) {
        self.trigger();    // reuse trigger as reset
    }
}
