//! `check-riggable` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{CheckRiggableRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Pre-check whether a model can be rigged.
#[derive(Debug, Args)]
pub struct CheckRiggableArgs {
    /// Model source: task id, file token, or URL.
    #[arg(long)]
    pub input: String,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for CheckRiggableArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::CheckRiggable(CheckRiggableRequest {
            input: self.input,
        }))
    }
}
