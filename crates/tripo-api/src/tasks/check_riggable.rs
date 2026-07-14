//! `check_riggable` task variant. Endpoint: `POST /animations/rig-check`.

use serde::{Deserialize, Serialize};

/// Request body for `POST /animations/rig-check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct CheckRiggableRequest {
    /// Model source: `task_id`, `file_token`, or URL.
    pub input: String,
}
