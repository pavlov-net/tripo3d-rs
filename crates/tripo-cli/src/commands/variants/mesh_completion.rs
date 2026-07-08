//! `mesh-completion` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{MeshCompletionRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Fill holes in an existing mesh.
#[derive(Debug, Args)]
pub struct MeshCompletionArgs {
    /// Model source: task id, file token, or URL.
    #[arg(long)]
    pub input: String,
    /// Model version.
    #[arg(long)]
    pub model: Option<String>,
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
            input: self.input,
            model: self.model,
            part_names: self.part_names,
        }))
    }
}
