//! `text_to_model` task variant.

use serde::{Deserialize, Serialize};

use crate::compress::CompressionMode;
use crate::enums::Quality;
use crate::error::Result;

/// Request body for `text_to_model`. Wire `type`: `text_to_model`.
///
/// See the Python SDK `Client.text_to_model` for the authoritative field list.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct TextToModelRequest {
    /// Prompt text. Required.
    pub prompt: String,
    /// Negative prompt (things to avoid).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// Model version string; see `versions::text_image`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
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
    /// Texture quality preset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_quality: Option<Quality>,
    /// Geometry quality preset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry_quality: Option<Quality>,
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
}

impl TextToModelRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        super::validate_p1_params(
            self.model_version.as_deref(),
            self.quad,
            self.smart_low_poly,
            self.generate_parts,
            self.geometry_quality.as_ref(),
        )
    }
}
