#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

static const uintptr_t MAX_BLOCK_SIZE = 128;

struct AudioSettings {
  uintptr_t block_size;
};

struct ValueParameters {
  float value;
};

struct MultichannelBuffer {
  float channels[1][MAX_BLOCK_SIZE];
};

struct Value {
  AudioSettings audio_settings;
  ValueParameters parameters;
  MultichannelBuffer input;
  MultichannelBuffer output;
};

extern "C" {

void generate(Value *state);

} // extern "C"
