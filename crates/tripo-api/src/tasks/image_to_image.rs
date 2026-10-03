//! `image_to_image` task variant. Endpoint: `POST /generation/image-to-image`.
//!
//! The docs mark `input` as required, yet their multi-image example sends only
//! `inputs`. Both are therefore optional here, and `validate()` requires at
//! least one. Setting both is allowed: the docs' "choose exactly one" applies
//! to the input type (URL, `file_token`, or `task_id`) of each value, not to
//! the two fields.

use serde::{Deserialize, Serialize};

use super::image_rules::{self, ImageParams};
use crate::enums::{ImageBackground, ImageOutputFormat, ImageQuality, ImageToImageTemplate};
use crate::error::{Error, Result};
use crate::image::ImageInput;

/// Request body for `POST /generation/image-to-image`. The finished task
/// reports its image in `output.generated_image_url`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ImageToImageRequest {
    /// Primary reference image: URL, `file_token`, `task_id`, or local path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ImageInput>,
    /// Multiple reference images, addressed in the prompt as `image[1]`,
    /// `image[2]`, ... At most 4 (seedream), 10 (banana), or 16 (`chat_image`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<Vec<ImageInput>>,
    /// Editing instruction. Required unless `template` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Image model; see `versions::image`. Server default: `seedream_v5`.
    /// `seedream_v4` is documented for text-to-image only.
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
    /// Editing template. When set, `prompt` becomes optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<ImageToImageTemplate>,
    /// Output file format. Server default: `png`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<ImageOutputFormat>,
}

impl ImageToImageRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        let inputs = self.inputs.as_deref().unwrap_or_default();
        if self.input.is_none() && inputs.is_empty() {
            return Err(Error::InvalidRequest(
                "image-to-image needs `input` or a non-empty `inputs`".into(),
            ));
        }
        if self.prompt.is_none() && self.template.is_none() {
            return Err(Error::InvalidRequest(
                "image-to-image needs `prompt` unless `template` is set".into(),
            ));
        }
        image_rules::validate(&ImageParams {
            model: self.model.as_deref(),
            inputs: inputs.len(),
            size: self.size.as_deref(),
            quality: self.quality,
            background: self.background,
            output_format: self.output_format,
        })
    }
}
