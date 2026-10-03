//! `texture-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{TextureAlignment, TextureQuality};
use tripo_api::{CompressionMode, ImageInput, TaskRequest, TextureModelRequest, TexturePrompt};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// (Re-)texture an existing model.
#[derive(Debug, Args)]
pub struct TextureModelArgs {
    /// Model source: task id, file token, or URL.
    #[arg(long)]
    pub input: String,
    /// Text prompt (maps into `texture_prompt.text`).
    #[arg(long)]
    pub text_prompt: Option<String>,
    /// Reference image (`URL`, `file_token`, or path); maps into `texture_prompt.image`.
    #[arg(long)]
    pub image_prompt: Option<String>,
    /// Repeated; exactly 4 reference images (`URL`, `file_token`, or path) in
    /// order [front, left, back, right]; maps into `texture_prompt.images`.
    #[arg(long, action = clap::ArgAction::Append)]
    pub images_prompt: Vec<String>,
    /// Style image (`URL`, `file_token`, or path); maps into `texture_prompt.style_image`.
    #[arg(long)]
    pub style_image: Option<String>,
    /// Texture model version (see `tripo_api::versions::texture`); fast
    /// quality requires v3.5.
    #[arg(long)]
    pub model: Option<String>,
    /// PBR shading.
    #[arg(long)]
    pub pbr: Option<bool>,
    /// Texture seed.
    #[arg(long)]
    pub texture_seed: Option<i32>,
    /// Texture quality preset (fast|standard|detailed|extreme).
    #[arg(long, value_parser = super::parsers::texture_quality)]
    pub texture_quality: Option<TextureQuality>,
    /// Texture alignment strategy.
    #[arg(long, value_parser = super::parsers::texture_alignment)]
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
    /// Remove baked-in lighting before texturing (texture v3.5 only).
    #[arg(long)]
    pub delight: Option<bool>,

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
            images: (!self.images_prompt.is_empty()).then(|| {
                self.images_prompt
                    .iter()
                    .map(|s| ImageInput::parse(s))
                    .collect()
            }),
            style_image: self.style_image.as_deref().map(ImageInput::parse),
        };
        Ok(TaskRequest::TextureModel(TextureModelRequest {
            input: self.input,
            texture_prompt: prompt,
            model: self.model,
            pbr: self.pbr,
            texture_seed: self.texture_seed,
            texture_quality: self.texture_quality,
            texture_alignment: self.texture_alignment,
            part_names: self.part_names,
            compress: self.compress.then_some(CompressionMode::Geometry),
            bake: self.bake,
            delight: self.delight,
        }))
    }
}
