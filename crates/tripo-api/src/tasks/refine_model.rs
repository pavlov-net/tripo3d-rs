//! `refine_model` task variant.

use serde::{Deserialize, Serialize};

/// Request body for `refine_model`. Wire `type`: `refine_model`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RefineModelRequest {
    /// Draft (pre-refinement) task id.
    pub draft_model_task_id: String,
}
