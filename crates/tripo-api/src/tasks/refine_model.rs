//! `refine_model` task variant. Endpoint: `POST /models/refine` (legacy, undocumented in v3).

use serde::{Deserialize, Serialize};

/// Request body for `POST /models/refine`. Note: this legacy endpoint keeps
/// v2 field names (`draft_model_task_id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RefineModelRequest {
    /// Draft (pre-refinement) task id.
    pub draft_model_task_id: String,
}
