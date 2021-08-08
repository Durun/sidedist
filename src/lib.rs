#[macro_use]
extern crate vst;

use std::sync::Arc;

use vst::api::Supported;
use vst::buffer::AudioBuffer;
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

#[derive(Default)]
struct Sidedist {
    params: Arc<SidedistParameters>,
}

struct SidedistParameters {
    threshold: AtomicFloat,
    sidechain_gain: AtomicFloat,
}
const SIDECHAIN_GAIN_DEFALUT: f32 = 0.2f32;

impl Default for SidedistParameters {
    fn default() -> Self {
        SidedistParameters {
            threshold: AtomicFloat::new(1.0),
            sidechain_gain: AtomicFloat::new(SIDECHAIN_GAIN_DEFALUT),
        }
    }
}

impl Plugin for Sidedist {
    fn get_info(&self) -> Info {
        Info {
            name: "SideDist".to_string(),
            category: Category::Effect,
            unique_id: 20210808,

            inputs: 4,
            outputs: 2,

            parameters: 2,

            ..Default::default()
        }
    }
    fn new(_host: HostCallback) -> Self {
        Sidedist {
            params: Arc::new(SidedistParameters::default()),
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::No,
            _ => Supported::Maybe,
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // Read the amplitude from the parameter object
        let threshold = self.params.threshold.get();
        let sidechain_amplitude = self.params.sidechain_gain.get() / SIDECHAIN_GAIN_DEFALUT;
        // First, we destructure our audio buffer into an arbitrary number of
        // input and output buffers.  Usually, we'll be dealing with stereo (2 of each)
        // but that might change.
        let (input_buffers, mut output_buffers) = buffer.split();

        // Next, we'll loop through each individual sample so we can apply the amplitude
        // value to it.
        for ((input, sidechain), output) in input_buffers.into_iter().take(2)
            .zip(input_buffers.into_iter().skip(2))
            .zip(output_buffers.into_iter())
        {
            let input: &[f32] = input;
            let sidechain: &[f32] = sidechain;
            let output: &mut [f32] = output;
            for ((input_sample, sidechain_sample), output_sample) in input.iter().zip(sidechain).zip(output) {
                let limit_upper = max(0f32, threshold - *sidechain_sample * sidechain_amplitude);
                let limit_lower = min(0f32, -threshold - *sidechain_sample * sidechain_amplitude);
                *output_sample = clip(*input_sample, limit_upper, limit_lower);
            }
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for SidedistParameters {
    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", self.threshold.get()),
            1 => format!("{:.2}", self.sidechain_gain.get() / SIDECHAIN_GAIN_DEFALUT),
            _ => "".to_string(),
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Threshold",
            1 => "Sidechain Gain",
            _ => "",
        }
            .to_string()
    }

    // the `get_parameter` function reads the value of a parameter.
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.threshold.get(),
            1 => self.sidechain_gain.get(),
            _ => 0.0,
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        match index {
            0 => self.threshold.set(val),
            1 => self.sidechain_gain.set(val),
            _ => (),
        }
    }
}

plugin_main!(Sidedist);


fn clip(signal: f32, limit_upper: f32, limit_lower: f32) -> f32 {
    if limit_upper < signal {
        limit_upper
    } else if signal < limit_lower {
        limit_lower
    } else {
        signal
    }
}

fn max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}

fn min(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}