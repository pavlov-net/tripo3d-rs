//! `mesh-segmentation` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{MeshSegmentationRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Decompose a model into semantic parts.
#[derive(Debug, Args)]
pub struct MeshSegmentationArgs {
    /// Model source: task id, file token, or URL.
    #[arg(long)]
    pub input: String,
    /// Model version.
    #[arg(long)]
    pub model: Option<String>,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for MeshSegmentationArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::MeshSegmentation(MeshSegmentationRequest {
            input: self.input,
            model: self.model,
        }))
    }
}
