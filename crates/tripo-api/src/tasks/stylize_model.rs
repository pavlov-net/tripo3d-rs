//! `stylize_model` task variant. Endpoint: `POST /models/stylize` (legacy, undocumented in v3).

use serde::{Deserialize, Serialize};

use crate::enums::PostStyle;

/// Request body for `POST /models/stylize`. Note: this legacy endpoint keeps
/// v2 field names (`original_model_task_id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct StylizeModelRequest {
    /// Source task id.
    pub original_model_task_id: String,
    /// Stylization preset.
    pub style: PostStyle,
    /// Voxel block size (for voxel/minecraft styles).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_size: Option<i32>,
}
