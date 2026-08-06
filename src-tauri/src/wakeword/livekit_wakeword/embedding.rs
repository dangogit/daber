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

//! Vendored from `livekit-wakeword` v0.1.3 — see the module docs in `mod.rs`.

use ndarray::{Array, Array1};
use ort::session::Session;
use ort::value::Tensor;

use super::{build_session_from_memory, WakeWordError, EMBEDDING_DIM};

const MODEL_BYTES: &[u8] = include_bytes!("onnx/embedding_model.onnx");

/// Produces a 96-dim embedding from mel spectrogram features.
///
/// Model input:  f32 tensor of shape (batch, 76, 32, 1) — mel spectrogram features
/// Model output: f32 tensor of shape (batch, 1, 1, 96) — embedding vector
pub struct EmbeddingModel {
    session: Session,
}

impl EmbeddingModel {
    pub fn new() -> Result<Self, WakeWordError> {
        Ok(Self {
            session: build_session_from_memory(MODEL_BYTES)?,
        })
    }

    /// Run the model on a flat slice of 76 * 32 = 2432 mel values (row-major)
    /// and return the 96-element embedding.
    pub fn detect(&mut self, mel_features: &[f32]) -> Result<Array1<f32>, WakeWordError> {
        let input = Array::from_shape_vec((1, 76, 32, 1), mel_features.to_vec())?;
        let tensor = Tensor::from_array(input)?;

        let outputs = self.session.run(ort::inputs![tensor])?;

        let raw = outputs["conv2d_19"].try_extract_array::<f32>()?;
        let embedding = raw.into_owned().into_shape_with_order(EMBEDDING_DIM)?;

        Ok(embedding)
    }
}
