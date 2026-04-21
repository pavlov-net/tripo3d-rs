//! `TripoServer` — the MCP handler.
//!
//! Tool methods are attached to this struct via `#[tool]` and aggregated by
//! `#[tool_router]`. `#[tool_handler]` then fills in the `list_tools` and
//! `call_tool` hooks on `impl ServerHandler`.

use std::sync::Arc;

use rmcp::{
    ErrorData, Json, RoleServer, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{
        Implementation, ProgressNotificationParam, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
};

use crate::params;
use tripo_api::Client;

#[derive(Clone)]
pub struct TripoServer {
    pub client: Arc<Client>,
}

impl TripoServer {
    /// Build a server around an already-configured [`Client`].
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

#[tool_router]
impl TripoServer {
    /// Get the account balance.
    #[tool(
        name = "get_balance",
        description = "Get the current Tripo account balance.",
        annotations(
            title = "Account Balance",
            read_only_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    async fn get_balance(&self) -> Result<Json<tripo_api::Balance>, ErrorData> {
        let bal = self.client.get_balance().await.map_err(to_error_data)?;
        Ok(Json(bal))
    }

    /// Fetch a task's current state.
    #[tool(
        name = "get_task",
        description = "Fetch the current state of a Tripo task by id.",
        annotations(
            title = "Get Task",
            read_only_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    async fn get_task(
        &self,
        Parameters(p): Parameters<params::GetTaskParams>,
    ) -> Result<Json<tripo_api::Task>, ErrorData> {
        let task = self
            .client
            .get_task(&p.task_id)
            .await
            .map_err(to_error_data)?;
        Ok(Json(task))
    }

    /// Upload a local file; returns a `file_token` usable as `ImageInput::FileToken`.
    #[tool(
        name = "upload_file",
        description = "Upload a local file to Tripo and return a file token usable as an image reference.",
        annotations(
            title = "Upload File",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn upload_file(
        &self,
        Parameters(p): Parameters<params::UploadParams>,
    ) -> Result<Json<tripo_api::UploadedFile>, ErrorData> {
        let up = self
            .client
            .upload_file(&p.path)
            .await
            .map_err(to_error_data)?;
        Ok(Json(up))
    }

    /// Submit an arbitrary JSON body to `POST /task`. Forward-compatibility
    /// escape hatch for variants not in the typed surface.
    #[tool(
        name = "create_raw_task",
        description = "Submit a raw JSON task body to POST /task. Use when a variant isn't in the typed surface.",
        annotations(
            title = "Create Task (raw)",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn create_raw_task(
        &self,
        Parameters(p): Parameters<params::RawTaskParams>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task_raw(&p.body)
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Poll a task until it reaches a terminal status, streaming progress.
    #[tool(
        name = "wait_for_task",
        description = "Poll a task until it reaches a terminal status. Streams MCP progress notifications when the caller sets a progressToken.",
        annotations(
            title = "Wait for Task",
            read_only_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    async fn wait_for_task(
        &self,
        Parameters(p): Parameters<params::WaitParams>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Json<tripo_api::Task>, ErrorData> {
        use std::time::Duration;
        use tripo_api::WaitOptions;

        let progress_token = ctx.meta.get_progress_token();
        let peer = ctx.peer.clone();

        let callback: tripo_api::ProgressCallback = if let Some(token) = progress_token {
            Box::new(move |task: &tripo_api::Task| {
                let pct = f64::from(task.progress.clamp(0, 100));
                let message = format!("{:?} ({pct:.0}%)", task.status);
                let param = ProgressNotificationParam {
                    progress_token: token.clone(),
                    progress: pct,
                    total: Some(100.0),
                    message: Some(message),
                };
                let peer = peer.clone();
                tokio::spawn(async move {
                    let _ = peer.notify_progress(param).await;
                });
            })
        } else {
            Box::new(|_task: &tripo_api::Task| {})
        };

        let opts = WaitOptions {
            timeout: p.timeout_seconds.map(Duration::from_secs),
            max_interval: p
                .max_interval_seconds
                .map_or_else(|| Duration::from_secs(30), Duration::from_secs),
            on_progress: Some(callback),
            ..Default::default()
        };
        let task = self
            .client
            .wait_for_task(&p.task_id, opts)
            .await
            .map_err(to_error_data)?;
        Ok(Json(task))
    }
}

#[tool_handler]
impl ServerHandler for TripoServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_06_18)
            .with_server_info(Implementation::from_build_env())
            .with_instructions(
                "Tools for submitting, polling, downloading, and managing Tripo 3D generation tasks."
                    .to_string(),
            )
    }
}

/// Map a [`tripo_api::Error`] into a JSON-RPC [`ErrorData`] suitable for
/// returning from a tool method.
pub(crate) fn to_error_data(err: tripo_api::Error) -> ErrorData {
    match err {
        tripo_api::Error::Api {
            code,
            message,
            suggestion,
        } => {
            let text = suggestion.map_or_else(
                || format!("[{code}] {message}"),
                |s| format!("[{code}] {message} — {s}"),
            );
            ErrorData::internal_error(text, None)
        }
        other => ErrorData::internal_error(other.to_string(), None),
    }
}
