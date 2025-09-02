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
    pub cutoff_frequency: f32,      // Hz
    pub resonance_amount: f32,      // 0.0 to 1.0
    pub modulation_amount: f32,     // Amount of modulation applied
}

pub trait ModulationSource: Send + Sync {
    fn next_value(&mut self) -> f32;  // Returns value between 0.0 and 1.0
    fn is_active(&self) -> bool;
    fn reset(&mut self);
}

#[derive(Clone)]
pub struct Filter {
    parameters: FilterParameters,
    sample_rate: f32,
    modulation_sources: Vec<Arc<Mutex<dyn ModulationSource>>>,
    filter_stages: Vec<FilterStage>,
}

#[derive(Clone)]
struct FilterStage {
    prev_input: f32,         // Previous input sample
    prev_prev_input: f32,    // Second previous input sample
    prev_output: f32,        // Previous output sample
    prev_prev_output: f32,   // Second previous output sample
}

impl FilterStage {
    fn new() -> Self {
        Self {
            prev_input: 0.0,
            prev_prev_input: 0.0,
            prev_output: 0.0,
            prev_prev_output: 0.0,
        }
    }
}

impl Filter {
    pub fn new(parameters: FilterParameters, sample_rate: f32) -> Self {
        let stages_count = match parameters.slope {
            FilterSlope::Slope6dB => 1,
            FilterSlope::Slope12dB => 2,
            FilterSlope::Slope24dB => 4,
        };

        Self {
            parameters,
            sample_rate,
            modulation_sources: Vec::new(),
            filter_stages: (0..stages_count).map(|_| FilterStage::new()).collect(),
        }
    }

    pub fn add_modulation_source(&mut self, source: Arc<Mutex<dyn ModulationSource>>) {
        self.modulation_sources.push(source);
    }

    pub fn process_sample(&mut self, input_sample: f32) -> f32 {
        // Calculate modulated cutoff frequency
        let mut modulated_freq = self.parameters.cutoff_frequency;
        
        // Apply all modulation sources
        for source in &mut self.modulation_sources {
            if let Ok(mut source) = source.lock() {
                if source.is_active() {
                    let mod_value = source.next_value();
                    let scaled_modulation = mod_value * self.parameters.modulation_amount;
                    // Exponential frequency modulation
                    modulated_freq *= 2.0f32.powf(scaled_modulation * 10.0);
                }
            }
        }

        // Clamp frequency between 20Hz and Nyquist
        let clamped_freq = modulated_freq.clamp(20.0, self.sample_rate * 0.49);
        let (feedback1, feedback2, feed0, feed1, feed2) = 
            self.calculate_coefficients(clamped_freq);

        // Process through all stages in series
        let mut processed_sample = input_sample;
        for stage in &mut self.filter_stages {
            processed_sample = process_filter_stage(
                stage, 
                processed_sample, 
                feedback1, 
                feedback2, 
                feed0, 
                feed1, 
                feed2
            );
        }

        processed_sample
    }

    fn calculate_coefficients(&self, cutoff_freq: f32) -> (f32, f32, f32, f32, f32) {
        let angular_freq = 2.0 * std::f32::consts::PI * cutoff_freq / self.sample_rate;
        let cosine = angular_freq.cos();
        let resonance_factor = angular_freq.sin() / (2.0 * self.parameters.resonance_amount);

        match self.parameters.filter_type {
            FilterType::LowPass => {
                let feedforward0 = (1.0 - cosine) / 2.0;
                let feedforward1 = 1.0 - cosine;
                let feedforward2 = (1.0 - cosine) / 2.0;
                let feedback0 = 1.0 + resonance_factor;
                let feedback1 = -2.0 * cosine;
                let feedback2 = 1.0 - resonance_factor;

                (
                    feedback1/feedback0, 
                    feedback2/feedback0, 
                    feedforward0/feedback0, 
                    feedforward1/feedback0, 
                    feedforward2/feedback0
                )
            },
            FilterType::HighPass => {
                let feedforward0 = (1.0 + cosine) / 2.0;
                let feedforward1 = -(1.0 + cosine);
                let feedforward2 = (1.0 + cosine) / 2.0;
                let feedback0 = 1.0 + resonance_factor;
                let feedback1 = -2.0 * cosine;
                let feedback2 = 1.0 - resonance_factor;

                (
                    feedback1/feedback0, 
                    feedback2/feedback0, 
                    feedforward0/feedback0, 
                    feedforward1/feedback0, 
                    feedforward2/feedback0
                )
            }
        }
    }

    pub fn update_sample_rate(&mut self, new_sample_rate: f32) {
        self.sample_rate = new_sample_rate;
    }
}

fn process_filter_stage(
    stage: &mut FilterStage, 
    input_sample: f32, 
    feedback1: f32, 
    feedback2: f32, 
    feed0: f32, 
    feed1: f32, 
    feed2: f32
) -> f32 {
    let output = feed0 * input_sample + 
                feed1 * stage.prev_input + 
                feed2 * stage.prev_prev_input -
                feedback1 * stage.prev_output - 
                feedback2 * stage.prev_prev_output;

    // Update state
    stage.prev_prev_input = stage.prev_input;
    stage.prev_input = input_sample;
    stage.prev_prev_output = stage.prev_output;
    stage.prev_output = output;

    output
}