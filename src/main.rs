use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use rust_vst_synth::envelope::Envelope;
use rust_vst_synth::filter::{Filter, FilterParameters, FilterSlope, FilterType};
use rust_vst_synth::oscillator::OscillatorConfig;
use rust_vst_synth::synthesizer::{Synthesizer, SynthesizerConfig};
use rust_vst_synth::voice_configuration::Waveform;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = 44100.0;
    // Create the envelope configuration
    let envelope = Envelope::new(
        1.5,     // attack time in seconds
        1.0,     // decay time
        0.6,     // sustain level (amplitude)
        2.0,     // release time
        sample_rate  // initial sample rate
    );

    // Wrap it in Arc<Mutex>
    let envelope = Arc::new(Mutex::new(envelope));

    let filter_config = FilterParameters {
        filter_type: FilterType::LowPass,
        slope: FilterSlope::Slope24dB,
        cutoff: 500.0,  // 2kHz initial cutoff
        resonance: 0.8,  // Moderate resonance
        mod_amount: 0.6,
    };


    // Create oscillator configurations
    let oscillator_configs = vec![
        OscillatorConfig {
            waveform: Waveform::SQUARE,
            detune_semitones: 0.0,
            volume: 1.0,
        },
        OscillatorConfig {
            waveform: Waveform::SAW,
            detune_semitones: 7.0,
            volume: 0.6,
        },
        OscillatorConfig {
            waveform: Waveform::SQUARE,
            detune_semitones: -12.0,
            volume: 0.5,
        }
    ];

    let filter = Filter::new(filter_config, sample_rate);

    // Create synthesizer configuration
    let config = SynthesizerConfig {
        oscillator_configs,
        envelope: envelope.clone(),
        filter,
        filter_envelope: envelope.clone(),
        max_voices: 16,
    };

    // Create and start the synthesizer
    let mut synth = Synthesizer::new(config);
    synth.start_audio()?;

    // Play a test note (A4 = 440 Hz)
    println!("Playing test note...");
    synth.note_on(110.0);

    // sleep(Duration::from_millis(500));

    // synth.note_on(165.0);

    // sleep(Duration::from_millis(500));

    // synth.note_on(220.0);
    
    // Keep the note playing for 2 seconds
    sleep(Duration::from_secs(2));
    
    // Release the note
    synth.note_off(110.0);
    
    // Wait for release to complete
    sleep(Duration::from_secs(20));

    // Keep the program running
    println!("Press Ctrl+C to exit...");
    loop {
        sleep(Duration::from_secs(10));
    }
}