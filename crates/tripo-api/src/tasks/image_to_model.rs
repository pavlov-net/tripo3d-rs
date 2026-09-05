//! `image_to_model` task variant. Endpoint: `POST /generation/image-to-model`.

use serde::{Deserialize, Serialize};

use crate::compress::CompressionMode;
use crate::enums::{
    ExportOrientation, GeometryQuality, Orientation, TextureAlignment, TextureQuality,
};
use crate::error::Result;
use crate::image::ImageInput;

/// Request body for `POST /generation/image-to-model`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ImageToModelRequest {
    /// The input image: URL, `file_token`, or `task_id` of a previous image
    /// generation task. Serialized as a bare string.
    pub input: ImageInput,
    /// AI model version string; see `versions::text_image`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Automatically optimize the input image before generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_image_autofix: Option<bool>,
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
    pub texture_quality: Option<TextureQuality>,
    /// Geometry quality preset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry_quality: Option<GeometryQuality>,
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
    /// UV unwrapping during generation (default true server-side).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_uv: Option<bool>,
    /// Forward axis for this generation only (server default: `+x`).
    /// Leave unset when post-processing; set orientation in the final convert
    /// step instead to avoid incorrectly oriented downstream results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_orientation: Option<ExportOrientation>,
}

impl ImageToModelRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        super::validate_p2_face_limit(self.model.as_deref(), self.quad, self.face_limit)?;
        super::validate_p1_params(
            self.model.as_deref(),
            self.quad,
            self.smart_low_poly,
            self.generate_parts,
            self.geometry_quality.as_ref(),
        )
    }
}
