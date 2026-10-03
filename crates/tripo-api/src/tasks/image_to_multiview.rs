//! `image_to_multiview` task variant. Endpoint: `POST /generation/image-to-multiview`.

use serde::{Deserialize, Serialize};

use crate::image::ImageInput;

/// Request body for `POST /generation/image-to-multiview`. The finished task
/// reports `output.{front,left,back,right}_view_url`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ImageToMultiviewRequest {
    /// Source image: URL, `file_token`, local path, or the `task_id` of a
    /// prior text-to-image or image-to-image task. Works best with the subject
    /// on a clean background.
    pub input: ImageInput,
}
