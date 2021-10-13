use libm;

// Bindgen won't allow the reference to core::f32::consts.
// This constant is directly directly from the Rust source code,
// so it should be safe to squelch Clippy.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
pub const PI: f32 = 3.14159265358979323846264338327950288f32;
pub const TWO_PI: f32 = 2.0 * PI;

#[cfg(feature = "lowmem")]
pub const MAX_BLOCK_SIZE: usize = 64;
#[cfg(not(feature = "lowmem"))]
pub const MAX_BLOCK_SIZE: usize = 128;

#[cfg(feature = "lowmem")]
pub const MAX_CHANNEL_COUNT: usize = 2;
#[cfg(not(feature = "lowmem"))]
pub const MAX_CHANNEL_COUNT: usize = 8;

// TODO: All the types in this struct are a bit weird
// for a public API. Consider having a more sensible
// public API and then transforming to a new struct
// with runtime-appropriate types.
#[repr(C)]
pub struct AudioSettings {
    pub sample_rate: f32,
    pub block_size: usize,
    pub num_channels: usize
}

#[derive(Debug)]
#[repr(C)]
pub struct MonoBuffer {
    pub samples: [f32; MAX_BLOCK_SIZE]
}

impl MonoBuffer {
    pub fn new_with_value(value: f32) -> MonoBuffer {
        MonoBuffer {
            samples: [value; MAX_BLOCK_SIZE]
        }
    }

    pub fn new_silent() -> MonoBuffer {
        MonoBuffer::new_with_value(0.0)
    }
}

#[no_mangle]
pub extern "C" fn MonoBuffer_new_with_value(value: f32) -> MonoBuffer {
    MonoBuffer::new_with_value(value)
}

#[no_mangle]
pub extern "C" fn MonoBuffer_new_silent() -> MonoBuffer {
    MonoBuffer::new_silent()
}

#[derive(Debug)]
#[repr(C)]
pub struct MultichannelBuffer {
    pub channels: [[f32; MAX_BLOCK_SIZE]; MAX_CHANNEL_COUNT]
}

impl MultichannelBuffer {
    pub fn new_with_value(value: f32) -> MultichannelBuffer {
        MultichannelBuffer {
            channels: [[value; MAX_BLOCK_SIZE]; MAX_CHANNEL_COUNT]
        }
    }

    pub fn new_silent() -> MultichannelBuffer {
        MultichannelBuffer::new_with_value(0.0)
    }
}

#[no_mangle]
pub extern "C" fn MultichannelBuffer_new_with_value(value: f32) -> MultichannelBuffer {
    MultichannelBuffer::new_with_value(value)
}

#[no_mangle]
pub extern "C" fn MultichannelBuffer_new_silent() -> MultichannelBuffer {
    MultichannelBuffer::new_silent()
}

pub trait Signal {
    fn generate(&mut self);
}

#[repr(C)]
pub struct Connection<'a> {
    pub buffer: &'a MonoBuffer,
    pub step_size: usize
}

#[repr(C)]
pub struct ValueParameters {
    pub value: f32
}

#[repr(C)]
pub struct Value {
    pub settings: AudioSettings,
    pub parameters: ValueParameters,
    pub output: MonoBuffer,
    pub last_sample: f32
}

impl Value {
    pub fn new(settings: AudioSettings) -> Value {
        Value {
            settings,
            parameters: ValueParameters {
                value: 0.0
            },
            output: MonoBuffer::new_silent(),
            last_sample: 0.0
        }
    }
}

impl Signal for Value {
    fn generate(&mut self) {
        // If we've already generated buffers containing this value,
        // don't bother with the main loop.
        #[allow(clippy::float_cmp)]
        if self.parameters.value == self.last_sample {
            return
        }

        for i in 0..self.settings.block_size {
            self.output.samples[i] = self.parameters.value;
        }
    }
}

#[no_mangle]
pub extern "C" fn Value_new(settings: AudioSettings) -> Value {
    Value::new(settings)
}

#[no_mangle]
pub extern "C" fn Value_generate(value: &mut Value) {
    value.generate()
}

// TODO: Express these as Connections.
#[repr(C)]
pub struct SineInputs {
    pub freq: MonoBuffer,
    pub phase_offset: MonoBuffer,
    pub mul: MonoBuffer,
    pub add: MonoBuffer
}

#[repr(C)]
pub struct Sine {
    pub settings: AudioSettings,
    pub inputs: SineInputs,
    pub output: MonoBuffer,
    pub phase_accumulator: f32
}

impl Sine {
    pub fn new(settings: AudioSettings) -> Sine {
        Sine {
            settings,

            // TODO: Remove hardcoding, introduce Connections,
            // bind to Value signals, implement default merging.
            inputs: SineInputs {
                freq: MonoBuffer::new_with_value(440.0),
                phase_offset: MonoBuffer::new_with_value(0.0),
                mul: MonoBuffer::new_with_value(1.0),
                add: MonoBuffer::new_with_value(0.0)
            },
            output: MonoBuffer::new_silent(),
            phase_accumulator: 0.0
        }
    }
}

