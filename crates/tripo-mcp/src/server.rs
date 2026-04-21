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

        let mut opts = WaitOptions {
            timeout: p.timeout_seconds.map(Duration::from_secs),
            on_progress: Some(callback),
            ..Default::default()
        };
        if let Some(s) = p.max_interval_seconds {
            opts.max_interval = Duration::from_secs(s);
        }
        let task = self
            .client
            .wait_for_task(&p.task_id, opts)
            .await
            .map_err(to_error_data)?;
        Ok(Json(task))
    }

    /// Download a task's output models into a local directory.
    #[tool(
        name = "download_task_models",
        description = "Download a completed task's output models into a local directory.",
        annotations(
            title = "Download Task Models",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn download_task_models(
        &self,
        Parameters(p): Parameters<params::DownloadParams>,
    ) -> Result<Json<tripo_api::DownloadedFiles>, ErrorData> {
        let task = self
            .client
            .get_task(&p.task_id)
            .await
            .map_err(to_error_data)?;
        let opts = tripo_api::DownloadOptions {
            overwrite: p.overwrite,
            ..Default::default()
        };
        let files = self
            .client
            .download_task_models(&task, &p.output_dir, opts)
            .await
            .map_err(to_error_data)?;
        Ok(Json(files))
    }

    /// Generate a 3D model from a text prompt.
    #[tool(
        name = "text_to_model",
        description = "Generate a 3D model from a text prompt. Returns the created task id.",
        annotations(
            title = "Text \u{2192} 3D Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn text_to_model(
        &self,
        Parameters(req): Parameters<tripo_api::TextToModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::TextToModel(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Generate a 3D model from a single image.
    #[tool(
        name = "image_to_model",
        description = "Generate a 3D model from a single image reference (URL, file token, or local path).",
        annotations(
            title = "Image \u{2192} 3D Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn image_to_model(
        &self,
        Parameters(req): Parameters<tripo_api::ImageToModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::ImageToModel(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Multi-view to 3D model.
    #[tool(
        name = "multiview_to_model",
        description = "Generate a 3D model from multiple images (front/back/left/right views).",
        annotations(
            title = "Multi-view \u{2192} 3D Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn multiview_to_model(
        &self,
        Parameters(req): Parameters<tripo_api::MultiviewToModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::MultiviewToModel(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Convert a model to another file format.
    #[tool(
        name = "convert_model",
        description = "Convert a completed model to another file format.",
        annotations(
            title = "Convert Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn convert_model(
        &self,
        Parameters(req): Parameters<tripo_api::ConvertModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::ConvertModel(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Stylize a model.
    #[tool(
        name = "stylize_model",
        description = "Apply a stylization preset (lego/voxel/etc) to an existing model.",
        annotations(
            title = "Stylize Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn stylize_model(
        &self,
        Parameters(req): Parameters<tripo_api::StylizeModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::Stylize(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// (Re)texture an existing model.
    #[tool(
        name = "texture_model",
        description = "Re-texture an existing model, optionally guided by a text or image prompt.",
        annotations(
            title = "Texture Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn texture_model(
        &self,
        Parameters(req): Parameters<tripo_api::TextureModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::TextureModel(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Refine a draft model.
    #[tool(
        name = "refine_model",
        description = "Turn a draft model into a finished one.",
        annotations(
            title = "Refine Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn refine_model(
        &self,
        Parameters(req): Parameters<tripo_api::RefineModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::Refine(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Rig compatibility probe.
    #[tool(
        name = "check_riggable",
        description = "Precheck whether a model can be rigged.",
        annotations(
            title = "Check Riggable",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn check_riggable(
        &self,
        Parameters(req): Parameters<tripo_api::CheckRiggableRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::CheckRiggable(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Generate a skeletal rig.
    #[tool(
        name = "rig_model",
        description = "Generate a skeletal rig for an existing model.",
        annotations(
            title = "Rig Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn rig_model(
        &self,
        Parameters(req): Parameters<tripo_api::RigModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::Rig(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Retarget animation presets onto a rigged model.
    #[tool(
        name = "retarget_animation",
        description = "Retarget animation presets onto a rigged model. Pass `animation` (single) or `animations` (list).",
        annotations(
            title = "Retarget Animation",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn retarget_animation(
        &self,
        Parameters(req): Parameters<tripo_api::RetargetAnimationRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::Retarget(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Decompose a model into semantic parts.
    #[tool(
        name = "mesh_segmentation",
        description = "Decompose a model into semantic parts.",
        annotations(
            title = "Mesh Segmentation",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn mesh_segmentation(
        &self,
        Parameters(req): Parameters<tripo_api::MeshSegmentationRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::MeshSegmentation(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Fill holes in an existing mesh.
    #[tool(
        name = "mesh_completion",
        description = "Complete missing parts of an existing mesh.",
        annotations(
            title = "Mesh Completion",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn mesh_completion(
        &self,
        Parameters(req): Parameters<tripo_api::MeshCompletionRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::MeshCompletion(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
    }

    /// Reduce a high-poly model to a lowpoly one.
    #[tool(
        name = "smart_lowpoly",
        description = "Reduce a high-poly model to a lowpoly one.",
        annotations(
            title = "Smart Lowpoly",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn smart_lowpoly(
        &self,
        Parameters(req): Parameters<tripo_api::SmartLowpolyRequest>,
    ) -> Result<Json<params::TaskCreated>, ErrorData> {
        let id = self
            .client
            .create_task(tripo_api::tasks::TaskRequest::SmartLowpoly(req))
            .await
            .map_err(to_error_data)?;
        Ok(Json(params::TaskCreated { task_id: id }))
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

/// Map a [`tripo_api::Error`] into a JSON-RPC [`ErrorData`]. Takes by value to
/// pair directly with `Result::map_err`.
#[allow(
    clippy::needless_pass_by_value,
    reason = "by-value signature matches Result::map_err"
)]
pub(crate) fn to_error_data(err: tripo_api::Error) -> ErrorData {
    ErrorData::internal_error(err.to_string(), None)
}
