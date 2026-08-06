# Wake word training

Configs for producing the "hey Claude" classifiers this fork ships. Training is
done with [livekit-wakeword](https://github.com/livekit/livekit-wakeword), which
synthesizes its own training data — no recording required.

The app loads whatever it finds; you do not have to retrain to use it.

## What the app expects

| File                                       | Where                                        |
| ------------------------------------------ | -------------------------------------------- |
| `hey_claude_en.onnx`, `hey_claude_he.onnx` | `src-tauri/resources/wakeword/` (shipped)    |
| same names                                 | `<app data>/wakeword/` (drop-in, no rebuild) |

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
livekit-wakeword run /path/to/hey_claude_en.yaml
```

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
