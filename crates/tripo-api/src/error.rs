//! Error types returned by the client.

use std::path::PathBuf;

use crate::types::{TaskId, TaskStatus};

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

    /// Structured API error envelope (Tripo returns `{code, message, suggestion}`).
    #[error("API [{code}] {message}{}", suggestion.as_deref().map(|s| format!(" — {s}")).unwrap_or_default())]
    Api {
        /// API-specific error code.
        code: i32,
        /// Human-readable error message.
        message: String,
        /// Optional suggestion for how to fix the error.
        suggestion: Option<String>,
    },

    /// A task polling loop observed a non-success terminal status.
    #[error("task {0} ended with status {1:?}")]
    TaskFailed(TaskId, TaskStatus),

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
}
