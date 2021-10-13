#include "../include/libflock.h"

float samples[flock_BLOCK_SIZE];

float * flock_generate_silence(void) {
    for (int i = 0; i < flock_BLOCK_SIZE; i++) {
        samples[i] = 0.0;
    }

    return samples;
}
