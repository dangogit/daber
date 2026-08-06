//! [`WakewordDetector`] backed by the `livekit-wakeword` crate.
//!
//! The mel-spectrogram and speech-embedding stages are compiled into the crate;
//! only the trained classifier (a small ONNX file) is loaded from disk. Several
//! classifiers can be active at once — this fork ships a Hebrew and an English
//! rendering of the same phrase and takes whichever scores higher, so the
//! trigger works whichever way the phrase comes out.

use super::livekit_wakeword::WakeWordModel;
use super::WakewordDetector;
use anyhow::{anyhow, Context, Result};
use log::info;
use std::path::{Path, PathBuf};

/// The classifiers are trained at one sample rate and the vendored code no
/// longer resamples, so the recorder's output rate has to match exactly.
const _: () = assert!(
    crate::audio_toolkit::constants::WHISPER_SAMPLE_RATE == super::livekit_wakeword::SAMPLE_RATE
);

pub struct LiveKitDetector {
    model: WakeWordModel,
}

impl LiveKitDetector {
    /// Load every classifier in `model_paths`. Paths that do not exist are
    /// skipped, so shipping only one of the phrase variants still works; an
    /// empty result is an error rather than a detector that can never fire.
    pub fn new(model_paths: &[PathBuf]) -> Result<Self> {
        let present: Vec<&Path> = model_paths
            .iter()
            .filter(|p| p.exists())
            .map(|p| p.as_path())
            .collect();

        if present.is_empty() {
            return Err(anyhow!(
                "no wake word classifier found (looked for: {})",
                model_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        info!(
            "Loading {} wake word classifier(s): {}",
            present.len(),
            present
                .iter()
                .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let model = WakeWordModel::new(&present)
            .map_err(|e| anyhow!("{e}"))
            .context("failed to initialise wake word model")?;

        Ok(Self { model })
    }
}

impl WakewordDetector for LiveKitDetector {
    fn score(&mut self, window: &[i16]) -> Result<f32> {
        let scores = self
            .model
            .predict(window)
            .map_err(|e| anyhow!("wake word prediction failed: {e}"))?;

        // Highest-scoring classifier wins; no classifier at all scores zero
        // rather than erroring, so a bad model file degrades to "never fires".
        Ok(scores.values().copied().fold(0.0f32, f32::max))
    }
}

/// End-to-end checks against real speech.
///
/// These run the whole vendored stack — mel spectrogram, speech embeddings and
/// classifier, all on Handy's ONNX Runtime — over LiveKit's own recordings of
/// someone saying "hey LiveKit". The phrase is theirs, not ours; what is being
/// verified is that the inference path and the windowing policy work on actual
/// audio, which is independent of which phrase the shipped classifier detects.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::wakeword::{SpotterCore, WakewordDetector, HOP_SAMPLES, WINDOW_SAMPLES};
    use std::time::{Duration, Instant};

    const THRESHOLD: f32 = 0.5;
    /// The recorder's frame size: 30 ms at 16 kHz.
    const FRAME: usize = 480;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("wakeword")
    }

    fn detector() -> LiveKitDetector {
        LiveKitDetector::new(&[fixtures_dir().join("hey_livekit.onnx")])
            .expect("test classifier should load")
    }

    /// Read a fixture as 16 kHz mono `f32`, the form the recorder delivers.
    fn read_fixture(name: &str) -> Vec<f32> {
        let path = fixtures_dir().join(name);
        let mut reader = hound::WavReader::open(&path)
            .unwrap_or_else(|e| panic!("failed to open {}: {e}", path.display()));
        let spec = reader.spec();
        assert_eq!(
            spec.sample_rate, 16_000,
            "fixture must already be at the recorder's rate"
        );

        let all: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        let mono: Vec<i16> = if spec.channels > 1 {
            all.chunks(spec.channels as usize).map(|c| c[0]).collect()
        } else {
            all
        };
        mono.iter().map(|s| *s as f32 / 32768.0).collect()
    }

    #[test]
    fn separates_the_wake_phrase_from_other_audio() {
        let mut d = detector();

        let positive: Vec<i16> = read_fixture("positive.wav")
            .iter()
            .map(|s| super::super::to_i16(*s))
            .collect();
        let negative: Vec<i16> = read_fixture("negative.wav")
            .iter()
            .map(|s| super::super::to_i16(*s))
            .collect();

        let pos_score = d.score(&positive).expect("scoring the positive clip");
        let neg_score = d.score(&negative).expect("scoring the negative clip");

        assert!(
            pos_score >= THRESHOLD,
            "spoken wake phrase should clear the threshold, scored {pos_score:.3}"
        );
        assert!(
            neg_score < THRESHOLD,
            "unrelated audio should stay below the threshold, scored {neg_score:.3}"
        );
    }

    /// Drives the real classifier through the real windowing policy, frame by
    /// frame, the way the audio thread does.
    fn fires_on(name: &str) -> bool {
        let samples = read_fixture(name);
        let mut core = SpotterCore::new(Box::new(detector()));
        let mut now = Instant::now();

        // The fixtures are about two seconds — exactly one window — so pad them
        // out to give the sliding window somewhere to land. The padding is a
        // deterministic low-level hiss rather than digital zeros, because a real
        // microphone always has a noise floor and the mel front-end behaves
        // differently on true silence.
        let mut seed: u32 = 12_345;
        let mut hiss = move |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    ((seed >> 16) as f32 / 32_768.0 - 1.0) * 0.0008
                })
                .collect()
        };
        let padded: Vec<f32> = hiss(HOP_SAMPLES)
            .into_iter()
            .chain(samples)
            .chain(hiss(WINDOW_SAMPLES))
            .collect();

        let mut fired = false;
        for frame in padded.chunks(FRAME) {
            now += Duration::from_millis(30);
            if core.push(frame, THRESHOLD, now) {
                fired = true;
            }
        }
        fired
    }

    #[test]
    fn spotter_fires_on_spoken_wake_phrase_and_stays_quiet_otherwise() {
        assert!(fires_on("positive.wav"), "wake phrase should trigger");
        assert!(
            !fires_on("negative.wav"),
            "unrelated speech must not trigger"
        );
    }
}
