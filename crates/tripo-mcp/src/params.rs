//! Parameter structs for non-variant tools.

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tripo_api::TaskId;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetTaskParams {
    /// Task identifier.
    pub task_id: TaskId,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct WaitParams {
    /// Task identifier.
    pub task_id: TaskId,
    /// Overall timeout in seconds.
    pub timeout_seconds: Option<u64>,
    /// Cap on the polling interval in seconds.
    pub max_interval_seconds: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DownloadParams {
    /// Task identifier.
    pub task_id: TaskId,
    /// Absolute path to an output directory.
    pub output_dir: PathBuf,
    /// Overwrite existing files.
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UploadParams {
    /// Absolute path to a local file.
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RawTaskParams {
    /// Raw JSON body to POST to `/task`. Use this for variants the SDK doesn't cover yet.
    pub body: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TaskCreated {
    /// Identifier of the newly created task.
    pub task_id: TaskId,
}
