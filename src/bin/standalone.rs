use nih_plug::wrapper::standalone::nih_export_standalone;

// Replace `rust_vst_synth` with your crate name if different.
fn main() {
    nih_export_standalone::<rust_vst_synth::MySynth>();
}