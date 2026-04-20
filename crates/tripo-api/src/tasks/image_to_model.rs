//! `image_to_model` task variant.

use serde::Serialize;

use crate::compress::CompressionMode;
use crate::enums::{Orientation, Quality, TextureAlignment};
use crate::image::ImageInput;

/// Request body for `image_to_model`. Wire `type`: `image_to_model`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ImageToModelRequest {
    /// The input image — on the wire this is serialized as `file`, not `image`.
    #[serde(rename = "file")]
    pub image: ImageInput,
    /// Model version string; see `versions::text_image`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Target face count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_limit: Option<i32>,
    /// Generate a texture.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<bool>,
    /// PBR shading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pbr: Option<bool>,
    /// Seed for model generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_seed: Option<i32>,
    /// Seed for texture generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_seed: Option<i32>,
    /// Texture quality preset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_quality: Option<Quality>,
    /// Geometry quality preset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry_quality: Option<Quality>,
    /// Texture alignment strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_alignment: Option<TextureAlignment>,
    /// Auto-size the output mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_size: Option<bool>,
    /// Output orientation hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
    /// Produce a quad mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad: Option<bool>,
    /// Geometry compression option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<CompressionMode>,
    /// Emit part decomposition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_parts: Option<bool>,
    /// Route through smart-lowpoly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smart_low_poly: Option<bool>,
}
