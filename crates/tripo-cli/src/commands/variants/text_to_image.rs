//! `text-to-image` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::TextToImageTemplate;
use tripo_api::{TaskRequest, TextToImageRequest};

use super::image_gen::ImageGenOpts;
use super::parsers::wire_enum;
use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Generate an image from a text prompt.
#[derive(Debug, Args)]
pub struct TextToImageArgs {
    /// Prompt describing the image. Append negative prompts after `--no`.
    #[arg(long)]
    pub prompt: String,
    #[command(flatten)]
    pub image: ImageGenOpts,
    /// Add an AI-content watermark; seedream models only.
    #[arg(long)]
    pub watermark: Option<bool>,
    /// Generation template (`asset_extraction|character_completion|t_pose|variants|figure`).
    #[arg(long, value_parser = wire_enum::<TextToImageTemplate>)]
    pub template: Option<TextToImageTemplate>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for TextToImageArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::TextToImage(TextToImageRequest {
            prompt: self.prompt,
            model: self.image.model,
            size: self.image.size,
            quality: self.image.quality,
            background: self.image.background,
            aspect_ratio: self.image.aspect_ratio,
            output_format: self.image.output_format,
            watermark: self.watermark,
            template: self.template,
        }))
    }
}
