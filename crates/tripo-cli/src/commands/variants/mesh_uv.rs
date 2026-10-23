//! `mesh-uv` subcommand (Smart UV).

use anyhow::Result;
use clap::Args;
use tripo_api::{CompressionMode, ExportOrientation, ImageInput, MeshUvRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Re-unwrap a model's UVs, remapping existing textures, rigging, and
/// animation.
#[derive(Debug, Args)]
pub struct MeshUvArgs {
    /// Model source: task id, file token, URL, or local path. At most 80,000
    /// equivalent triangles (a quad counts as two); run `mesh-decimate` first
    /// on denser models.
    #[arg(long)]
    pub input: String,
    /// Model version (the server documents only `default`).
    #[arg(long)]
    pub model: Option<String>,
    /// Bake existing textures onto the new UVs (server default true); false
    /// returns geometry with UVs only.
    #[arg(long)]
    pub bake: Option<bool>,
    /// Export forward axis (+x|-x|+y|-y).
    #[arg(long, allow_hyphen_values = true, value_parser = super::parsers::export_orientation)]
    pub export_orientation: Option<ExportOrientation>,
    /// Enable geometry compression.
    #[arg(long)]
    pub compress: bool,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for MeshUvArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::MeshUv(MeshUvRequest {
            input: ImageInput::parse(&self.input),
            model: self.model,
            bake: self.bake,
            export_orientation: self.export_orientation,
            compress: self.compress.then_some(CompressionMode::Geometry),
        }))
    }
}
