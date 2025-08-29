pub mod voice_configuration;
pub mod envelope;
pub mod oscillator;
pub mod synthesizer;
pub mod filter;

mod voice;

use nih_plug::prelude::*;
use std::sync::Arc;
use nih_plug_vizia::ViziaState;
use nih_plug_vizia::ViziaTheming;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::ParamSlider;

pub struct MySynth {
    params: Arc<MyParams>,
    vizia_state: Arc<ViziaState>,
}

impl Default for MySynth {
    fn default() -> Self {
        Self {
            params: Arc::new(MyParams::default()),
            vizia_state: ViziaState::new(|| (520, 360)),
        }
    }
}

#[derive(Params)]
pub struct MyParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for MyParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Plugin for MySynth {
    const NAME: &'static str = "My Rust Synth";
    const VENDOR: &'static str = "Your Name";
    const URL: &'static str = "https://example.com";
    const EMAIL: &'static str = "dev@example.com";
    const VERSION: &'static str = "0.0.1";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: Some(NonZeroU32::new(2).unwrap()),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        nih_plug_vizia::create_vizia_editor(
            self.vizia_state.clone(),
            ViziaTheming::Custom, // or Builtin if you prefer (no font reg needed)
            move |cx, _| {
                // Only needed for Custom theming:
                nih_plug_vizia::assets::register_noto_sans_light(cx);

                // 1) Build the model into the VIZIA context
                ParamsModel { params: params.clone() }.build(cx);

                // 2) Build your UI using lenses into that model
                VStack::new(cx, |cx| {
                    Label::new(cx, "My Rust Synth").hoverable(false);

                    // 3) Pass a LENS (ParamsModel::params), not Arc<MyParams>
                    ParamSlider::new(cx, ParamsModel::params, |p: &Arc<MyParams>| &p.gain)
                        .height(Pixels(50.0));
                })
                    .space(Pixels(4.0));

                ResizeHandle::new(cx);
            },
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let gain = self.params.gain.value();

        for channel_samples in buffer.as_slice() {
            for sample in channel_samples.iter_mut() {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }
}

#[derive(Lens)]
struct ParamsModel {
    params: Arc<MyParams>,
}
impl Model for ParamsModel {}