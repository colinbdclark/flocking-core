use libm;

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

#[repr(C)]
pub struct AudioSettings {
    sample_rate: f32,
    block_size: usize,
    num_channels: usize
}

#[repr(C)]
pub struct MultichannelBuffer {
    // TODO: Parameterize channel count
    pub channels: [[f32; MAX_BLOCK_SIZE]; MAX_CHANNEL_COUNT]
}

#[no_mangle]
pub extern "C" fn MultichannelBuffer_new_with_value(value: f32) -> MultichannelBuffer {
    MultichannelBuffer {
        channels: [[value; MAX_BLOCK_SIZE]; MAX_CHANNEL_COUNT]
    }
}

#[no_mangle]
pub extern "C" fn MultichannelBuffer_new_silent() -> MultichannelBuffer {
    MultichannelBuffer_new_with_value(0.0)
}

pub trait Signal {
    fn generate(&mut self);
}

#[repr(C)]
pub struct Connection<'a> {
    pub buffer: &'a MultichannelBuffer,
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
    pub output: MultichannelBuffer,
    pub last_sample: f32
}

impl Signal for Value {
    fn generate(&mut self) {
        // If we've already generated buffers containing this value,
        // don't bother with the main loop.
        if self.parameters.value == self.last_sample {
            return
        }

        for i in 0..self.settings.num_channels {
            let channel = &mut self.output.channels[i];
            for j in 0..self.settings.block_size {
                channel[j] = self.parameters.value;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn Value_new(settings: AudioSettings) -> Value {
    Value {
        settings: settings,
        parameters: ValueParameters {
            value: 0.0
        },
        output: MultichannelBuffer_new_silent(),
        last_sample: 0.0
    }
}

#[no_mangle]
pub extern "C" fn Value_generate(value: &mut Value) {
    value.generate()
}

// TODO: Express these as Connections.
#[repr(C)]
pub struct SineInputs {
    pub freq: MultichannelBuffer,
    pub phase: MultichannelBuffer,
    pub mul: MultichannelBuffer,
    pub add: MultichannelBuffer
}

#[repr(C)]
pub struct Sine {
    pub settings: AudioSettings,
    pub inputs: SineInputs,
    pub output: MultichannelBuffer,
    pub phase_accumulator: f32
}

impl Signal for Sine {
    fn generate(&mut self) {
        // TODO: Write a macro to handle to the core loop boilerplate
        // and scaling/offset logic.
        for i in 0..self.settings.num_channels {
            let channel = &mut self.output.channels[i];
            for j in 0..self.settings.block_size {
                // TODO: Handle non-audio rate inputs.
                // TODO: Is this phase modulation actually correct?
                let sample = libm::sinf(self.phase_accumulator +
                    self.inputs.phase.channels[i][j]);
                let scaled = sample * self.inputs.mul.channels[i][j] +
                    self.inputs.add.channels[i][j];

                channel[j] = scaled;

                let phase_step = self.inputs.freq.channels[i][j] *
                    TWO_PI / self.settings.sample_rate;

                // TODO: This will overflow.
                // Reset it after we've done a full cycle? (>TWO_PI)
                self.phase_accumulator += phase_step;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn Sine_new(settings: AudioSettings) -> Sine{
    Sine {
        settings: settings,

        // TODO: Remove hardcoding, introduce Connections,
        // bind to Value signals, implement default merging.
        inputs: SineInputs {
            freq: MultichannelBuffer_new_with_value(440.0),
            phase: MultichannelBuffer_new_with_value(0.0),
            mul: MultichannelBuffer_new_with_value(1.0),
            add: MultichannelBuffer_new_with_value(0.0)
        },
        output: MultichannelBuffer_new_silent(),
        phase_accumulator: 0.0
    }
}

#[no_mangle]
pub extern "C" fn Sine_generate(sine: &mut Sine) {
    sine.generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output_is_equal_rounded(expected: [f32;MAX_BLOCK_SIZE], actual: [f32;MAX_BLOCK_SIZE], block_size: usize) {
        for i in 0..block_size {
            let rounded_actual = (actual[i] * 10000.0).round() / 10000.0;
            let rounded_expected = (expected[i] * 10000.0).round() / 10000.0;
            assert_eq!(rounded_expected, rounded_actual,
                "Sample {} did not match. Expected: {}, actual: {}",
                i, expected[i], actual[i]
            );
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

        let actual = value_signal.output.channels[0];

        assert!(
            actual.iter().zip(expected.iter()).all(|(a, b)| a == b),
            "Output does not contain the value. {:?}", actual
        );
    }

    #[test]
    fn sin_is_output() {
        let expected: [f32;MAX_BLOCK_SIZE] = [
            0.0,0.06264832615852356,0.12505052983760834,0.18696144223213196,0.24813784658908844,0.3083394169807434,0.36732959747314453,0.4248766601085663,0.480754554271698,0.5347436666488647,0.5866320133209229,0.6362156271934509,0.6832997798919678,0.7276994585990906,0.7692402601242065,0.8077589869499207,0.8431042432785034,0.8751372694969177,0.9037321209907532,0.9287765622138977,0.9501721262931824,0.9678347706794739,0.9816950559616089,0.991698682308197,0.997806191444397,0.9999936819076538,0.9982524514198303,0.9925894737243652,0.9830269813537598,0.9696024060249329,0.9523686766624451,0.9313933253288269,0.9067588448524475,0.8785620331764221,0.8469136357307434,0.8119379878044128,0.7737725377082825,0.7325671315193176,0.6884837746620178,0.6416955590248108,0.5923863053321838,0.5407497882843018,0.4869888126850128,0.4313146471977234,0.3739459812641144,0.31510820984840393,0.255032479763031,0.19395482540130615,0.1321151703596115,0.06975647062063217,0.007123732473701239,-0.05553699657320976,-0.11797953397035599,-0.17995858192443848,-0.24123062193393707,-0.30155494809150696,-0.36069455742836,-0.4184171259403229,-0.47449585795402527,-0.5287104845046997,-0.5808479189872742,-0.6307034492492676,-0.6780811548233032,-0.7227948904037476, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        ];

        let mut sine_signal = Sine_new(AudioSettings {
            sample_rate: 44100.0,
            block_size: 64,
            num_channels: 1
        });
        Sine_generate(&mut sine_signal);

        output_is_equal_rounded(
            expected,
            sine_signal.output.channels[0],
            sine_signal.settings.block_size
        );
    }

    #[test]
    fn sin_is_offset() {
        let expected: [f32;MAX_BLOCK_SIZE] = [
            1.0,1.0626482963562012,1.1250505447387695,1.1869614124298096,1.2481378316879272,1.3083393573760986,1.3673295974731445,1.4248766899108887,1.4807544946670532,1.5347436666488647,1.5866320133209229,1.6362156867980957,1.6832997798919678,1.7276995182037354,1.7692402601242065,1.8077589273452759,1.8431042432785034,1.875137209892273,1.903732180595398,1.928776502609253,1.9501720666885376,1.967834711074829,1.9816950559616089,1.9916986227035522,1.997806191444397,1.9999936819076538,1.998252511024475,1.9925894737243652,1.9830269813537598,1.9696024656295776,1.9523686170578003,1.9313933849334717,1.9067589044570923,1.878562092781067,1.8469136953353882,1.8119380474090576,1.7737724781036377,1.7325671911239624,1.688483715057373,1.641695499420166,1.592386245727539,1.5407497882843018,1.4869887828826904,1.4313147068023682,1.373945951461792,1.3151081800460815,1.2550325393676758,1.1939548254013062,1.132115125656128,1.0697565078735352,1.0071237087249756,0.9444630146026611,0.8820204734802246,0.8200414180755615,0.7587693929672241,0.6984450817108154,0.6393054723739624,0.5815829038619995,0.5255041122436523,0.4712895452976227,0.41915205121040344,0.3692965507507324,0.32191887497901917,0.27720513939857483, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        ];

        let mut sine_signal = Sine_new(AudioSettings {
            sample_rate: 44100.0,
            block_size: 64,
            num_channels: 1
        });
        sine_signal.inputs.add = MultichannelBuffer_new_with_value(1.0);

        Sine_generate(&mut sine_signal);

        output_is_equal_rounded(
            expected,
            sine_signal.output.channels[0],
            sine_signal.settings.block_size
        );
    }
}
