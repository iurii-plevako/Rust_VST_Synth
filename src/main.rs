use std::sync::{Arc, Mutex};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use midir::{MidiInput, MidiInputConnection};
use rust_vst_synth::envelope::{Envelope, EnvelopeConfig};
use rust_vst_synth::filter::{Filter, FilterParameters, FilterSlope, FilterType};
use rust_vst_synth::oscillator::OscillatorConfig;
use rust_vst_synth::synthesizer::{Synthesizer, SynthesizerConfig};
use rust_vst_synth::voice_configuration::Waveform;

fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let sample_rate = 44100.0;

    let envelope_config = EnvelopeConfig::new(
        0.5,    // attack time
        0.5,    // decay time
        0.7,    // sustain level
        3.0,    // release time
        false,  // retrigger off - will continue from current value
    );

    let filter_envelope_config = EnvelopeConfig::new(
        0.3,    // attack time
        0.2,    // decay time
        0.7,    // sustain level
        3.0,    // release time
        false,   // retrigger on - will start from beginning
    );

    let filter_config = FilterParameters {
        filter_type: FilterType::LowPass,
        slope: FilterSlope::Slope24dB,
        cutoff_frequency: 2000.0,
        resonance_amount: 0.8,
        modulation_amount: 0.6,
    };

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
        // OscillatorConfig {
        //     waveform: Waveform::SQUARE,
        //     detune_semitones: -12.0,
        //     volume: 0.5,
        // },
        // OscillatorConfig {
        //     waveform: Waveform::WHITE_NOISE,
        //     detune_semitones: 0.0,
        //     volume: 0.2,
        // }
    ];

    let filter = Filter::new(filter_config, sample_rate);

    let config = SynthesizerConfig {
        oscillator_configs,
        envelope_config,
        filter,
        filter_envelope_config,
        max_voices: 16,
        sample_rate,
    };


    // Create and start the synthesizer
    let synth = Arc::new(Mutex::new(Synthesizer::new(config)));
    synth.lock().unwrap().start_audio()?;

    // Initialize MIDI
    let midi_in = MidiInput::new("rust-synth-input")?;
    
    // Get available MIDI input ports
    let ports = midi_in.ports();
    let in_ports_len = ports.len();

    // No MIDI inputs available
    if in_ports_len == 0 {
        println!("No MIDI input ports available");
        return Ok(());
    }

    println!("\nAvailable input ports:");
    for (i, p) in ports.iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p)?);
    }

    print!("Please select input port: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let port_number = input.trim().parse::<usize>()?.min(in_ports_len - 1);

    let synth_clone = synth.clone();
    
    // Create MIDI connection and handle incoming messages
    let _conn = midi_in.connect(
        &ports[port_number],
        "midi-read",
        move |_stamp, message, _| {
            let command = message[0] & 0xF0;
            let note = message[1];
            let velocity = message[2];

            match command {
                0x90 if velocity > 0 => {
                    // Note On
                    let freq = midi_note_to_freq(note);
                    if let Ok(mut synth) = synth_clone.lock() {
                        synth.note_on(freq);
                    }
                },
                0x80 | 0x90 => {
                    // Note Off (0x80 or 0x90 with velocity 0)
                    let freq = midi_note_to_freq(note);
                    if let Ok(mut synth) = synth_clone.lock() {
                        synth.note_off(freq);
                    }
                },
                _ => (),
            }
        },
        (),
    )?;

    println!("\nReading MIDI input... Press Enter to exit.");
    input.clear();
    stdin().read_line(&mut input)?;

    Ok(())
}