//! `TripoServer` — the MCP handler.
//!
//! Tool methods are attached to this struct via `#[tool]` and aggregated by
//! `#[tool_router]`. `#[tool_handler]` then fills in the `list_tools` and
//! `call_tool` hooks on `impl ServerHandler`.
//!
//! Tool execution failures (API errors, failed validation, I/O) come back as
//! a [`CallToolResult`] with `is_error: true` via [`ToolError`], so the model
//! can read the message. JSON-RPC errors are left to rmcp for protocol
//! problems such as malformed arguments.

use std::{path::Path, sync::Arc};

use rmcp::{
    ErrorData, Json, RoleServer, ServerHandler,
    handler::server::{tool::IntoCallToolResult, wrapper::Parameters},
    model::{
        CallToolResponse, CallToolResult, ContentBlock, Implementation, ProgressNotificationParam,
        ServerCapabilities, ServerConfig,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
};

use crate::params;
use tripo_api::{Client, TaskRequest};

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

    /// Submit a typed task request and return the created task id.
    async fn submit(&self, req: TaskRequest) -> Result<Json<params::TaskCreated>, ToolError> {
        let task_id = self.client.create_task(req).await?;
        Ok(Json(params::TaskCreated { task_id }))
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
    async fn get_balance(&self) -> Result<Json<tripo_api::Balance>, ToolError> {
        Ok(Json(self.client.get_balance().await?))
    }

    /// Per-task credit usage history.
    #[tool(
        name = "get_usage",
        description = "List per-task credit consumption for the Tripo account. Optional limit/offset page through the history.",
        annotations(
            title = "Account Usage",
            read_only_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    async fn get_usage(
        &self,
        Parameters(q): Parameters<tripo_api::UsageQuery>,
    ) -> Result<Json<params::Usage>, ToolError> {
        let records = self.client.get_usage(q).await?;
        Ok(Json(params::Usage { records }))
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
    ) -> Result<Json<tripo_api::Task>, ToolError> {
        Ok(Json(self.client.get_task(&p.task_id).await?))
    }

    /// Fetch several tasks' current state in one request.
    #[tool(
        name = "list_tasks",
        description = "Fetch the current state of up to 100 Tripo tasks in one request. Ids the server does not know are returned in `missed`.",
        annotations(
            title = "List Tasks",
            read_only_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    async fn list_tasks(
        &self,
        Parameters(p): Parameters<params::ListTasksParams>,
    ) -> Result<Json<tripo_api::TaskList>, ToolError> {
        Ok(Json(self.client.list_tasks(&p.task_ids).await?))
    }

    /// Upload a local file; returns a `file_token` usable as `ImageInput::FileToken`.
    #[tool(
        name = "upload_file",
        description = "Upload a local file to Tripo and return a file token usable as an image or model input. Files over 60 MiB go through a presigned storage URL; set `presign` to use it for any size.",
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
    ) -> Result<Json<tripo_api::UploadedFile>, ToolError> {
        let up = if p.presign {
            self.client.upload_file_presigned(&p.path).await
        } else {
            self.client.upload_file(&p.path).await
        }
        .map_err(|e| ToolError::with_path(e, &p.path))?;
        Ok(Json(up))
    }

    /// Submit an arbitrary JSON body to a task-creation endpoint.
    /// Forward-compatibility escape hatch for variants not in the typed surface.
    #[tool(
        name = "create_raw_task",
        description = "Submit a raw JSON task body to a v3 task-creation endpoint (e.g. generation/text-to-model). Use when a variant isn't in the typed surface.",
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        let task_id = self.client.create_task_raw(&p.endpoint, &p.body).await?;
        Ok(Json(params::TaskCreated { task_id }))
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
    ) -> Result<Json<tripo_api::Task>, ToolError> {
        use std::time::Duration;
        use tripo_api::WaitOptions;

        let on_progress = ctx.meta.get_progress_token().map(|token| {
            let peer = ctx.peer.clone();
            Box::new(move |task: &tripo_api::Task| {
                let pct = f64::from(task.progress.clamp(0, 100));
                let message = format!("{:?} ({pct:.0}%)", task.status);
                let param = ProgressNotificationParam::new(token.clone(), pct)
                    .with_total(100.0)
                    .with_message(message);
                let peer = peer.clone();
                tokio::spawn(async move {
                    let _ = peer.notify_progress(param).await;
                });
            }) as tripo_api::ProgressCallback
        });

        let mut opts = WaitOptions {
            timeout: p.timeout_seconds.map(Duration::from_secs),
            on_progress,
            ..Default::default()
        };
        if let Some(s) = p.max_interval_seconds {
            opts.max_interval = Duration::from_secs(s);
        }
        Ok(Json(self.client.wait_for_task(&p.task_id, opts).await?))
    }

    /// Download a task's output files into a local directory.
    #[tool(
        name = "download_task_models",
        description = "Download a completed task's output files (models, generated images, multiview views) into a local directory.",
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
    ) -> Result<Json<tripo_api::DownloadedFiles>, ToolError> {
        let task = self.client.get_task(&p.task_id).await?;
        let opts = tripo_api::DownloadOptions {
            overwrite: p.overwrite,
            ..Default::default()
        };
        let files = self
            .client
            .download_task_models(&task, &p.output_dir, opts)
            .await
            .map_err(|e| ToolError::with_path(e, &p.output_dir))?;
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::TextToModel(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::ImageToModel(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::MultiviewToModel(req)).await
    }

    /// Generate an image from a text prompt.
    #[tool(
        name = "text_to_image",
        description = "Generate an image from a text prompt. Models include seedream, banana, and chat_image (GPT Image 2/2.5). Result: output.generated_image_url.",
        annotations(
            title = "Text \u{2192} Image",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn text_to_image(
        &self,
        Parameters(req): Parameters<tripo_api::TextToImageRequest>,
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::TextToImage(req)).await
    }

    /// Edit or combine reference images.
    #[tool(
        name = "image_to_image",
        description = "Edit or combine reference images (input and/or inputs: URL, file token, task id, or local path). prompt is required unless template is set. Result: output.generated_image_url.",
        annotations(
            title = "Image \u{2192} Image",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn image_to_image(
        &self,
        Parameters(req): Parameters<tripo_api::ImageToImageRequest>,
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::ImageToImage(req)).await
    }

    /// Render four views of a single image.
    #[tool(
        name = "image_to_multiview",
        description = "Render front, left, back, and right views of a single image (URL, file token, local path, or task id of a prior text_to_image/image_to_image task). Results: output.{front,left,back,right}_view_url.",
        annotations(
            title = "Image \u{2192} Multiview",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn image_to_multiview(
        &self,
        Parameters(req): Parameters<tripo_api::ImageToMultiviewRequest>,
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::ImageToMultiview(req)).await
    }

    /// Apply per-view edits to a multiview image.
    #[tool(
        name = "edit_multiview",
        description = "Edit a multiview image (task id of an image_to_multiview or edit_multiview task; file token, URL, or local path are documented but the service currently accepts only a task id) with at most one prompt per view: front/left/back/right.",
        annotations(
            title = "Edit Multiview",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn edit_multiview(
        &self,
        Parameters(req): Parameters<tripo_api::EditMultiviewRequest>,
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::EditMultiview(req)).await
    }

    /// Generate a Gaussian Splat from a single image.
    #[tool(
        name = "image_to_splat",
        description = "Generate a 3D Gaussian Splat (.splat) from a single image reference (URL, file token, or local path). Fixed cost: 30 credits.",
        annotations(
            title = "Image \u{2192} Gaussian Splat",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn image_to_splat(
        &self,
        Parameters(req): Parameters<tripo_api::ImageToSplatRequest>,
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::ImageToSplat(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::ConvertModel(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::Stylize(req)).await
    }

    /// (Re)texture an existing model.
    #[tool(
        name = "texture_model",
        description = "Re-texture an existing model, optionally guided by a text prompt (plus style image), one reference image, or exactly 4 reference images [front, left, back, right]. Texture model v3.5 adds texture_quality fast and delight.",
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::TextureModel(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::Refine(req)).await
    }

    /// Import an external model file.
    #[tool(
        name = "import_model",
        description = "Import an external model file (GLB/GLTF/FBX/OBJ/STL, up to 150 MB; URL, file token, or local path) so other model tasks can use it.",
        annotations(
            title = "Import Model",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn import_model(
        &self,
        Parameters(req): Parameters<tripo_api::ImportModelRequest>,
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::ImportModel(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::CheckRiggable(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::Rig(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::Retarget(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::MeshSegmentation(req)).await
    }

    /// Segment an image or GLB into parts, including auto modeling.
    #[tool(
        name = "mesh_smart_segment",
        description = "Segment an image or GLB into parts, including auto modeling. seg_type=model takes a GLB and requires a 16-number column-major transform.",
        annotations(
            title = "Smart Segmentation",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn mesh_smart_segment(
        &self,
        Parameters(req): Parameters<tripo_api::MeshSmartSegmentRequest>,
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::MeshSmartSegment(req)).await
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
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::MeshCompletion(req)).await
    }

    /// Reduce model polycount (retopology).
    #[tool(
        name = "mesh_decimate",
        description = "Reduce model polycount: smart retopology (model v2.0, default) or basic decimation (v1.0).",
        annotations(
            title = "Mesh Decimate",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true,
        )
    )]
    async fn mesh_decimate(
        &self,
        Parameters(req): Parameters<tripo_api::MeshDecimateRequest>,
    ) -> Result<Json<params::TaskCreated>, ToolError> {
        self.submit(TaskRequest::MeshDecimate(req)).await
    }
}

#[tool_handler]
impl ServerHandler for TripoServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Tools for submitting, polling, downloading, and managing Tripo 3D and image generation tasks.",
            )
    }
}

/// A tool execution failure, reported to the model as a [`CallToolResult`]
/// with `is_error: true` and the message as text content.
#[derive(Debug)]
pub struct ToolError(String);

impl ToolError {
    /// Like `From<tripo_api::Error>`, but names `path` in I/O errors, whose
    /// messages (e.g. "No such file or directory") omit it.
    fn with_path(err: tripo_api::Error, path: &Path) -> Self {
        match err {
            tripo_api::Error::Io(e) => Self(format!("{}: {e}", path.display())),
            other => other.into(),
        }
    }
}

impl From<tripo_api::Error> for ToolError {
    fn from(err: tripo_api::Error) -> Self {
        Self(err.to_string())
    }
}

impl IntoCallToolResult for ToolError {
    fn into_call_tool_result(self) -> Result<CallToolResponse, ErrorData> {
        Ok(CallToolResult::error(vec![ContentBlock::text(self.0)]).into())
    }
}
