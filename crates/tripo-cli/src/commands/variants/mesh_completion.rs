//! `mesh-completion` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{MeshCompletionRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Fill holes in an existing mesh.
#[derive(Debug, Args)]
pub struct MeshCompletionArgs {
    /// Source task id.
    #[arg(long)]
    pub original_model_task_id: String,
    /// Model version.
    #[arg(long)]
    pub model_version: Option<String>,
    /// Restrict to named parts (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub part_names: Option<Vec<String>>,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for MeshCompletionArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::MeshCompletion(MeshCompletionRequest {
            original_model_task_id: self.original_model_task_id,
            model_version: self.model_version,
            part_names: self.part_names,
        }))
    }
}
