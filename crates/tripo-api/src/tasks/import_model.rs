//! `import_model` task variant. Endpoint: `POST /models/import`.

use serde::{Deserialize, Serialize};

use crate::image::ImageInput;

/// Request body for `POST /models/import`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ImportModelRequest {
    /// Model file: URL, `file_token`, or local path (uploaded first).
    /// GLB, GLTF, FBX, OBJ, or STL; at most 150 MB.
    pub input: ImageInput,
}
