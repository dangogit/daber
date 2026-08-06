# handy-he: Hebrew by default, plus a "hey Claude" wake phrase

Fork of [cjpais/handy](https://github.com/cjpais/handy). Two changes, both aimed
at the same thing: make the app usable in Hebrew without configuring anything,
and let a recording start by voice instead of a keypress.

## What upstream already had

Worth stating, because it determines how small these changes are:

- a complete Hebrew UI locale plus RTL handling (`src/i18n/locales/he`, `src/lib/utils/rtl.ts`)
- Hebrew in the transcription language list, and `he` among Whisper's supported languages
- `ModelSource::HuggingFace { repo_id, revision }` — models can be pulled straight from the Hub
- an always-on microphone mode that keeps the capture stream open between recordings
- a recorder that already resamples to 16 kHz continuously, recording or not
- `send_transcription_input()`, the entry point signal handlers and CLI flags use to start a recording

So neither feature is built from scratch. The Hebrew work is defaults; the wake
word is a second trigger for an existing pipeline.

## Part 1 — Hebrew out of the box

### Model

Ship [`ivrit-ai/whisper-large-v3-turbo-ggml`](https://huggingface.co/ivrit-ai/whisper-large-v3-turbo-ggml)
(1.62 GB, single `ggml-model.bin`) as the default, registered through the
existing `HuggingFace` model source and pinned to a commit SHA, matching how the
bundled catalog pins its own entries.

This replaces the originally requested Whisper Medium. Medium is a generic
multilingual model; the ivrit.ai build is Whisper Large v3 Turbo fine-tuned on
Hebrew and is meaningfully more accurate on it. The cost is size (1.6 GB vs
469 MB) and that it is Hebrew-only in practice.

That last point is not incidental — the model card states language detection and
the translation task were both degraded during fine-tuning. The registry entry
therefore declares `supported_languages: ["he"]` with detection and translation
off, so the UI never offers a mode the weights cannot serve.

Parakeet V3 loses its `is_recommended` flag in this fork: it covers 25 European
languages and Hebrew is not among them.

### Defaults

- `default_selected_language()` → `he` (was `auto`, which would actively hurt given the degraded detection)
- `default_app_language()` → `he` (was the system locale)
- onboarding downloads the Hebrew model automatically instead of presenting a picker

The auto-download is skipped when a model is already on disk, and cancelling it
drops back to the normal picker. Other models stay reachable in settings for
English dictation.

Model list ordering needed one change: the sort keys on catalog rank, and
non-catalog ids rank `u32::MAX`, which would have buried the fork's default at
the bottom. `ModelManager::sort_rank` puts it first and shifts everything else
by one.

## Part 2 — "hey Claude"

### Where the audio comes from

The recorder's consumer loop already resamples every chunk to 16 kHz and hands
the frames to `handle_frame`, which drops them unless a recording is active. A
`with_monitor_callback` hook takes those otherwise-discarded frames.

The callback fires only when **not** recording. That single condition is what
keeps the spotter from hearing the user's own dictation and re-triggering on it
— no coordination with the recorder's state machine, no mute flag to get wrong.

This only works in always-on microphone mode, so enabling the wake word turns
that on and persists it.

### Detection

`livekit-wakeword` (Apache-2.0) — actively maintained, trains from synthetic TTS
with no recording required, and lists Hebrew among its supported languages.

Rustpotter was evaluated first and rejected: its last release is from October
2023 and it no longer compiles, failing with 20 errors inside an old
`candle-core` against current `half`/`rand_distr`.

The crate is **vendored** into `src-tauri/src/wakeword/livekit_wakeword` rather
than depended on. Published `livekit-wakeword 0.1.3` pins `ort-tract 0.2`, which
pins `ort-sys =2.0.0-rc.11`; Handy's `vad-rs` pins `ort-sys =2.0.0-rc.12`. Cargo
cannot satisfy both. Vendoring drops `ort-tract` entirely, which is the better
outcome anyway — the wake word models now run on the accelerated ONNX Runtime
Handy already links instead of adding a second inference engine to the binary.
Upstream's resampling path is dropped with it, since audio always arrives at
16 kHz.

### Trigger policy

`SpotterCore` holds a 2-second sliding window and scores it every 250 ms of
fresh audio, so a phrase straddling a window boundary is still caught.

Two rules keep it honest:

- **A detection empties the window.** The next one cannot happen until two more
  seconds of audio arrive, which is what stops one utterance from firing on
  every overlapping window. An explicit cooldown timer was written first and
  then removed — it duplicated this guarantee.
- **A gap longer than 300 ms restarts the window.** Frames stop flowing during a
  recording and while the queue is saturated, and scoring across such a seam
  would be meaningless.

`SpotterCore` takes `now` as a parameter and owns no threads or channels, so the
whole policy is tested synchronously with a fake clock. The thread around it is
plumbing: a bounded queue that drops frames rather than ever blocking the audio
thread.

On detection it calls `send_transcription_input(app, "transcribe", "wakeword")`
— exactly what a shortcut press does. Everything downstream (recording, VAD
endpointing, transcription, paste) is inherited unchanged. Recording stops the
same way it does for any hands-free start: Silero VAD detects the end of speech.

### Models

Two classifiers ship and vote, with the higher score winning:

- `hey_claude_en.onnx` — Piper backbone, 904 speakers, the primary model
- `hey_claude_he.onnx` — VoxCPM2, Hebrew phrase, the second voter

LiveKit document that their multilingual models score worse than English ones,
because the frozen speech-embedding front-end is English-biased and VoxCPM
produces less diverse speech than Piper's speaker pool. Rather than pick one on
theory, both are trained and both are loaded; `target_fp_per_hour: 0.1` (one
false start per ten hours) sets the operating point, which matters more than
usual for an always-on trigger.

Missing model files are not an error: the feature reports itself unavailable and
the settings UI hides the toggle rather than offering a dead control.

## Testing

- `SpotterCore` policy: partial windows, threshold edges, live threshold changes, gap handling, a detector that always errors, and sample clamping — nine synchronous tests, no sleeps
- the Rust suite and the frontend typecheck/lint
- the built app on macOS: Hebrew UI and model download on first run, then the wake phrase spoken aloud

## Out of scope

Text-to-speech replies, routing transcripts into Claude Code, and Windows/Linux
verification. The code stays buildable on all three; only macOS is tested.
