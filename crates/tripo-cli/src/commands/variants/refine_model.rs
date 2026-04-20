//! `refine-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{RefineModelRequest, TaskRequest};

use crate::cli::GlobalArgs;
use crate::commands::variants::{IntoRequest, VariantRunOpts};

/// Refine a draft model into a finished one.
#[derive(Debug, Args)]
pub struct RefineModelArgs {
    /// Draft (pre-refinement) task id.
    #[arg(long)]
    pub draft_model_task_id: String,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl IntoRequest for RefineModelArgs {
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::Refine(RefineModelRequest {
            draft_model_task_id: self.draft_model_task_id,
        }))
    }
}

/// Run `refine-model`.
pub async fn run(g: &GlobalArgs, a: RefineModelArgs) -> Result<()> {
    let opts = VariantRunOpts {
        wait: a.run.wait || a.run.output.is_some(),
        output: a.run.output.clone(),
        timeout: a.run.timeout,
        poll_interval: a.run.poll_interval,
    };
    let inner = RefineModelArgs {
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
