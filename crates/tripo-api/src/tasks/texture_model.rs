//! `texture_model` task variant.
//!
//! Wire-format quirk: `text_prompt` / `image_prompt` / `style_image` are
//! rolled up into a nested `texture_prompt` object, sent only when at least
//! one of the three is present.

use serde::{Deserialize, Serialize};

use crate::compress::CompressionMode;
use crate::enums::{Quality, TextureAlignment};
use crate::image::ImageInput;

/// Sub-object carrying the three texture-prompt inputs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, default)]
pub struct TexturePrompt {
    /// Text prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Reference image (uploaded/URL/token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageInput>,
    /// Style image (uploaded/URL/token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_image: Option<ImageInput>,
}

impl TexturePrompt {
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_none() && self.image.is_none() && self.style_image.is_none()
    }
}

/// Request body for `texture_model`. Wire `type`: `texture_model`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct TextureModelRequest {
    /// Source task id.
    pub original_model_task_id: String,
    /// Nested prompt object; omitted when all sub-fields are None.
    #[serde(default, skip_serializing_if = "TexturePrompt::is_empty")]
    pub texture_prompt: TexturePrompt,
    /// Model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Texture.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<bool>,
    /// PBR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pbr: Option<bool>,
    /// Model seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_seed: Option<i32>,
    /// Texture seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_seed: Option<i32>,
    /// Texture quality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_quality: Option<Quality>,
    /// Texture alignment strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_alignment: Option<TextureAlignment>,
    /// Restrict to named parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_names: Option<Vec<String>>,
    /// Geometry compression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<CompressionMode>,
    /// Bake textures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake: Option<bool>,
}
