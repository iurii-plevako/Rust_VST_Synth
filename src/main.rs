use rust_vst_synth::sound_module::SoundModule;

fn main() {
    // // Replace `MySynth` with your plugin type and make it `pub` so it’s constructible here
    // nih_export_standalone::<rust_vst_synth::MySynth>();

    SoundModule::new().play_note();
}