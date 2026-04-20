//! `multiview_to_model` task variant.
//!
//! Wire-format note: the images array is sent as `files` (list); `None`
//! entries serialize as `{}` empty objects (positional "no image at this slot").

use serde::{Serialize, Serializer};

use crate::compress::CompressionMode;
use crate::enums::{Orientation, Quality, TextureAlignment};
use crate::image::ImageInput;

/// Request body for `multiview_to_model`. Wire `type`: `multiview_to_model`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MultiviewToModelRequest {
    /// Ordered list of images. `None` entries become `{}` placeholders on the wire.
    #[serde(rename = "files", serialize_with = "serialize_files")]
    pub images: Vec<Option<ImageInput>>,
    /// Model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Target face count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_limit: Option<i32>,
    /// Texture.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<bool>,
    /// PBR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pbr: Option<bool>,
    /// Seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_seed: Option<i32>,
    /// Seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_seed: Option<i32>,
    /// Texture quality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_quality: Option<Quality>,
    /// Geometry quality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry_quality: Option<Quality>,
    /// Texture alignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_alignment: Option<TextureAlignment>,
    /// Auto-size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_size: Option<bool>,
    /// Orientation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
    /// Quad mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad: Option<bool>,
    /// Compression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<CompressionMode>,
    /// Generate parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_parts: Option<bool>,
    /// Smart lowpoly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smart_low_poly: Option<bool>,
}

fn serialize_files<S: Serializer>(
    v: &[Option<ImageInput>],
    s: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut seq = s.serialize_seq(Some(v.len()))?;
    for entry in v {
        match entry {
            Some(img) => seq.serialize_element(img)?,
            None => seq.serialize_element(&serde_json::Value::Object(serde_json::Map::new()))?,
        }
    }
    seq.end()
}
