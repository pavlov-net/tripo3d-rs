//! `stylize-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::PostStyle;
use tripo_api::{StylizeModelRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Apply a stylization preset to an existing model.
#[derive(Debug, Args)]
pub struct StylizeModelArgs {
    /// Source task id.
    #[arg(long)]
    pub original_model_task_id: String,
    /// Stylization preset.
    #[arg(long, value_parser = parse_style)]
    pub style: PostStyle,
    /// Voxel block size (for voxel/minecraft styles).
    #[arg(long)]
    pub block_size: Option<i32>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

fn parse_style(s: &str) -> Result<PostStyle, String> {
    Ok(match s {
        "lego" => PostStyle::Lego,
        "voxel" => PostStyle::Voxel,
        "voronoi" => PostStyle::Voronoi,
        "minecraft" => PostStyle::Minecraft,
        o => return Err(format!("invalid style `{o}`")),
    })
}

impl VariantArgs for StylizeModelArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::Stylize(StylizeModelRequest {
            original_model_task_id: self.original_model_task_id,
            style: self.style,
            block_size: self.block_size,
        }))
    }
}
