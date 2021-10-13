#include "../vendor/kxmx_bluemchen/src/kxmx_bluemchen.h"
#include "../../../../libflock/include/libflock.h"
#include <string>

using namespace kxmx;
using namespace daisy;

Bluemchen bluemchen;
Parameter knob1;
Parameter knob2;
Parameter cv1;
Parameter cv2;

void UpdateOled() {
    bluemchen.display.Fill(false);

    bluemchen.display.SetCursor(0, 0);
    std::string str = "Hello Euro";
    char *cstr = &str[0];
    bluemchen.display.WriteString(cstr, Font_6x8, true);

    bluemchen.display.Update();
}

void UpdateControls() {
    bluemchen.ProcessAllControls();
}

void AudioCallback(daisy::AudioHandle::InputBuffer in, daisy::AudioHandle::OutputBuffer out, size_t size) {
    UpdateControls();
    // Note, we're not taking into account block size here.
    float* samples = flock_generate_silence();

    for (size_t i = 0; i < size; i++) {
        out[0][i] = samples[i];
        out[1][i] = samples[i];
    }
}

int main(void) {
    bluemchen.Init();
    bluemchen.StartAdc();

    knob1.Init(bluemchen.controls[bluemchen.CTRL_1], 0.001f, 0.1f, Parameter::LINEAR);
    knob2.Init(bluemchen.controls[bluemchen.CTRL_2], 0.001f, 0.5f, Parameter::LINEAR);

    cv1.Init(bluemchen.controls[bluemchen.CTRL_3], -5000.0f, 5000.0f, Parameter::LINEAR);
    cv2.Init(bluemchen.controls[bluemchen.CTRL_4], -5000.0f, 5000.0f, Parameter::LINEAR);

    bluemchen.StartAudio(AudioCallback);

    while (1) {
        UpdateControls();
        UpdateOled();
    }
}
