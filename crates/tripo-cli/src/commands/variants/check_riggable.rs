//! `check-riggable` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{CheckRiggableRequest, TaskRequest};

use crate::cli::GlobalArgs;
use crate::commands::variants::{IntoRequest, VariantRunOpts};

/// Pre-check whether a model can be rigged.
#[derive(Debug, Args)]
pub struct CheckRiggableArgs {
    /// Source task id.
    #[arg(long)]
    pub original_model_task_id: String,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl IntoRequest for CheckRiggableArgs {
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::CheckRiggable(CheckRiggableRequest {
            original_model_task_id: self.original_model_task_id,
        }))
    }
}

/// Run `check-riggable`.
pub async fn run(g: &GlobalArgs, a: CheckRiggableArgs) -> Result<()> {
    let opts = VariantRunOpts {
        wait: a.run.wait || a.run.output.is_some(),
        output: a.run.output.clone(),
        timeout: a.run.timeout,
        poll_interval: a.run.poll_interval,
    };
    let inner = CheckRiggableArgs {
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
