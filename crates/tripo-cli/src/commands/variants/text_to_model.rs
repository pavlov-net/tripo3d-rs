//! `text-to-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::Quality;
use tripo_api::{CompressionMode, TaskRequest, TextToModelRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Generate a 3D model from a text prompt.
#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)] // CLI flag struct.
pub struct TextToModelArgs {
    /// Prompt describing the desired output.
    #[arg(long)]
    pub prompt: String,
    /// Negative prompt (things to avoid).
    #[arg(long)]
    pub negative_prompt: Option<String>,
    /// Model version.
    #[arg(long)]
    pub model_version: Option<String>,
    /// Target face count.
    #[arg(long)]
    pub face_limit: Option<i32>,
    /// Generate a texture.
    #[arg(long)]
    pub texture: Option<bool>,
    /// PBR shading.
    #[arg(long)]
    pub pbr: Option<bool>,
    /// Seed for the initial reference image.
    #[arg(long)]
    pub image_seed: Option<i32>,
    /// Seed for model generation.
    #[arg(long)]
    pub model_seed: Option<i32>,
    /// Seed for texture generation.
    #[arg(long)]
    pub texture_seed: Option<i32>,
    /// Texture quality preset.
    #[arg(long, value_parser = super::parsers::quality)]
    pub texture_quality: Option<Quality>,
    /// Geometry quality preset.
    #[arg(long, value_parser = super::parsers::quality)]
    pub geometry_quality: Option<Quality>,
    /// Auto-size the output mesh.
    #[arg(long)]
    pub auto_size: Option<bool>,
    /// Produce a quad mesh.
    #[arg(long)]
    pub quad: Option<bool>,
    /// Enable geometry compression.
    #[arg(long)]
    pub compress: bool,
    /// Emit part decomposition.
    #[arg(long)]
    pub generate_parts: Option<bool>,
    /// Route through smart-lowpoly after generation.
    #[arg(long)]
    pub smart_low_poly: Option<bool>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for TextToModelArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::TextToModel(TextToModelRequest {
            prompt: self.prompt,
            negative_prompt: self.negative_prompt,
            model_version: self.model_version,
            face_limit: self.face_limit,
            texture: self.texture,
            pbr: self.pbr,
            image_seed: self.image_seed,
            model_seed: self.model_seed,
            texture_seed: self.texture_seed,
            texture_quality: self.texture_quality,
            geometry_quality: self.geometry_quality,
            auto_size: self.auto_size,
            quad: self.quad,
            compress: self.compress.then_some(CompressionMode::Geometry),
            generate_parts: self.generate_parts,
            smart_low_poly: self.smart_low_poly,
        }))
    }
}
