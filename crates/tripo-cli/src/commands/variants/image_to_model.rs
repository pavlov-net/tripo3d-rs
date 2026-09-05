//! `image-to-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{
    ExportOrientation, GeometryQuality, Orientation, TextureAlignment, TextureQuality,
};
use tripo_api::{CompressionMode, ImageInput, ImageToModelRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Generate a 3D model from a single image.
#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct ImageToModelArgs {
    /// URL, `file_token`, `task_id`, or local path.
    #[arg(long)]
    pub input: String,
    /// Automatically optimize the input image before generation.
    #[arg(long)]
    pub enable_image_autofix: Option<bool>,
    /// Model version.
    #[arg(long)]
    pub model: Option<String>,
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
    /// Texture quality preset (standard|detailed|extreme).
    #[arg(long, value_parser = super::parsers::texture_quality)]
    pub texture_quality: Option<TextureQuality>,
    /// Geometry quality preset (standard|detailed).
    #[arg(long, value_parser = super::parsers::geometry_quality)]
    pub geometry_quality: Option<GeometryQuality>,
    /// Texture alignment strategy.
    #[arg(long, value_parser = super::parsers::texture_alignment)]
    pub texture_alignment: Option<TextureAlignment>,
    /// Auto-size.
    #[arg(long)]
    pub auto_size: Option<bool>,
    /// Output orientation hint.
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
    /// UV unwrapping during generation.
    #[arg(long)]
    pub export_uv: Option<bool>,
    /// Forward axis (+x|+y|-x|-y). Leave unset if post-processing; orient at final conversion.
    #[arg(long, allow_hyphen_values = true, value_parser = super::parsers::export_orientation)]
    pub export_orientation: Option<ExportOrientation>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for ImageToModelArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::ImageToModel(ImageToModelRequest {
            input: ImageInput::parse(&self.input),
            enable_image_autofix: self.enable_image_autofix,
            model: self.model,
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
            export_uv: self.export_uv,
            export_orientation: self.export_orientation,
        }))
    }
}
