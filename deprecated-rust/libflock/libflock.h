#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

static const float PI = 3.14159265358979323846264338327950288;

static const float TWO_PI = (2.0 * PI);

#if defined(LOWMEM)
static const uintptr_t MAX_BLOCK_SIZE = 64;
#endif

#if !defined(LOWMEM)
static const uintptr_t MAX_BLOCK_SIZE = 128;
#endif

#if defined(LOWMEM)
static const uintptr_t MAX_CHANNEL_COUNT = 2;
#endif

#if !defined(LOWMEM)
static const uintptr_t MAX_CHANNEL_COUNT = 8;
#endif

struct MonoBuffer {
  float samples[MAX_BLOCK_SIZE];
};

struct MultichannelBuffer {
  float channels[MAX_CHANNEL_COUNT][MAX_BLOCK_SIZE];
};

struct AudioSettings {
  float sample_rate;
  uintptr_t block_size;
  uintptr_t num_channels;
};

struct ValueParameters {
  float value;
};

struct Value {
  AudioSettings settings;
  ValueParameters parameters;
  MonoBuffer output;
  float last_sample;
};

struct SineInputs {
  MonoBuffer freq;
  MonoBuffer phase_offset;
  MonoBuffer mul;
  MonoBuffer add;
};

struct Sine {
  AudioSettings settings;
  SineInputs inputs;
  MonoBuffer output;
  float phase_accumulator;
};

struct FanInputs {
  MonoBuffer source;
};

struct Fan {
  AudioSettings settings;
  FanInputs inputs;
  MultichannelBuffer output;
};

extern "C" {

MonoBuffer MonoBuffer_new_with_value(float value);

MonoBuffer MonoBuffer_new_silent();

MultichannelBuffer MultichannelBuffer_new_with_value(float value);

MultichannelBuffer MultichannelBuffer_new_silent();

Value Value_new(AudioSettings settings);

void Value_generate(Value *value);

Sine Sine_new(AudioSettings settings);

void Sine_generate(Sine *sine);

void Fan_generate(Fan *fan);

} // extern "C"
