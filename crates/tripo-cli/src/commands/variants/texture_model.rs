//! `texture-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{Quality, TextureAlignment};
use tripo_api::{CompressionMode, ImageInput, TaskRequest, TextureModelRequest, TexturePrompt};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// (Re-)texture an existing model.
#[derive(Debug, Args)]
pub struct TextureModelArgs {
    /// Source task id.
    #[arg(long)]
    pub original_model_task_id: String,
    /// Text prompt (maps into `texture_prompt.text`).
    #[arg(long)]
    pub text_prompt: Option<String>,
    /// Reference image (`URL`, `file_token`, or path); maps into `texture_prompt.image`.
    #[arg(long)]
    pub image_prompt: Option<String>,
    /// Style image (`URL`, `file_token`, or path); maps into `texture_prompt.style_image`.
    #[arg(long)]
    pub style_image: Option<String>,
    /// Model version.
    #[arg(long)]
    pub model_version: Option<String>,
    /// Generate a texture.
    #[arg(long)]
    pub texture: Option<bool>,
    /// PBR shading.
    #[arg(long)]
    pub pbr: Option<bool>,
    /// Model seed.
    #[arg(long)]
    pub model_seed: Option<i32>,
    /// Texture seed.
    #[arg(long)]
    pub texture_seed: Option<i32>,
    /// Texture quality preset.
    #[arg(long, value_parser = super::text_to_model::parse_quality)]
    pub texture_quality: Option<Quality>,
    /// Texture alignment strategy.
    #[arg(long, value_parser = super::image_to_model::__private::parse_alignment)]
    pub texture_alignment: Option<TextureAlignment>,
    /// Restrict to named parts (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub part_names: Option<Vec<String>>,
    /// Enable geometry compression.
    #[arg(long)]
    pub compress: bool,
    /// Bake textures.
    #[arg(long)]
    pub bake: Option<bool>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for TextureModelArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        let prompt = TexturePrompt {
            text: self.text_prompt,
            image: self.image_prompt.as_deref().map(ImageInput::parse),
            style_image: self.style_image.as_deref().map(ImageInput::parse),
        };
        Ok(TaskRequest::TextureModel(TextureModelRequest {
            original_model_task_id: self.original_model_task_id,
            texture_prompt: prompt,
            model_version: self.model_version,
            texture: self.texture,
            pbr: self.pbr,
            model_seed: self.model_seed,
            texture_seed: self.texture_seed,
            texture_quality: self.texture_quality,
            texture_alignment: self.texture_alignment,
            part_names: self.part_names,
            compress: self.compress.then_some(CompressionMode::Geometry),
            bake: self.bake,
        }))
    }
}
