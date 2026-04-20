//! `mesh-completion` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{MeshCompletionRequest, TaskRequest};

use crate::cli::GlobalArgs;
use crate::commands::variants::{IntoRequest, VariantRunOpts};

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

impl IntoRequest for MeshCompletionArgs {
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::MeshCompletion(MeshCompletionRequest {
            original_model_task_id: self.original_model_task_id,
            model_version: self.model_version,
            part_names: self.part_names,
        }))
    }
}

/// Run `mesh-completion`.
pub async fn run(g: &GlobalArgs, a: MeshCompletionArgs) -> Result<()> {
    let opts = VariantRunOpts {
        wait: a.run.wait || a.run.output.is_some(),
        output: a.run.output.clone(),
        timeout: a.run.timeout,
        poll_interval: a.run.poll_interval,
    };
    let inner = MeshCompletionArgs {
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
