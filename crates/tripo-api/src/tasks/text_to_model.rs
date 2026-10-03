//! `text_to_model` task variant. Endpoint: `POST /generation/text-to-model`.

use serde::{Deserialize, Serialize};

use crate::compress::CompressionMode;
use crate::enums::{ExportOrientation, GeometryQuality, TextureQuality};
use crate::error::Result;

/// Request body for `POST /generation/text-to-model`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct TextToModelRequest {
    /// Prompt text. Required.
    pub prompt: String,
    /// Negative prompt (things to avoid).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// AI model version string; see `versions::text_image`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Target face count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_limit: Option<i32>,
    /// Generate a texture? (default true server-side)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<bool>,
    /// Physically-based shading?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pbr: Option<bool>,
    /// Seed for the initial reference image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_seed: Option<i32>,
    /// Seed for the model generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_seed: Option<i32>,
    /// Seed for texture generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_seed: Option<i32>,
    /// Texture quality preset; see [`TextureQuality::Fast`] for `fast`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_quality: Option<TextureQuality>,
    /// Texture model version, independent of `model`; see [`crate::versions::texture`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_version: Option<String>,
    /// Strip baked-in lighting before texturing (v3.5 texture only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delight: Option<bool>,
    /// Geometry quality preset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry_quality: Option<GeometryQuality>,
    /// Auto-size the output mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_size: Option<bool>,
    /// Produce a quad mesh (subdivision-ready).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad: Option<bool>,
    /// Geometry compression option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<CompressionMode>,
    /// Emit part decomposition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_parts: Option<bool>,
    /// Route through the smart-lowpoly pipeline after generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smart_low_poly: Option<bool>,
    /// UV unwrapping during generation (default true server-side).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_uv: Option<bool>,
    /// Forward axis for this generation only (server default: `+x`).
    /// Leave unset when post-processing; set orientation in the final convert
    /// step instead to avoid incorrectly oriented downstream results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_orientation: Option<ExportOrientation>,
}

impl TextToModelRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        super::validate_generation(&super::GenerationParams {
            model: self.model.as_deref(),
            quad: self.quad,
            face_limit: self.face_limit,
            smart_low_poly: self.smart_low_poly,
            generate_parts: self.generate_parts,
            geometry_quality: self.geometry_quality.as_ref(),
            texture_quality: self.texture_quality.as_ref(),
            texture_version: self.texture_version.as_deref(),
        })
    }
}
