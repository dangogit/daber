# handy-he: Hebrew by default

Fork of [cjpais/handy](https://github.com/cjpais/handy). One goal: make the app
usable in Hebrew without configuring anything. Install, press the shortcut,
speak Hebrew, get Hebrew.

## What upstream already had

Worth stating, because it determines how small this change is:

- a complete Hebrew UI locale plus RTL handling (`src/i18n/locales/he`, `src/lib/utils/rtl.ts`)
- Hebrew in the transcription language list, and `he` among Whisper's supported languages
- `ModelSource::HuggingFace { repo_id, revision }` — models can be pulled straight from the Hub

Nothing here is built from scratch. This is a change of defaults.

## The model

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

## Defaults

- `default_selected_language()` → `he` (was `auto`, which would actively hurt given the degraded detection)
- `default_app_language()` → `he` (was the system locale)
- onboarding downloads the Hebrew model automatically instead of presenting a picker

The auto-download is skipped when a model is already on disk, and cancelling it
drops back to the normal picker. Other models stay reachable in settings for
English dictation, which the ivrit.ai model is not suited to.

Model list ordering needed one change: the list sorts by catalog rank, and
non-catalog ids rank `u32::MAX`, which would have buried the fork's default at
the bottom. `ModelManager::sort_rank` puts it first and shifts everything else
by one.

## Identity

The fork takes `ai.saasit.handy-he` / "Handy HE" rather than inheriting
`com.pais.handy`. Sharing an identifier with an installed upstream Handy means
sharing its settings store, model directory and history database — and being
offered upstream's builds as updates, which would replace the fork. The cost of
separating is the usual one: microphone and accessibility permissions are
granted per bundle id, so this install asks for its own.

The updater points at this repository's releases. Its public key is still
upstream's, so a published release would fail signature verification until it is
replaced with a keypair for this repo. Failing closed is the right default
meanwhile.

## Testing

- `ModelManager::sort_rank` keeps the Hebrew model ahead of all 67 catalog entries
- the upstream Rust suite, the frontend typecheck, lint, and the translation completeness check
- a real run against a fresh profile, confirming `app_language: he` and `selected_language: he` are what actually land in the settings store
- transcribing Hebrew speech end to end through the built app

## Dropped: the "hey Claude" wake phrase

A wake phrase trigger was built and then removed at the user's request — a
keyboard shortcut is simpler and upstream already has one.

It worked: an always-on spotter fed by the recorder's idle 16 kHz frames,
scoring a 2-second sliding window with a vendored `livekit-wakeword`, firing the
same `send_transcription_input` entry point the shortcut uses. Testing it against
real speech turned up a false positive that whole-clip scoring hides — a sliding
scan of a negative clip produced one window at 0.76 with every neighbour below
0.001 — which was fixed by requiring two consecutive windows above threshold.

What stopped it was the classifier. Training one means a multi-hour synthesis and
training run; the first attempt exhausted GPU memory at 94% of an 8000-clip
synthesis, and continuing risked swapping a machine that was in use.

The implementation is recoverable from git history (`git log --all --oneline
--grep=wake`) if it is ever wanted.
