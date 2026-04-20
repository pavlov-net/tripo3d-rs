//! `multiview-to-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{Orientation, Quality, TextureAlignment};
use tripo_api::{CompressionMode, ImageInput, MultiviewToModelRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Multi-view to 3D model. Pass `--image` once per view (empty slots: `--image=""`).
#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct MultiviewToModelArgs {
    /// Repeated; one per view. Empty string inserts a blank slot.
    #[arg(long, action = clap::ArgAction::Append, required = true)]
    pub image: Vec<String>,
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
    /// Model seed.
    #[arg(long)]
    pub model_seed: Option<i32>,
    /// Texture seed.
    #[arg(long)]
    pub texture_seed: Option<i32>,
    /// Texture quality preset.
    #[arg(long, value_parser = super::parsers::quality)]
    pub texture_quality: Option<Quality>,
    /// Geometry quality preset.
    #[arg(long, value_parser = super::parsers::quality)]
    pub geometry_quality: Option<Quality>,
    /// Texture alignment strategy.
    #[arg(long, value_parser = super::parsers::texture_alignment)]
    pub texture_alignment: Option<TextureAlignment>,
    /// Auto-size.
    #[arg(long)]
    pub auto_size: Option<bool>,
    /// Orientation.
    #[arg(long, value_parser = super::parsers::orientation)]
    pub orientation: Option<Orientation>,
    /// Produce a quad mesh.
    #[arg(long)]
    pub quad: Option<bool>,
    /// Enable geometry compression.
    #[arg(long)]
    pub compress: bool,
    /// Emit part decomposition.
    #[arg(long)]
    pub generate_parts: Option<bool>,
    /// Route through smart-lowpoly.
    #[arg(long)]
    pub smart_low_poly: Option<bool>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for MultiviewToModelArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        let images: Vec<Option<ImageInput>> = self
            .image
            .iter()
            .map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(ImageInput::parse(s))
                }
            })
            .collect();
        Ok(TaskRequest::MultiviewToModel(MultiviewToModelRequest {
            images,
            model_version: self.model_version,
            face_limit: self.face_limit,
            texture: self.texture,
            pbr: self.pbr,
            model_seed: self.model_seed,
            texture_seed: self.texture_seed,
            texture_quality: self.texture_quality,
            geometry_quality: self.geometry_quality,
            texture_alignment: self.texture_alignment,
            auto_size: self.auto_size,
            orientation: self.orientation,
            quad: self.quad,
            compress: self.compress.then_some(CompressionMode::Geometry),
            generate_parts: self.generate_parts,
            smart_low_poly: self.smart_low_poly,
        }))
    }
}
