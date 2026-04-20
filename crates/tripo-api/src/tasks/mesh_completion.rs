//! `mesh_completion` task variant.

use serde::Serialize;

/// Request body for `mesh_completion`. Wire `type`: `mesh_completion`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MeshCompletionRequest {
    /// Source task id.
    pub original_model_task_id: String,
    /// Model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Restrict to named parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_names: Option<Vec<String>>,
}
