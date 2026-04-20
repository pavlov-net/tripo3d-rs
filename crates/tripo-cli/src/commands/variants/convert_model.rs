//! `convert-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{ExportOrientation, FbxPreset, OutputFormat, TextureFormat};
use tripo_api::{ConvertModelRequest, TaskRequest};

use crate::cli::GlobalArgs;
use crate::commands::variants::{IntoRequest, VariantRunOpts};

/// Convert a model to a different file format.
#[derive(Debug, Args)]
pub struct ConvertModelArgs {
    /// Source task id.
    #[arg(long)]
    pub original_model_task_id: String,
    /// Output mesh format.
    #[arg(long, value_parser = parse_format)]
    pub format: OutputFormat,
    /// Quad output.
    #[arg(long)]
    pub quad: Option<bool>,
    /// Enforce mirror symmetry.
    #[arg(long)]
    pub force_symmetry: Option<bool>,
    /// Face count limit.
    #[arg(long)]
    pub face_limit: Option<i32>,
    /// Flatten the model bottom.
    #[arg(long)]
    pub flatten_bottom: Option<bool>,
    /// Flatten threshold (meters).
    #[arg(long)]
    pub flatten_bottom_threshold: Option<f64>,
    /// Texture resolution (pixels per side).
    #[arg(long)]
    pub texture_size: Option<i32>,
    /// Texture format.
    #[arg(long, value_parser = parse_texture_format)]
    pub texture_format: Option<TextureFormat>,
    /// Uniform scale factor.
    #[arg(long)]
    pub scale_factor: Option<f64>,
    /// Reset pivot to bottom-center of bbox.
    #[arg(long)]
    pub pivot_to_center_bottom: Option<bool>,
    /// Include animation data.
    #[arg(long)]
    pub with_animation: Option<bool>,
    /// Pack UVs.
    #[arg(long)]
    pub pack_uv: Option<bool>,
    /// Bake textures.
    #[arg(long)]
    pub bake: Option<bool>,
    /// Restrict to named parts (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub part_names: Option<Vec<String>>,
    /// Export per-vertex colors.
    #[arg(long)]
    pub export_vertex_colors: Option<bool>,
    /// FBX target preset.
    #[arg(long, value_parser = parse_fbx_preset)]
    pub fbx_preset: Option<FbxPreset>,
    /// Output axis orientation.
    #[arg(long, value_parser = parse_export_orientation)]
    pub export_orientation: Option<ExportOrientation>,
    /// Keep animated character in place.
    #[arg(long)]
    pub animate_in_place: Option<bool>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

fn parse_format(s: &str) -> Result<OutputFormat, String> {
    Ok(match s.to_ascii_uppercase().as_str() {
        "GLTF" => OutputFormat::Gltf,
        "USDZ" => OutputFormat::Usdz,
        "FBX" => OutputFormat::Fbx,
        "OBJ" => OutputFormat::Obj,
        "STL" => OutputFormat::Stl,
        "3MF" => OutputFormat::ThreeMf,
        o => return Err(format!("invalid format `{o}`")),
    })
}

fn parse_texture_format(s: &str) -> Result<TextureFormat, String> {
    Ok(match s.to_ascii_uppercase().as_str() {
        "BMP" => TextureFormat::Bmp,
        "DPX" => TextureFormat::Dpx,
        "HDR" => TextureFormat::Hdr,
        "JPEG" => TextureFormat::Jpeg,
        "OPEN_EXR" => TextureFormat::OpenExr,
        "PNG" => TextureFormat::Png,
        "TARGA" => TextureFormat::Targa,
        "TIFF" => TextureFormat::Tiff,
        "WEBP" => TextureFormat::Webp,
        o => return Err(format!("invalid texture format `{o}`")),
    })
}

fn parse_fbx_preset(s: &str) -> Result<FbxPreset, String> {
    Ok(match s {
        "blender" => FbxPreset::Blender,
        "mixamo" => FbxPreset::Mixamo,
        "3dsmax" => FbxPreset::ThreeDsMax,
        o => return Err(format!("invalid fbx preset `{o}`")),
    })
}

fn parse_export_orientation(s: &str) -> Result<ExportOrientation, String> {
    Ok(match s {
        "+x" => ExportOrientation::PlusX,
        "+y" => ExportOrientation::PlusY,
        "-x" => ExportOrientation::MinusX,
        "-y" => ExportOrientation::MinusY,
        o => return Err(format!("invalid orientation `{o}`")),
    })
}

impl IntoRequest for ConvertModelArgs {
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::ConvertModel(ConvertModelRequest {
            original_model_task_id: self.original_model_task_id,
            format: self.format,
            quad: self.quad,
            force_symmetry: self.force_symmetry,
            face_limit: self.face_limit,
            flatten_bottom: self.flatten_bottom,
            flatten_bottom_threshold: self.flatten_bottom_threshold,
            texture_size: self.texture_size,
            texture_format: self.texture_format,
            scale_factor: self.scale_factor,
            pivot_to_center_bottom: self.pivot_to_center_bottom,
            with_animation: self.with_animation,
            pack_uv: self.pack_uv,
            bake: self.bake,
            part_names: self.part_names,
            export_vertex_colors: self.export_vertex_colors,
            fbx_preset: self.fbx_preset,
            export_orientation: self.export_orientation,
            animate_in_place: self.animate_in_place,
        }))
    }
}

/// Run `convert-model`.
pub async fn run(g: &GlobalArgs, a: ConvertModelArgs) -> Result<()> {
    let opts = VariantRunOpts {
        wait: a.run.wait || a.run.output.is_some(),
        output: a.run.output.clone(),
        timeout: a.run.timeout,
        poll_interval: a.run.poll_interval,
    };
    let inner = ConvertModelArgs {
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
