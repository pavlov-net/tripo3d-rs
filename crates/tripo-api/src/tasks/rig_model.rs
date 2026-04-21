//! `rig_model` task variant. Wire `type`: `animate_rig`.

use serde::{Deserialize, Serialize};

use crate::enums::{RigOutputFormat, RigSpec, RigType};

/// Request body for `rig_model`. Wire `type`: `animate_rig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RigModelRequest {
    /// Source task id.
    pub original_model_task_id: String,
    /// Model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Output file format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_format: Option<RigOutputFormat>,
    /// Rig classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rig_type: Option<RigType>,
    /// Skeleton spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<RigSpec>,
}
