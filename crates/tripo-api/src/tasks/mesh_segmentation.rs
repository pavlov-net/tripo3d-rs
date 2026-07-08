//! `mesh_segmentation` task variant. Endpoint: `POST /mesh/segment`.

use serde::{Deserialize, Serialize};

/// Request body for `POST /mesh/segment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MeshSegmentationRequest {
    /// Model source: `task_id`, `file_token`, or URL.
    pub input: String,
    /// Segmentation model version; see `versions::mesh`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
