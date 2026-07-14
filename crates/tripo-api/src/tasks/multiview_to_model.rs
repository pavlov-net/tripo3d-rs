//! `multiview_to_model` task variant. Endpoint: `POST /generation/multiview-to-model`.
//!
//! Wire-format note: v3's "legacy positional" `inputs` format — an array of
//! strings in order [front, left, back, right]; `None` entries serialize as
//! `""` (positional "no image at this slot").

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::compress::CompressionMode;
use crate::enums::{GeometryQuality, Orientation, TextureAlignment, TextureQuality};
use crate::error::Result;
use crate::image::ImageInput;

/// Request body for `POST /generation/multiview-to-model`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MultiviewToModelRequest {
    /// Ordered list of images [front, left, back, right]. `None` entries
    /// become `""` placeholders on the wire. The front view is required.
    #[serde(
        serialize_with = "serialize_inputs",
        deserialize_with = "deserialize_inputs"
    )]
    pub inputs: Vec<Option<ImageInput>>,
    /// AI model version string; see `versions::multiview`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
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
    pub texture_quality: Option<TextureQuality>,
    /// Geometry quality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry_quality: Option<GeometryQuality>,
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
    /// UV unwrapping during generation (default true server-side).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_uv: Option<bool>,
}

impl MultiviewToModelRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        super::validate_p1_params(
            self.model.as_deref(),
            self.quad,
            self.smart_low_poly,
            self.generate_parts,
            self.geometry_quality.as_ref(),
        )
    }
}

fn serialize_inputs<S: Serializer>(v: &[Option<ImageInput>], s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut seq = s.serialize_seq(Some(v.len()))?;
    for entry in v {
        match entry {
            Some(img) => seq.serialize_element(img)?,
            None => seq.serialize_element("")?,
        }
    }
    seq.end()
}

fn deserialize_inputs<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Vec<Option<ImageInput>>, D::Error> {
    let entries: Vec<serde_json::Value> = Vec::deserialize(d)?;
    let mut out = Vec::with_capacity(entries.len());
    for v in entries {
        match &v {
            serde_json::Value::String(s) if s.is_empty() => out.push(None),
            // Legacy v2 placeholders.
            serde_json::Value::Object(m) if m.is_empty() => out.push(None),
            serde_json::Value::Null => out.push(None),
            _ => {
                let img: ImageInput =
                    serde_json::from_value(v).map_err(serde::de::Error::custom)?;
                out.push(Some(img));
            }
        }
    }
    Ok(out)
}
