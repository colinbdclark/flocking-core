#include <stdio.h>
#include <stdlib.h>
#include "libflock.h"

void print_sample(float sample) {
    printf("%.2f", sample);
}

int main(int argc, char *argv[]) {
    float* samples = flock_generate_silence();
    printf("Enjoy the ");

    for (int i = 0; i < flock_BLOCK_SIZE - 1; i++) {
        print_sample(samples[i]);
        printf(",");
    }

    print_sample(samples[flock_BLOCK_SIZE - 1]);
    printf("\n");

    return EXIT_SUCCESS;
}
