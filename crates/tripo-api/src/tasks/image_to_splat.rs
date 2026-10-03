//! `image_to_splat` task variant. Endpoint: `POST /generation/image-to-splat`.

use serde::{Deserialize, Serialize};

use crate::image::ImageInput;

/// Request body for `POST /generation/image-to-splat`. The task's
/// `output.model_url` is a 3D Gaussian Splat (`.splat`) file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ImageToSplatRequest {
    /// Source image: URL, `file_token`, or local path (uploaded first).
    /// PNG, JPEG, or WebP; at most 20 MB.
    pub input: ImageInput,
    /// Seed; the same seed and input produce an identical `.splat`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_seed: Option<i32>,
}
