//! `mesh_completion` task variant. Endpoint: `POST /mesh/complete`.

use serde::{Deserialize, Serialize};

/// Request body for `POST /mesh/complete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MeshCompletionRequest {
    /// `task_id` of a `mesh/segment` task.
    pub input: String,
    /// Completion model version; see `versions::mesh`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Restrict to named parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_names: Option<Vec<String>>,
}
