// src/filter/mod.rs
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
pub enum FilterType {
    LowPass,
    HighPass,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FilterSlope {
    Slope6dB,   // 1-pole
    Slope12dB,  // 2-pole
    Slope24dB   // 4-pole
}

#[derive(Clone)]
pub struct FilterParameters {
    pub filter_type: FilterType,
    pub slope: FilterSlope,
    pub cutoff: f64,      // Hz
    pub resonance: f64,   // 0.0 to 1.0
}

// Trait for any modulation source (envelope, LFO, etc.)
pub trait ModulationSource: Send + Sync {
    fn next_value(&mut self) -> f64;  // Returns value between 0.0 and 1.0
    fn is_active(&self) -> bool;
    fn reset(&mut self);
}

#[derive(Clone)]
pub struct Filter {
    parameters: FilterParameters,
    sample_rate: f64,
    modulation_sources: Vec<Arc<Mutex<dyn ModulationSource>>>,
    
    // State variables for each filter stage
    stages: Vec<FilterStage>,
}

#[derive(Clone)]
struct FilterStage {
    x1: f64,  // Previous input
    x2: f64,  // Second previous input
    y1: f64,  // Previous output
    y2: f64,  // Second previous output
}

impl FilterStage {
    fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}

impl Filter {
    pub fn new(parameters: FilterParameters, sample_rate: f64) -> Self {
        let num_stages = match parameters.slope {
            FilterSlope::Slope6dB => 1,
            FilterSlope::Slope12dB => 2,
            FilterSlope::Slope24dB => 4,
        };

        Self {
            parameters,
            sample_rate,
            modulation_sources: Vec::new(),
            stages: (0..num_stages).map(|_| FilterStage::new()).collect(),
        }
    }

    pub fn add_modulation_source(&mut self, source: Arc<Mutex<dyn ModulationSource>>) {
        self.modulation_sources.push(source);
    }

    pub fn process_sample(&mut self, input: f64) -> f64 {
        // Calculate modulated cutoff frequency
        let mut modulated_cutoff = self.parameters.cutoff;
        
        // Apply all modulation sources
        for source in &mut self.modulation_sources {
            if let Ok(mut source) = source.lock() {
                if source.is_active() {
                    let mod_value = source.next_value();
                    modulated_cutoff *= 2.0f64.powf(mod_value * 10.0);
                }
            }
        }

        modulated_cutoff = modulated_cutoff.clamp(20.0, self.sample_rate * 0.49);
        let (a1, a2, b0, b1, b2) = self.calculate_coefficients(modulated_cutoff);

        // Process through all stages in series
        let mut output = input;
        for stage in &mut self.stages {
            output = process_filter_stage(stage, output, a1, a2, b0, b1, b2);
        }

        output
    }

    fn calculate_coefficients(&self, cutoff: f64) -> (f64, f64, f64, f64, f64) {
        let w0 = 2.0 * std::f64::consts::PI * cutoff / self.sample_rate;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * self.parameters.resonance);

        match self.parameters.filter_type {
            FilterType::LowPass => {
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;

                (a1/a0, a2/a0, b0/a0, b1/a0, b2/a0)
            },
            FilterType::HighPass => {
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;

                (a1/a0, a2/a0, b0/a0, b1/a0, b2/a0)
            }
        }
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f64) {
        self.sample_rate = new_sample_rate;
    }
}

fn process_filter_stage(stage: &mut FilterStage, input: f64, a1: f64, a2: f64, b0: f64, b1: f64, b2: f64) -> f64 {
    let output = b0 * input + b1 * stage.x1 + b2 * stage.x2
        - a1 * stage.y1 - a2 * stage.y2;

    // Update state
    stage.x2 = stage.x1;
    stage.x1 = input;
    stage.y2 = stage.y1;
    stage.y1 = output;

    output
}
