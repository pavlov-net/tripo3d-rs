//! `texture_model` task variant. Endpoint: `POST /models/texture`.
//!
//! Wire-format quirk: `text` / `image` / `style_image` are rolled up into a
//! nested `texture_prompt` object, sent only when at least one is present.
//! `text`/`image` are mutually exclusive; `style_image` may accompany `text`.

use serde::{Deserialize, Serialize};

use crate::compress::CompressionMode;
use crate::enums::{TextureAlignment, TextureQuality};
use crate::image::ImageInput;

/// Sub-object carrying the texture-prompt inputs.
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
    /// Style image (uploaded/URL/token). Only used with `text`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_image: Option<ImageInput>,
}

impl TexturePrompt {
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_none() && self.image.is_none() && self.style_image.is_none()
    }
}

/// Request body for `POST /models/texture`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct TextureModelRequest {
    /// Model source: `task_id`, `file_token`, or URL.
    pub input: String,
    /// Nested prompt object; omitted when all sub-fields are None.
    #[serde(default, skip_serializing_if = "TexturePrompt::is_empty")]
    pub texture_prompt: TexturePrompt,
    /// Texture model version; see `versions::texture`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// PBR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pbr: Option<bool>,
    /// Texture seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_seed: Option<i32>,
    /// Texture quality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_quality: Option<TextureQuality>,
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
