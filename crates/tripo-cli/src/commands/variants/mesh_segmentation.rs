//! `mesh-segmentation` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{MeshSegmentationRequest, TaskRequest};

use crate::cli::GlobalArgs;
use crate::commands::variants::{IntoRequest, VariantRunOpts};

/// Decompose a model into semantic parts.
#[derive(Debug, Args)]
pub struct MeshSegmentationArgs {
    /// Source task id.
    #[arg(long)]
    pub original_model_task_id: String,
    /// Model version.
    #[arg(long)]
    pub model_version: Option<String>,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl IntoRequest for MeshSegmentationArgs {
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::MeshSegmentation(MeshSegmentationRequest {
            original_model_task_id: self.original_model_task_id,
            model_version: self.model_version,
        }))
    }
}

/// Run `mesh-segmentation`.
pub async fn run(g: &GlobalArgs, a: MeshSegmentationArgs) -> Result<()> {
    let opts = VariantRunOpts {
        wait: a.run.wait || a.run.output.is_some(),
        output: a.run.output.clone(),
        timeout: a.run.timeout,
        poll_interval: a.run.poll_interval,
    };
    let inner = MeshSegmentationArgs {
        run: VariantRunOpts {
            wait: false,
            output: None,
            timeout: None,
            poll_interval: None,
        },
        ..a
    };
    super::run_variant(g, opts, inner).await
}
