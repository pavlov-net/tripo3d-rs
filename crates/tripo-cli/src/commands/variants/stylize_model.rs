//! `stylize-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::PostStyle;
use tripo_api::{StylizeModelRequest, TaskRequest};

use crate::cli::GlobalArgs;
use crate::commands::variants::{IntoRequest, VariantRunOpts};

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

impl IntoRequest for StylizeModelArgs {
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::Stylize(StylizeModelRequest {
            original_model_task_id: self.original_model_task_id,
            style: self.style,
            block_size: self.block_size,
        }))
    }
}

/// Run `stylize-model`.
pub async fn run(g: &GlobalArgs, a: StylizeModelArgs) -> Result<()> {
    let opts = VariantRunOpts {
        wait: a.run.wait || a.run.output.is_some(),
        output: a.run.output.clone(),
        timeout: a.run.timeout,
        poll_interval: a.run.poll_interval,
    };
    let inner = StylizeModelArgs {
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
