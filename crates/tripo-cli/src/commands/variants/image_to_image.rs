//! `image-to-image` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::ImageToImageTemplate;
use tripo_api::{ImageInput, ImageToImageRequest, TaskRequest};

use super::image_gen::ImageGenOpts;
use super::parsers::wire_enum;
use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Edit or combine reference images. Pass `--input` once for a single
/// reference, or repeat it to send `inputs` (`image[1]`, `image[2]`, ...).
#[derive(Debug, Args)]
pub struct ImageToImageArgs {
    /// URL, `file_token`, `task_id`, or local path. Repeat for multiple
    /// references, which the prompt addresses as `image[1]`, `image[2]`, ...
    #[arg(long, action = clap::ArgAction::Append, required = true)]
    pub input: Vec<String>,
    /// Editing instruction. Required unless `--template` is set.
    #[arg(long, required_unless_present = "template")]
    pub prompt: Option<String>,
    #[command(flatten)]
    pub image: ImageGenOpts,
    /// Editing template (`t_pose|character_completion|3d_enhance|variants|figure`).
    #[arg(long, value_parser = wire_enum::<ImageToImageTemplate>)]
    pub template: Option<ImageToImageTemplate>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for ImageToImageArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        let mut images: Vec<ImageInput> = self.input.iter().map(|s| ImageInput::parse(s)).collect();
        let (input, inputs) = if images.len() == 1 {
            (images.pop(), None)
        } else {
            (None, Some(images))
        };
        Ok(TaskRequest::ImageToImage(ImageToImageRequest {
            input,
            inputs,
            prompt: self.prompt,
            model: self.image.model,
            size: self.image.size,
            quality: self.image.quality,
            background: self.image.background,
            aspect_ratio: self.image.aspect_ratio,
            template: self.template,
            output_format: self.image.output_format,
        }))
    }
}
