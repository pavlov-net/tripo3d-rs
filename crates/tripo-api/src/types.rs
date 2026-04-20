//! Core data types exposed by the public API.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Opaque task identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct TaskId(pub String);

impl TaskId {
    /// Construct from any string-like.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// Borrow as `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for TaskId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for TaskId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Task lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Waiting in the queue.
    Queued,
    /// Currently being processed.
    Running,
    /// Completed successfully.
    Success,
    /// Completed with failure.
    Failed,
    /// User or system cancelled.
    Cancelled,
    /// Unknown / uncategorized.
    Unknown,
    /// Banned by moderation.
    Banned,
    /// Past retention.
    Expired,
}

impl TaskStatus {
    /// True for statuses that cause `wait_for_task` to stop polling.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Success | Self::Failed | Self::Cancelled | Self::Banned | Self::Expired
        )
    }
}

/// User account balance.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Balance {
    /// Available credit balance.
    pub balance: f64,
    /// Reserved (in-flight) balance.
    pub frozen: f64,
}

/// Server-side result of `upload_file`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UploadedFile {
    /// Opaque token to pass back as `ImageInput::FileToken`.
    pub file_token: uuid::Uuid,
}

/// Download URLs and auxiliary output fields returned on the task object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TaskOutput {
    /// URL for the main output model.
    #[serde(default)]
    pub model: Option<String>,
    /// URL for the base (pre-texture) model.
    #[serde(default)]
    pub base_model: Option<String>,
    /// URL for the PBR-textured model.
    #[serde(default)]
    pub pbr_model: Option<String>,
    /// URL for a rendered preview image.
    #[serde(default)]
    pub rendered_image: Option<String>,
    /// Populated by `check_riggable`.
    #[serde(default)]
    pub riggable: Option<bool>,
    /// Populated by `check_riggable`.
    #[serde(default)]
    pub rig_type: Option<crate::enums::RigType>,
}

/// Task record returned by `GET /task/{id}`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Task {
    /// Identifier.
    pub task_id: TaskId,
    /// Wire-format task type string (e.g. `text_to_model`, `animate_rig`).
    #[serde(rename = "type")]
    pub task_type: String,
    /// Current status.
    pub status: TaskStatus,
    /// Echo of request parameters.
    #[serde(default)]
    pub input: BTreeMap<String, serde_json::Value>,
    /// Output URLs and flags.
    #[serde(default)]
    pub output: TaskOutput,
    /// Progress percent 0–100.
    #[serde(default)]
    pub progress: i32,
    /// Unix seconds.
    #[serde(default)]
    pub create_time: i64,
    /// Estimated seconds until completion; used by the polling backoff.
    #[serde(default)]
    pub running_left_time: Option<i64>,
    /// Queue depth ahead of this task.
    #[serde(default)]
    pub queuing_num: Option<i32>,
    /// Non-zero on failure.
    #[serde(default)]
    pub error_code: Option<i32>,
    /// Human-readable error message.
    #[serde(default)]
    pub error_msg: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_terminality() {
        assert!(TaskStatus::Success.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Banned.is_terminal());
        assert!(!TaskStatus::Queued.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
        assert!(!TaskStatus::Unknown.is_terminal());
    }

    #[test]
    fn deserializes_task_with_minimal_body() {
        let body = r#"{
            "task_id":"abc123","type":"text_to_model","status":"running","progress":42
        }"#;
        let task: Task = serde_json::from_str(body).unwrap();
        assert_eq!(task.task_id.as_str(), "abc123");
        assert_eq!(task.status, TaskStatus::Running);
        assert_eq!(task.progress, 42);
        assert!(task.output.model.is_none());
    }
}
