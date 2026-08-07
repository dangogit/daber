# One model, and an onboarding that ends in a sentence

_2026-08-07. Supersedes the model-delivery part of
[2026-08-06-hebrew-by-default-design.md](2026-08-06-hebrew-by-default-design.md),
which described the full-precision 1.6 GB download and a first run that still
showed a model list._

## The difference that drives everything

Handy is a tool for people who choose models. Dibur is for an Israeli who wants
their speech to become Hebrew text. There is one engine, so every place the app
asked about that had to go.

It also has to hold on Windows, not only on the Mac it was developed on. That
turned out to matter more than expected.

## What was measured

Six quantizations of the ivrit.ai model, over eight Hebrew clips — three of them
real microphone recordings with known-good reference transcripts from the
full-precision model.

|          | size       | Metal | CPU    | vs f16   |
| -------- | ---------- | ----- | ------ | -------- |
| f16      | 1,625 MB   | 29.5s | 83.5s  | baseline |
| **q8_0** | **874 MB** | 30.5s | 208.8s | **8/8**  |
| q5_1     | 624 MB     | 31.3s | 151.0s | 7/8      |
| q5_0     | 574 MB     | 26.1s | 118.1s | 7/8      |
| q4_1     | 524 MB     | 30.5s | 151.5s | 6/8      |
| q4_0     | 474 MB     | 25.6s | 229.6s | 5/8      |

Two results changed the decision.

**On CPU, f16 is the fastest of all of them.** Apple Silicon does fp16 in
hardware, so there is nothing to unpack; every quantization pays a cost that the
GPU hides. This is an ARM result and does not carry to x86, which is exactly why
it is dangerous — measuring only on the development machine would have made
5-bit look like the obvious winner.

**Even the fastest CPU run is 1.68x real time**, so ten seconds of speech takes
seventeen to transcribe. The CPU path is unusable whatever the format, which
means it cannot be what decides the format. Windows runs Vulkan and any Intel
integrated GPU from 2015 on supports it, so the overwhelming majority of machines
take the GPU path — where every option here is within 20% of every other.

So the choice comes down to accuracy and size, and q8_0 is the only one that
matched full precision everywhere. q4_0's failures are the kind that destroy
trust in a dictation app: `ועזד פלוי` for `ועזה דפלוי`, `ל' 14` for `ל-14`.

## Decisions

**q8_0, self-hosted, pinned by SHA-256.** 874 MB. Hosted on a GitHub release
rather than the Hub, because huggingface.co is blocked on a fair number of
Israeli school and workplace networks and a blocked download is
indistinguishable from a broken app. Apache-2.0 permits the redistribution.

**Downloaded in the background, not bundled.** Bundling was the plan until the
size landed at 874 MB, which makes an ~890 MB installer that everyone pays
before they know whether they want the app. The download instead starts when the
app opens and runs under the setup someone has to do anyway. It is owned by
`App` rather than by the onboarding component, so skipping the last step early
cannot strand a downloaded model that nothing ever selects.

**Onboarding ends with a successful dictation.** The old flow ended at a
progress bar, which teaches nothing. The last step is a focused textarea and an
instruction to press the shortcut and speak; the text arrives by exactly the
route it will in every other app. That makes it a real end-to-end check of
microphone, permissions, model and paste, at the one moment where failing is
cheap.

**Steps are derived from the platform.** macOS asks for microphone and
accessibility, Windows for microphone alone, Linux for neither — so Linux starts
at the shortcut rather than flashing an empty screen. This is why the shortcut is
its own step rather than living inside the permissions screen: on Linux that
screen does not exist, and the shortcut would have vanished with it.

**One model control survives.** A single row in Advanced re-downloads the
engine. A truncated download otherwise leaves someone with an app that fails
every time and no way to fix it short of finding the data directory by hand.
