//! `image-to-splat` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{ImageInput, ImageToSplatRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Generate a 3D Gaussian Splat (`.splat`) from a single image.
#[derive(Debug, Args)]
pub struct ImageToSplatArgs {
    /// URL, `file_token`, or local path (PNG/JPEG/WebP).
    #[arg(long)]
    pub input: String,
    /// Model seed.
    #[arg(long)]
    pub model_seed: Option<i32>,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for ImageToSplatArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::ImageToSplat(ImageToSplatRequest {
            input: ImageInput::parse(&self.input),
            model_seed: self.model_seed,
        }))
    }
}
