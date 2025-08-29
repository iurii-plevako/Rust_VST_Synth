use std::time::Instant;

pub struct Envelope {
    pub start_time: Option<Instant>,  // Changed to Option
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub is_released: bool,
    pub release_start: Option<Instant>,
    pub last_value: f32,
}

impl Envelope {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            start_time: None,  // Initialize as None
            attack,
            decay,
            sustain,
            release,
            is_released: false,
            release_start: None,
            last_value: 0.0,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn get_amplitude(&mut self) -> f32 {
        let start = self.start_time.expect("Envelope not started");
        let now = Instant::now();
        let elapsed = (now - start).as_secs_f32();

        // Rest of the implementation remains the same
        if self.is_released {
            let release_elapsed = (now - self.release_start.unwrap()).as_secs_f32();
            if release_elapsed >= self.release {
                return 0.0;
            }
            return self.last_value * (1.0 - (release_elapsed / self.release));
        }

        let amplitude = if elapsed < self.attack {
            elapsed / self.attack
        } else if elapsed < (self.attack + self.decay) {
            let decay_elapsed = elapsed - self.attack;
            1.0 - ((1.0 - self.sustain) * (decay_elapsed / self.decay))
        } else {
            self.sustain
        };

        self.last_value = amplitude;
        amplitude
    }

    pub fn release(&mut self) {
        if !self.is_released {
            self.is_released = true;
            self.release_start = Some(Instant::now());
        }
    }
}