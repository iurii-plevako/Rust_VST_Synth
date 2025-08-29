use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dasp::{signal, Signal};
use std::{sync::{Arc, Mutex}, time::Duration};

use crate::envelope::Envelope;

pub struct SoundModule {
}

impl SoundModule {
    pub fn new() -> Self {
        Self {}
    }

    pub fn play_note(&self) {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .expect("no output device available");
        
        let config = device.default_output_config()
            .expect("no default output config");
            
        let sample_rate = config.sample_rate().0 as f64;
        let frequency = 220.0;
        
        let envelope = Arc::new(Mutex::new(Envelope::new(
            0.5,  // 500ms attack
            0.05, // 50ms decay
            0.8,  // 80% sustain
            2.0   // 1s release
        )));
        
        let mut saw_wave = signal::rate(sample_rate)
            .const_hz(frequency)
            .saw();
        
        let envelope_clone = envelope.clone();
        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut env = envelope_clone.lock().unwrap();
                for sample in data.iter_mut() {
                    let amplitude = env.get_amplitude();
                    *sample = saw_wave.next() as f32 * amplitude;
                }
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None
        ).expect("failed to build output stream");

        // Start the envelope right before playing
        envelope.lock().unwrap().start();
        stream.play().expect("failed to play stream");
        
        std::thread::sleep(Duration::from_secs(2));
        envelope.lock().unwrap().release();
        std::thread::sleep(Duration::from_secs(3));
    }
}