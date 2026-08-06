// Copyright 2026 LiveKit, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Vendored from the `livekit-wakeword` crate v0.1.3 (Apache-2.0), see
//! <https://github.com/livekit/livekit-wakeword>.
//!
//! Vendored rather than depended on because the published crate pins
//! `ort-tract 0.2`, which in turn pins `ort-sys =2.0.0-rc.11`, while Handy's
//! `vad-rs` pins `ort-sys =2.0.0-rc.12`. Cargo cannot satisfy both. Dropping
//! `ort-tract` resolves the conflict and is the better outcome anyway: the wake
//! word models now run on the same accelerated ONNX Runtime Handy already links
//! instead of adding a second, pure-Rust inference engine to the binary.
//!
//! Changes from upstream:
//!   * removed the `ort-tract` backend selection (`use_tract` cfg and
//!     `ensure_tract_backend`) — the default `ort` backend is Handy's
//!   * removed input resampling and the `resampler` dependency; the recorder
//!     already delivers 16 kHz, the rate these models are trained on
//!   * `WakeWordError` gained no variants; the resampling ones were dropped

mod embedding;
mod melspectrogram;
mod model;

pub use model::WakeWordModel;

use ort::session::Session;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum WakeWordError {
    #[error(transparent)]
    Ort(#[from] ort::Error),
    #[error(transparent)]
    Shape(#[from] ndarray::ShapeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("wake word model not found: {0}")]
    ModelNotFound(String),
}

/// The only sample rate these models accept. Upstream resampled other rates
/// internally; here the caller is Handy's recorder, which resamples to exactly
/// this before anything else sees the audio.
pub const SAMPLE_RATE: u32 = 16_000;
pub const EMBEDDING_WINDOW: usize = 76; // mel frames per embedding
pub const EMBEDDING_STRIDE: usize = 8; // mel frames between embeddings
pub const EMBEDDING_DIM: usize = 96;
pub const MIN_EMBEDDINGS: usize = 16; // classifier input length

pub(crate) fn build_session_from_memory(bytes: &[u8]) -> Result<Session, WakeWordError> {
    Ok(Session::builder()?.commit_from_memory(bytes)?)
}

pub(crate) fn build_session_from_file(path: impl AsRef<Path>) -> Result<Session, WakeWordError> {
    let bytes = std::fs::read(path)?;
    Ok(Session::builder()?.commit_from_memory(&bytes)?)
}
