//! `mesh_segmentation` task variant.

use serde::Serialize;

/// Request body for `mesh_segmentation`. Wire `type`: `mesh_segmentation`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MeshSegmentationRequest {
    /// Source task id.
    pub original_model_task_id: String,
    /// Model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
}
