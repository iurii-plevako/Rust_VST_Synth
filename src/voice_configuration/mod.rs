use std::sync::{Arc, Mutex};

use crate::envelope::Envelope;

#[derive(Clone, Copy)]
pub enum Waveform {
  SINE,
  SAW,
  SQUARE,
  RANDOM,
  WHITE_NOISE,
}

pub struct VoiceConfiguration {
  pub waveform: Waveform,
  pub envelope: Arc<Mutex<Envelope>>,
}

impl VoiceConfiguration {
  pub fn new(waveform: Waveform, envelope: Arc<Mutex<Envelope>>) -> Self {
    Self {
      waveform: waveform,
      envelope: envelope,
    }
  }
}