//! `image-to-multiview` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{ImageInput, ImageToMultiviewRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Render front, left, back, and right views of a single image.
#[derive(Debug, Args)]
pub struct ImageToMultiviewArgs {
    /// URL, `file_token`, local path, or `task_id` of a prior image task.
    #[arg(long)]
    pub input: String,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for ImageToMultiviewArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::ImageToMultiview(ImageToMultiviewRequest {
            input: ImageInput::parse(&self.input),
        }))
    }
}
