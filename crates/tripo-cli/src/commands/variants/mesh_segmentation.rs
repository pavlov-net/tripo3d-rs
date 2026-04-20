//! `mesh-segmentation` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{MeshSegmentationRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

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

impl VariantArgs for MeshSegmentationArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::MeshSegmentation(MeshSegmentationRequest {
            original_model_task_id: self.original_model_task_id,
            model_version: self.model_version,
        }))
    }
}
