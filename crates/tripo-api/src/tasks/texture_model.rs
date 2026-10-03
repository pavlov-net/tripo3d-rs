//! `texture_model` task variant. Endpoint: `POST /models/texture`.
//!
//! Wire-format quirk: `text` / `image` / `images` / `style_image` are rolled
//! up into a nested `texture_prompt` object, sent only when at least one is
//! present. `text`/`image`/`images` are mutually exclusive; `style_image` may
//! only accompany `text`.

use serde::{Deserialize, Serialize};

use crate::compress::CompressionMode;
use crate::enums::{TextureAlignment, TextureQuality};
use crate::error::{Error, Result};
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
    /// Exactly 4 reference images in order [front, left, back, right] for
    /// multi-angle texture guidance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ImageInput>>,
    /// Style image (uploaded/URL/token). Only used with `text`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_image: Option<ImageInput>,
}

impl TexturePrompt {
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_none()
            && self.image.is_none()
            && self.images.is_none()
            && self.style_image.is_none()
    }

    fn validate(&self) -> Result<()> {
        let modes = [
            self.text.is_some(),
            self.image.is_some(),
            self.images.is_some(),
        ];
        if modes.into_iter().filter(|&set| set).count() > 1 {
            return Err(Error::InvalidRequest(
                "texture_prompt.text, texture_prompt.image, and texture_prompt.images are mutually exclusive".into(),
            ));
        }
        if self.style_image.is_some() && self.text.is_none() {
            return Err(Error::InvalidRequest(
                "texture_prompt.style_image is only used with texture_prompt.text".into(),
            ));
        }
        if let Some(images) = &self.images
            && images.len() != 4
        {
            return Err(Error::InvalidRequest(format!(
                "texture_prompt.images requires exactly 4 images [front, left, back, right], got {}",
                images.len()
            )));
        }
        Ok(())
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
    /// Texture model version; see `versions::texture`. `v3.5-20260815` is
    /// required for `texture_quality: fast` and `delight`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// PBR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pbr: Option<bool>,
    /// Texture seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_seed: Option<i32>,
    /// Texture quality. `fast` requires `model: v3.5-20260815`.
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
    /// Remove baked-in lighting from the reference image before texturing
    /// (default true server-side). Only texture model v3.5 reads it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delight: Option<bool>,
}

impl TextureModelRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        super::validate_fast_texture(
            self.texture_quality.as_ref(),
            "model",
            self.model.as_deref(),
        )?;
        self.texture_prompt.validate()
    }
}
