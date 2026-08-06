# Wake word training

Configs for producing the "hey Claude" classifiers this fork ships. Training is
done with [livekit-wakeword](https://github.com/livekit/livekit-wakeword), which
synthesizes its own training data — no recording required.

**No classifier ships with the fork today.** Until one is trained and installed,
the wake phrase feature is dormant and its toggle does not appear in settings.
Everything else about the app is unaffected.

## What the app expects

| File                                       | Where                                             |
| ------------------------------------------ | ------------------------------------------------- |
| `hey_claude_en.onnx`, `hey_claude_he.onnx` | `src-tauri/resources/wakeword/` (needs a rebuild) |
| same names                                 | `<app data>/wakeword/` (drop-in, no rebuild)      |

On macOS the app data directory is
`~/Library/Application Support/ai.saasit.handy-he/wakeword/`.

Both files are optional. Any that are present are loaded together and the
highest score wins, so the trigger fires whether the phrase comes out closer to
English or to Hebrew. With neither file present the feature disappears from
settings instead of offering a toggle that cannot work.

## Retraining

```bash
git clone https://github.com/livekit/livekit-wakeword
cd livekit-wakeword
uv sync --extra train --extra eval --extra export --extra voxcpm
```

`--all-extras` pulls in `pyaudio`, which needs portaudio headers and is only
used by the Python microphone listener. This app listens in Rust, so skip it.

macOS also needs `brew install espeak-ng ffmpeg`.

Then, with a config from this directory:

```bash
livekit-wakeword setup --config configs/prod.yaml   # ~16 GB of negatives, RIRs, noise
PYTORCH_MPS_HIGH_WATERMARK_RATIO=0.5 livekit-wakeword run /path/to/hey_claude_en.yaml
```

**The cap is not optional on Apple Silicon.** MPS defaults to a high watermark of 1.7x the
Metal recommended working set, which on a 24 GB Mac is roughly 32 GB, and unified memory
means every one of those GPU buffers is system RAM. Piper holds its allocations across the
whole synthesis pass rather than freeing per clip, so an uncapped run reaches a 28 GB
footprint somewhere around clip 7500 of 8000, fills swap, and hangs the machine. At `0.5`
the allocator raises an out-of-memory error instead, which is a failure you can read.

The run is also not resumable: clips already written to `output/<model_name>/` are ignored on
restart and regenerated from scratch. Expect roughly an hour of synthesis before training
steps even begin, and do not start it on a machine you need for anything else.

Pass `--skip-acav` to `setup` to avoid the 16 GB ACAV100M download. Expect more
false triggers if you do: that corpus is what teaches the model to stay quiet
through everyday audio, which matters more here than usual because the
microphone is always on.

The export lands at `output/<model_name>/<model_name>.onnx`. Copy it to
`src-tauri/resources/wakeword/`.

## Why two configs

`hey_claude_en.yaml` is the primary model. Piper synthesizes from a 904-speaker
pool and the frozen Google speech-embedding front-end was trained mostly on
English, so English models come out measurably stronger — LiveKit document this
gap themselves. "Hey Claude" in an Israeli accent stays close enough to the
English phonemes for this model to carry most of the work.

`hey_claude_he.yaml` covers the Hebrew rendering with VoxCPM2. It synthesizes
one clip at a time rather than in batches, so its sample counts are an order of
magnitude smaller — it is the second voter, not a replacement.

## Tuning

`livekit-wakeword eval <config>` writes a DET curve and a metrics JSON, and
reports the threshold that meets `target_fp_per_hour` (0.1 in these configs — one
false start per ten hours). Use that number as the default for the sensitivity
slider in settings rather than guessing.
