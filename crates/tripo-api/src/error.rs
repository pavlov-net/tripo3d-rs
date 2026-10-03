//! Error types returned by the client.

use std::path::PathBuf;

use crate::types::{Task, TaskId, TaskStatus};

/// Result alias using [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors returned by the client.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// HTTP layer error with no structured API envelope.
    #[error("HTTP {status}: {message}")]
    Http {
        /// HTTP status code.
        status: u16,
        /// Response body or derived message.
        message: String,
    },

    /// Structured API error envelope (Tripo returns
    /// `{code, message, suggestion, request_id}`).
    #[error("API [{code}] {message}{}{}",
        suggestion.as_deref().map(|s| format!(" — {s}")).unwrap_or_default(),
        request_id.as_deref().map(|r| format!(" (request_id: {r})")).unwrap_or_default())]
    Api {
        /// API-specific error code.
        code: i32,
        /// Human-readable error message.
        message: String,
        /// Optional suggestion for how to fix the error.
        suggestion: Option<String>,
        /// Request identifier for support/troubleshooting.
        request_id: Option<String>,
    },

    /// A task ended with a non-success terminal status. Build with
    /// [`Error::task_failed`].
    #[error("task {task_id} ended with status {status:?}{}", failure_detail(*error_code, error_message.as_deref()))]
    TaskFailed {
        /// Task identifier.
        task_id: TaskId,
        /// Terminal status.
        status: TaskStatus,
        /// Server error code, when reported.
        error_code: Option<i64>,
        /// Server failure reason, when reported.
        error_message: Option<String>,
    },

    /// `wait_for_task` exceeded its timeout.
    #[error("timed out waiting for task {0}")]
    WaitTimeout(TaskId),

    /// `TRIPO_API_KEY` not set and no key passed programmatically.
    #[error("missing API key (set TRIPO_API_KEY or pass --api-key)")]
    MissingApiKey,

    /// API key does not begin with `tsk_`.
    #[error("invalid API key (must start with `tsk_`)")]
    InvalidApiKey,

    /// Download target exists and `overwrite` was not set.
    #[error("file already exists: {0} (use --force to overwrite)")]
    FileExists(PathBuf),

    /// Client-side request validation failed before the request was sent.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// HTTP transport error.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// JSON (de)serialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Webhook delivery failed verification or parsing.
    #[cfg(feature = "webhook")]
    #[error(transparent)]
    Webhook(#[from] crate::webhook::WebhookError),
}

impl Error {
    /// [`Error::TaskFailed`] describing `task`'s terminal status and failure
    /// details.
    #[must_use]
    pub fn task_failed(task: &Task) -> Self {
        Self::TaskFailed {
            task_id: task.task_id.clone(),
            status: task.status,
            error_code: task.error_code,
            error_message: task.error_message.clone(),
        }
    }
}

fn failure_detail(code: Option<i64>, message: Option<&str>) -> String {
    match (code, message) {
        (Some(c), Some(m)) => format!(" (error {c}: {m})"),
        (Some(c), None) => format!(" (error {c})"),
        (None, Some(m)) => format!(": {m}"),
        (None, None) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_failed_display_includes_server_detail() {
        let task: Task = serde_json::from_value(serde_json::json!({
            "task_id": "t1", "type": "text_to_model", "status": "failed",
            "error_code": 2018, "error_message": "model too complex"
        }))
        .unwrap();
        assert_eq!(
            Error::task_failed(&task).to_string(),
            "task t1 ended with status Failed (error 2018: model too complex)"
        );
    }

    #[test]
    fn task_failed_display_without_detail() {
        let task: Task = serde_json::from_value(serde_json::json!({
            "task_id": "t1", "type": "text_to_model", "status": "cancelled"
        }))
        .unwrap();
        assert_eq!(
            Error::task_failed(&task).to_string(),
            "task t1 ended with status Cancelled"
        );
    }
}
