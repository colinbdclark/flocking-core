pub const MAX_BLOCK_SIZE: usize = 128;

#[repr(C)]
pub struct AudioSettings {
    block_size: usize
}

#[repr(C)]
pub struct MultichannelBuffer {
    // TODO: Parameterize block size.
    // TODO: Parameterize channel count
    pub channels: [[f32; MAX_BLOCK_SIZE]; 1]
}

#[repr(C)]
pub struct ValueParameters {
    pub value: f32
}

#[repr(C)]
pub struct Value {
    pub audio_settings: AudioSettings,
    pub parameters: ValueParameters,
    pub input: MultichannelBuffer,
    pub output: MultichannelBuffer
}

impl Value {
    #[no_mangle]
    pub extern "C" fn generate(state: &mut Value) {
        for i in 0..state.output.channels.len() {
            let channel = &mut state.output.channels[i];
            for j in 0..state.audio_settings.block_size {
                channel[j] = state.parameters.value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_is_output() {
        let mut value_signal = Value {
            audio_settings: AudioSettings {
                block_size: 64
            },

            parameters: ValueParameters {
                value: 1.0
            },
            input: MultichannelBuffer {
                channels: [[0.0; MAX_BLOCK_SIZE]]
            },
            output: MultichannelBuffer {
                channels: [[0.0; MAX_BLOCK_SIZE]]
            }
        };

        Value::generate(&mut value_signal);

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
}
