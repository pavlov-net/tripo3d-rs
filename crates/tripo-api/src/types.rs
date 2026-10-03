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
    /// Banned by moderation.
    Banned,
    /// Past retention.
    Expired,
    /// Unrecognized status string from a newer server; not terminal.
    #[serde(other)]
    Unknown,
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
    /// Opaque token to pass back as `ImageInput::FileToken` or the `input`
    /// field of other API calls.
    pub file_token: String,
}

/// Response of `POST /files/presign`: where to `PUT` the file, and the token
/// that refers to it once the `PUT` completes.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PresignedUpload {
    /// Presigned storage URL accepting a single HTTP `PUT` of the raw file.
    /// Older responses name this field `upload_url`.
    #[serde(alias = "upload_url")]
    pub presigned_url: String,
    /// Token to pass as `input` once the upload completes.
    pub file_token: String,
    /// Validity of `presigned_url` in seconds (documented as 1800).
    #[serde(default)]
    pub expires_in: Option<u64>,
}

/// Download URLs and auxiliary output fields returned on the task object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TaskOutput {
    /// URL for the main output model.
    #[serde(default)]
    pub model_url: Option<String>,
    /// URL for a rendered preview image.
    #[serde(default)]
    pub rendered_image_url: Option<String>,
    /// URL for the intermediate generated image (`text_to_model` only).
    #[serde(default)]
    pub generated_image_url: Option<String>,
    /// Populated by `check_riggable`.
    #[serde(default)]
    pub riggable: Option<bool>,
    /// Populated by `check_riggable`.
    #[serde(default)]
    pub rig_type: Option<crate::enums::RigTypeResponse>,
    /// Output fields without a typed accessor (e.g. smartsegment's
    /// `mask_url`), kept verbatim.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Task record returned by `GET /tasks/{id}`.
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
    /// Server error code; present on some failed tasks.
    #[serde(default)]
    pub error_code: Option<i64>,
    /// Human-readable failure reason; accompanies `error_code`.
    #[serde(default, alias = "error_msg")]
    pub error_message: Option<String>,
    /// ISO 8601 creation time (e.g. `2026-04-28T12:00:00Z`).
    #[serde(default)]
    pub created_at: String,
    /// ISO 8601 completion time; set once terminal.
    #[serde(default)]
    pub completed_at: Option<String>,
    /// Credits consumed by this task.
    #[serde(default)]
    pub credits_consumed: Option<f64>,
    /// Estimated seconds until completion; used by the polling backoff when
    /// present.
    #[serde(default)]
    pub running_left_time: Option<i64>,
    /// Queue depth ahead of this task.
    #[serde(default)]
    pub queuing_num: Option<i32>,
}

/// Result of [`Client::list_tasks`](crate::Client::list_tasks).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TaskList {
    /// Tasks found, in the order they were requested.
    pub tasks: Vec<Task>,
    /// Requested ids the server did not return (unknown, owned by another
    /// API key, or expired).
    pub missed: Vec<TaskId>,
}

impl TaskList {
    /// Normalize the `data` payload of `POST /tasks/list`. Accepts the
    /// documented `{tasks: {<id>: Task}, missed: [..]}` form as well as
    /// `{tasks: [Task]}` and a bare `[Task]`. When the server omits
    /// `missed`, it is derived from `requested`.
    pub(crate) fn from_data(
        data: serde_json::Value,
        requested: &[TaskId],
    ) -> serde_json::Result<Self> {
        use serde_json::Value;

        let (tasks, missed) = match data {
            Value::Object(mut obj) => (
                obj.remove("tasks").unwrap_or_default(),
                obj.remove("missed"),
            ),
            other => (other, None),
        };
        let tasks: Vec<Task> = match tasks {
            Value::Null => Vec::new(),
            Value::Object(mut by_id) => {
                let mut ordered = Vec::with_capacity(by_id.len());
                for id in requested {
                    if let Some(v) = by_id.remove(id.as_str()) {
                        ordered.push((id.0.clone(), v));
                    }
                }
                ordered.extend(by_id);
                ordered
                    .into_iter()
                    .map(|(id, mut v)| {
                        if let Value::Object(o) = &mut v {
                            o.entry("task_id").or_insert(Value::String(id));
                        }
                        serde_json::from_value(v)
                    })
                    .collect::<serde_json::Result<_>>()?
            }
            other => serde_json::from_value(other)?,
        };
        let missed = match missed
            .map(serde_json::from_value::<Option<Vec<TaskId>>>)
            .transpose()?
        {
            Some(Some(missed)) => missed,
            _ => requested
                .iter()
                .filter(|id| !tasks.iter().any(|t| &t.task_id == *id))
                .cloned()
                .collect(),
        };
        Ok(Self { tasks, missed })
    }
}

