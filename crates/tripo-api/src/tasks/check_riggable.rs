//! `check_riggable` task variant. Wire `type`: `animate_prerigcheck`.

use serde::Serialize;

/// Request body for `check_riggable`. Wire `type`: `animate_prerigcheck`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct CheckRiggableRequest {
    /// Source task id.
    pub original_model_task_id: String,
}