impl Signal for Sine {
    fn generate(&mut self) {
        // TODO: Write a macro to handle to the core loop boilerplate
        // and scaling/offset logic.

        for i in 0..self.settings.block_size {
            // TODO: Do negative values need to be handled?
            let modulated_phase = (self.phase_accumulator +
                self.inputs.phase_offset.samples[i]) % TWO_PI;

            self.output.samples[i] = libm::sinf(modulated_phase) *
                self.inputs.mul.samples[i] +
                self.inputs.add.samples[i];

            let phase_step = self.inputs.freq.samples[i] /
                self.settings.sample_rate * TWO_PI;

            self.phase_accumulator += phase_step;
            if self.phase_accumulator > TWO_PI {
                self.phase_accumulator -= TWO_PI;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn Sine_new(settings: AudioSettings) -> Sine {
    Sine::new(settings)
}

#[no_mangle]
pub extern "C" fn Sine_generate(sine: &mut Sine) {
    sine.generate()
}


#[repr(C)]
pub struct FanInputs {
    pub source: MonoBuffer
}

#[repr(C)]
pub struct Fan {
    pub settings: AudioSettings,
    pub inputs: FanInputs,
    pub output: MultichannelBuffer
}

impl Signal for Fan {
    fn generate(&mut self) {
        for i in 0..self.settings.num_channels {
            let mut channel = self.output.channels[i];
            channel[0..self.settings.block_size].clone_from_slice(
                &self.inputs.source.samples[0..self.settings.block_size]);
        }
    }
}

#[no_mangle]
pub extern "C" fn Fan_generate(fan: &mut Fan) {
    fan.generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_f32_eq_with_error(expected: f32, actual: f32, error_margin: f32) {
        let actual_error = (expected - actual).abs();
        assert_eq!(true, (expected - actual).abs() <= error_margin,
            "f32 value did not match. \nExpected: {:.26}, actual: {:.26}. \
            Error: {:.26}",
                expected, actual, actual_error);
    }

    fn assert_f32_buffer_eq(expected: [f32;MAX_BLOCK_SIZE], actual: [f32;MAX_BLOCK_SIZE], block_size: usize) {
        for i in 0..block_size {
            // https://rust-lang.github.io/rust-clippy/master/#float_cmp
            let error_margin = f32::EPSILON;
            let actual_error = (expected[i] - actual[i]).abs();

            assert_eq!(true, (expected[i] - actual[i]).abs() <= error_margin,
                "Sample {} did not match. Expected: {:.26}, actual: {:.26}. \
                Error: {:.26}",
                i, expected[i], actual[i], actual_error);
        }
    }

    #[test]
    fn multichannel_buffer_contains_value() {
        let actual = MultichannelBuffer_new_with_value(1.0);
        assert_eq!(MAX_CHANNEL_COUNT, actual.channels.len());
        for channel in actual.channels {
            assert_eq!(MAX_BLOCK_SIZE, channel.len());
            for sample in channel {
                assert_eq!(1.0, sample);
            }
        }
    }

    #[test]
    fn value_is_output() {
        let mut value_signal = Value_new(AudioSettings {
            sample_rate: 44100.0,
            block_size: 64,
            num_channels: 1
        });
        value_signal.parameters.value = 1.0;

        Value_generate(&mut value_signal);

        let mut expected = [0.0; MAX_BLOCK_SIZE];
        for i in 0..64 {
            expected[i] = 1.0;
        }

        let actual = value_signal.output.samples;

        assert!(
            actual.iter().zip(expected.iter()).all(|(a, b)| a == b),
            "Output does not contain the value. {:?}", actual
        );
    }

    #[test]
    fn sin_is_output() {
        let expected: [f32;MAX_BLOCK_SIZE] = [
            0.0,0.06264832615852355957031250,0.12505052983760833740234375,0.18696144223213195800781250,0.24813786149024963378906250,0.30833938717842102050781250,0.36732956767082214355468750,0.42487663030624389648437500,0.48075449466705322265625000,0.53474360704421997070312500,0.58663195371627807617187500,0.63621556758880615234375000,0.68329972028732299804687500,0.72769939899444580078125000,0.76924020051956176757812500,0.80775886774063110351562500,0.84310418367385864257812500,0.87513720989227294921875000,0.90373212099075317382812500,0.92877656221389770507812500,0.95017212629318237304687500,0.96783477067947387695312500,0.98169511556625366210937500,0.99169868230819702148437500,0.99780619144439697265625000,0.99999368190765380859375000,0.99825245141983032226562500,0.99258947372436523437500000,0.98302686214447021484375000,0.96960228681564331054687500,0.95236849784851074218750000,0.93139308691024780273437500,0.90675866603851318359375000,0.87856185436248779296875000,0.84691345691680908203125000,0.81193780899047851562500000,0.77377235889434814453125000,0.73256701231002807617187500,0.68848365545272827148437500,0.64169549942016601562500000,0.59238636493682861328125000,0.54074990749359130859375000,0.48698899149894714355468750,0.43131488561630249023437500,0.37394630908966064453125000,0.31510862708091735839843750,0.25503295660018920898437500,0.19395537674427032470703125,0.13211579620838165283203125,0.06975717842578887939453125,0.00712451478466391563415527,-0.05553614348173141479492188,-0.11797861754894256591796875,-0.17995759844779968261718750,-0.24122957885265350341796875,-0.30155384540557861328125000,-0.36069342494010925292968750,-0.41841593384742736816406250,-0.47449466586112976074218750,-0.52870923280715942382812500,-0.58084672689437866210937500,-0.63070219755172729492187500,-0.67807990312576293945312500,-0.72279369831085205078125000, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        ];

        let mut sine_signal = Sine_new(AudioSettings {
            sample_rate: 44100.0,
            block_size: 64,
            num_channels: 1
        });
        Sine_generate(&mut sine_signal);

        assert_f32_buffer_eq(
            expected,
            sine_signal.output.samples,
            sine_signal.settings.block_size
        );
    }

    #[test]
    fn sin_is_offset() {
        let expected: [f32;MAX_BLOCK_SIZE] = [
            1.0,1.06264829635620117187500000,1.12505054473876953125000000,1.18696141242980957031250000,1.24813783168792724609375000,1.30833935737609863281250000,1.36732959747314453125000000,1.42487668991088867187500000,1.48075449466705322265625000,1.53474354743957519531250000,1.58663201332092285156250000,1.63621556758880615234375000,1.68329977989196777343750000,1.72769939899444580078125000,1.76924014091491699218750000,1.80775880813598632812500000,1.84310412406921386718750000,1.87513720989227294921875000,1.90373206138610839843750000,1.92877650260925292968750000,1.95017218589782714843750000,1.96783471107482910156250000,1.98169517517089843750000000,1.99169874191284179687500000,1.99780619144439697265625000,1.99999368190765380859375000,1.99825239181518554687500000,1.99258947372436523437500000,1.98302686214447021484375000,1.96960234642028808593750000,1.95236849784851074218750000,1.93139314651489257812500000,1.90675866603851318359375000,1.87856185436248779296875000,1.84691345691680908203125000,1.81193780899047851562500000,1.77377235889434814453125000,1.73256707191467285156250000,1.68848371505737304687500000,1.64169549942016601562500000,1.59238636493682861328125000,1.54074990749359130859375000,1.48698902130126953125000000,1.43131494522094726562500000,1.37394630908966064453125000,1.31510865688323974609375000,1.25503301620483398437500000,1.19395542144775390625000000,1.13211584091186523437500000,1.06975722312927246093750000,1.00712454319000244140625000,0.94446384906768798828125000,0.88202136754989624023437500,0.82004237174987792968750000,0.75877040624618530273437500,0.69844615459442138671875000,0.63930654525756835937500000,0.58158409595489501953125000,0.52550530433654785156250000,0.47129076719284057617187500,0.41915327310562133789062500,0.36929780244827270507812500,0.32192009687423706054687500,0.27720630168914794921875000, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        ];

        let mut sine_signal = Sine_new(AudioSettings {
            sample_rate: 44100.0,
            block_size: 64,
            num_channels: 1
        });
        sine_signal.inputs.add = MonoBuffer::new_with_value(1.0);

        Sine_generate(&mut sine_signal);

        assert_f32_buffer_eq(
            expected,
            sine_signal.output.samples,
            sine_signal.settings.block_size
        );
    }

    #[test]
    fn sin_accumulates_phase() {
        let mut sine_signal = Sine_new(AudioSettings {
            sample_rate: 48000.0,
            block_size: 48,
            num_channels: 1
        });

        let phase_step = 0.05759586393833160400390625_f32;

        sine_signal.generate();
        assert_f32_eq_with_error(
            phase_step * 48.0,
            sine_signal.phase_accumulator,
            0.000001);

        sine_signal.generate();
        assert_f32_eq_with_error(
            phase_step * 96.0,
            sine_signal.phase_accumulator,
            0.000001);
    }

    #[test]
    fn sin_limits_phase_to_twopi() {
        let mut sine_signal = Sine_new(AudioSettings {
            sample_rate: 48000.0,
            block_size: 48,
            num_channels: 1
        });

        sine_signal.generate();
        sine_signal.generate();
        sine_signal.generate();

        assert!(sine_signal.phase_accumulator <= TWO_PI &&
            sine_signal.phase_accumulator >= 0.0);
    }
}
