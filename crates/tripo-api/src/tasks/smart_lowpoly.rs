//! `smart_lowpoly` task variant. Wire `type`: `highpoly_to_lowpoly`.

use serde::{Deserialize, Serialize};

/// Request body for `smart_lowpoly`. Wire `type`: `highpoly_to_lowpoly`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct SmartLowpolyRequest {
    /// Source (high-poly) task id.
    pub original_model_task_id: String,
    /// Model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Produce a quad mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad: Option<bool>,
    /// Restrict to named parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_names: Option<Vec<String>>,
    /// Face count limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_limit: Option<i32>,
    /// Bake textures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake: Option<bool>,
}
