//! `refine-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{RefineModelRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Refine a draft model into a finished one.
#[derive(Debug, Args)]
pub struct RefineModelArgs {
    /// Draft (pre-refinement) task id.
    #[arg(long)]
    pub draft_model_task_id: String,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for RefineModelArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::Refine(RefineModelRequest {
            draft_model_task_id: self.draft_model_task_id,
        }))
    }
}