/// Query parameters for [`Client::get_usage`](crate::Client::get_usage).
/// Unset fields are omitted, leaving paging to the server's defaults.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UsageQuery {
    /// Maximum number of records to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Number of records to skip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// A timestamp as sent by the server: an ISO 8601 string or an epoch number.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Timestamp {
    /// Numeric epoch time, unit as sent by the server.
    Epoch(serde_json::Number),
    /// Textual time, normally ISO 8601 (e.g. `2026-01-01T00:00:00Z`).
    Text(String),
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Epoch(n) => n.fmt(f),
            Self::Text(s) => f.write_str(s),
        }
    }
}

/// One per-task credit charge from `GET /account/usage`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UsageRecord {
    /// Task that consumed the credits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    /// Wire-format task type string (e.g. `text_to_model`).
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    /// Task status string, when reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Credits consumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credits_consumed: Option<f64>,
    /// Creation time; also read from the v2 name `create_time`.
    #[serde(
        default,
        alias = "create_time",
        skip_serializing_if = "Option::is_none"
    )]
    pub created_at: Option<Timestamp>,
    /// Fields not modeled above, passed through unchanged.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Normalize the `data` payload of `GET /account/usage`: a bare array, or an
/// object wrapping the array (the first array-valued field is used).
pub(crate) fn usage_from_data(data: serde_json::Value) -> serde_json::Result<Vec<UsageRecord>> {
    let list = match data {
        serde_json::Value::Object(obj) => obj
            .into_values()
            .find(serde_json::Value::is_array)
            .ok_or_else(|| {
                <serde_json::Error as serde::de::Error>::custom(
                    "usage response object contains no record list",
                )
            })?,
        other => other,
    };
    serde_json::from_value(list)
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
        assert!(task.output.model_url.is_none());
    }

    #[test]
    fn deserializes_v3_task_body() {
        let body = r#"{
            "task_id":"task_abc123","type":"text_to_model","status":"success",
            "progress":100,
            "output":{"model_url":"https://cdn/m.glb","rendered_image_url":"https://cdn/p.png"},
            "credits_consumed":30.0,
            "created_at":"2026-04-28T12:00:00Z","completed_at":"2026-04-28T12:01:30Z"
        }"#;
        let task: Task = serde_json::from_str(body).unwrap();
        assert_eq!(task.status, TaskStatus::Success);
        assert_eq!(task.output.model_url.as_deref(), Some("https://cdn/m.glb"));
        assert_eq!(task.credits_consumed, Some(30.0));
        assert_eq!(task.created_at, "2026-04-28T12:00:00Z");
        assert_eq!(task.completed_at.as_deref(), Some("2026-04-28T12:01:30Z"));
    }

    #[test]
    fn deserializes_failure_detail() {
        let body = r#"{"task_id":"t","type":"text_to_model","status":"failed",
            "error_code":2018,"error_message":"model too complex"}"#;
        let task: Task = serde_json::from_str(body).unwrap();
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error_code, Some(2018));
        assert_eq!(task.error_message.as_deref(), Some("model too complex"));
    }

    #[test]
    fn accepts_legacy_error_msg_spelling() {
        let body = r#"{"task_id":"t","type":"text_to_model","status":"failed","error_msg":"nope"}"#;
        let task: Task = serde_json::from_str(body).unwrap();
        assert_eq!(task.error_message.as_deref(), Some("nope"));
    }

    #[test]
    fn unknown_status_is_not_a_decode_error() {
        let body = r#"{"task_id":"t","type":"text_to_model","status":"paused"}"#;
        let task: Task = serde_json::from_str(body).unwrap();
        assert_eq!(task.status, TaskStatus::Unknown);
        assert!(!task.status.is_terminal());
    }

    #[test]
    fn unknown_output_fields_round_trip() {
        let output = serde_json::json!({
            "model_url": "https://cdn/m.glb",
            "mask_url": "https://cdn/mask.png",
            "seg_task_id": "seg_1"
        });
        let parsed: TaskOutput = serde_json::from_value(output.clone()).unwrap();
        assert_eq!(parsed.model_url.as_deref(), Some("https://cdn/m.glb"));
        assert_eq!(parsed.extra["mask_url"], "https://cdn/mask.png");
        assert!(!parsed.extra.contains_key("model_url"));

        let mut reserialized = serde_json::to_value(&parsed).unwrap();
        reserialized
            .as_object_mut()
            .unwrap()
            .retain(|_, v| !v.is_null());
        assert_eq!(reserialized, output);
    }
}
