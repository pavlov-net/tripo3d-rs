//! `convert_model` task variant.

use serde::{Deserialize, Serialize};

use crate::enums::{ExportOrientation, FbxPreset, OutputFormat, TextureFormat};

/// Request body for `convert_model`. Wire `type`: `convert_model`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ConvertModelRequest {
    /// Source task id.
    pub original_model_task_id: String,
    /// Target mesh format.
    pub format: OutputFormat,
    /// Quad output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad: Option<bool>,
    /// Mirror symmetry enforcement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_symmetry: Option<bool>,
    /// Face count limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_limit: Option<i32>,
    /// Flatten the model bottom.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flatten_bottom: Option<bool>,
    /// Flatten threshold (meters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flatten_bottom_threshold: Option<f64>,
    /// Texture resolution (pixels per side).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_size: Option<i32>,
    /// Texture format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_format: Option<TextureFormat>,
    /// Uniform scale factor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_factor: Option<f64>,
    /// Reset pivot to the bottom-center of the bounding box.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pivot_to_center_bottom: Option<bool>,
    /// Include animation data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_animation: Option<bool>,
    /// Pack UVs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_uv: Option<bool>,
    /// Bake textures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake: Option<bool>,
    /// Restrict to named parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_names: Option<Vec<String>>,
    /// Export per-vertex colors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_vertex_colors: Option<bool>,
    /// FBX target preset (Blender/Mixamo/3DsMax).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fbx_preset: Option<FbxPreset>,
    /// Output axis orientation (e.g. `+y`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_orientation: Option<ExportOrientation>,
    /// Keep animated character in place.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animate_in_place: Option<bool>,
}
