# Daber 1.0: an app for people who just want to talk

Handy is a power tool for people who choose models. Daber is for an Israeli who
wants their speech to become Hebrew text. Everything below follows from that one
difference.

Cross-platform from the start: the release workflow inherited from upstream
already builds macOS (Intel + ARM), Windows (x64 + ARM64) and Linux, so every
decision here has to hold on a Windows laptop with no usable GPU, not only on an
M4 with Metal.

## Decisions

**The model ships inside the app, quantized.** The full-precision model is
1,625 MB, which is too much to bundle and too much to download on first run.
Quantization was measured rather than assumed — see `docs/` for the numbers.
5-bit formats were ruled out despite their size: they pay a heavy unpacking cost
on CPU, which is exactly the machine that can least afford it.

**No model picker anywhere.** One model, chosen for Hebrew. The picker, the
catalog UI and the "show all models" list all go. What stays is a single repair
affordance in Advanced, for when the file on disk is damaged.

**Onboarding ends with a successful dictation.** The current flow ends with a
download bar, which teaches nothing. The new one ends with the user having said
something and watched it become text. That is the moment the app makes sense,
and it doubles as an end-to-end check of microphone, permissions, model and
paste before they try it in a real app.

## Plan

### 1. Model delivery

- [x] Quantize the ivrit.ai model and publish it (Apache-2.0 permits this, with attribution)
- [x] Host it where a blocked huggingface.co cannot break setup, pinned by SHA-256
- [x] Download it in the background from app start rather than at a screen of its own
- [x] Keep the download path alive purely as a repair route

Bundling into the installer was the original plan and was dropped once the
measurement came in: the quantization that keeps full accuracy is 874 MB, which
makes for a ~890 MB installer that everyone pays before they know whether they
want the app. Overlapping the download with permissions and shortcut setup costs
the user nothing, because that setup takes about as long. The resource-bundling
path stays supported for an offline installer variant.

### 2. Onboarding

- [x] Turn the flow into a platform-aware step list rather than two hardcoded screens
- [x] Permissions step: unchanged behaviour, skipped where the platform has none
- [x] Hotkey step: pick the key, with the platform default pre-filled
- [x] Try-it step: a box that fills with what the user says
- [x] Model step appears only when the model is genuinely missing

### 3. Strip what nobody needs

- [x] Remove the Models section from the sidebar and delete the picker components
- [x] Remove the model card from the main screen
- [x] Move channel, mute-while-recording, output device and volume into Advanced
- [x] Main screen keeps: hotkey, push-to-talk, microphone, audio feedback

### 4. Prove it

- [x] Hebrew transcription still matches the reference on real recordings
- [x] Fresh-profile first run on macOS: download, verify, transcribe
- [ ] Windows build green in CI

## Review

**The quantization is free.** q8_0 output is byte-identical to the 1,625 MB
original on all three of Daniel's own microphone recordings, run through the app
itself rather than a benchmark harness, on Metal at 1.1-2.5x real time. 874 MB
instead of 1,625 MB for no measurable loss.

**5-bit formats were the trap.** On this M4 they looked best of all — fastest on
Metal, output matching on seven of eight clips. They are also 3-5x slower on a
CPU without native fp16, which is the Windows laptop this app has to be good on
and the one machine that cannot absorb it. Measuring only on the development
machine would have shipped that.

**Two bugs the build did not catch,** both found by rereading the diff:

- The download was owned by the onboarding component, so someone who skipped the
  last step mid-download ended up with the model on disk and never selected.
  It moved to `App`, where it outlives onboarding.
- Selecting the model on every launch would have re-loaded 874 MB onto the GPU
  each time. Now guarded on `currentModel`.

**Left undone on purpose.** The 22 locales other than English and Hebrew carry
the English text for the new strings — the same thing i18next would have shown
through its fallback, and a translator's job rather than a guess. Windows and
Linux are built by CI but were not run by hand; nothing in the change is
platform-specific beyond the step list, which is derived from `platform()`.
