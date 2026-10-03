//! `text_to_image` task variant. Endpoint: `POST /generation/text-to-image`.

use serde::{Deserialize, Serialize};

use super::image_rules::{self, ImageParams};
use crate::enums::{ImageBackground, ImageOutputFormat, ImageQuality, TextToImageTemplate};
use crate::error::Result;

/// Request body for `POST /generation/text-to-image`. The finished task
/// reports its image in `output.generated_image_url`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct TextToImageRequest {
    /// Prompt text. Required, even with `template`. Append negative prompts
    /// after `--no`.
    pub prompt: String,
    /// Image model; see `versions::image`. Server default: `seedream_v4`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Output size: a keyword (`2K`, `4K`) or `WIDTHxHEIGHT`. Supported
    /// values vary by model. Server default: `2048x2048`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Rendering tier; model support and cost in [`ImageQuality`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,
    /// Background handling; model support in [`ImageBackground`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<ImageBackground>,
    /// Aspect ratio such as `16:9`. Banana models only; others use `size`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// Output file format. Server default: `png`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<ImageOutputFormat>,
    /// Add an AI-generated-content watermark. Effective for seedream only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<bool>,
    /// Generation template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<TextToImageTemplate>,
}

impl TextToImageRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        image_rules::validate(&ImageParams {
            model: self.model.as_deref(),
            inputs: 0,
            size: self.size.as_deref(),
            quality: self.quality,
            background: self.background,
            output_format: self.output_format,
        })
    }
}
