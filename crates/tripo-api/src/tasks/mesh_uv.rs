//! `mesh_uv` task variant (Smart UV). Endpoint: `POST /mesh/uv`.

use serde::{Deserialize, Serialize};

use crate::compress::CompressionMode;
use crate::enums::ExportOrientation;
use crate::image::ImageInput;

/// Request body for `POST /mesh/uv`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MeshUvRequest {
    /// Model source: `task_id`, `file_token`, URL, or local path (uploaded
    /// first). At most 80,000 equivalent triangles (a quad counts as two);
    /// larger models fail after submission and are refunded, so run
    /// `mesh/decimate` on them first.
    pub input: ImageInput,
    /// Smart UV model version. The server documents only `default`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Bake existing textures onto the new UV layout (server default `true`).
    /// `false` returns geometry with UVs only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake: Option<bool>,
    /// Export forward axis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_orientation: Option<ExportOrientation>,
    /// Geometry compression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<CompressionMode>,
}
