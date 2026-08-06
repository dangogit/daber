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
