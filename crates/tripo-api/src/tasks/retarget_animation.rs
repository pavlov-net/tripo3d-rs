//! `retarget_animation` task variant. Wire `type`: `animate_retarget`.
//!
//! Polymorphic animation field: a single animation serializes under the key
//! `animation`, a list under `animations`. Different field names, so the
//! simplest correct encoding is two `Option` fields that are mutually exclusive.

use serde::Serialize;

use crate::enums::{Animation, RigOutputFormat};

/// Request body for `retarget_animation`. Wire `type`: `animate_retarget`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RetargetAnimationRequest {
    /// Source rigged task id.
    pub original_model_task_id: String,
    /// Single animation preset (serializes as `animation`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<Animation>,
    /// List of animation presets (serializes as `animations`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<Animation>>,
    /// Output file format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_format: Option<RigOutputFormat>,
    /// Bake animation samples.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake_animation: Option<bool>,
    /// Export with skinned geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_with_geometry: Option<bool>,
    /// Animate in-place.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animate_in_place: Option<bool>,
}

impl RetargetAnimationRequest {
    /// Build with a single animation preset.
    #[must_use]
    pub fn single(original_model_task_id: impl Into<String>, animation: Animation) -> Self {
        Self {
            original_model_task_id: original_model_task_id.into(),
            animation: Some(animation),
            animations: None,
            out_format: None,
            bake_animation: None,
            export_with_geometry: None,
            animate_in_place: None,
        }
    }
    /// Build with multiple animations (list).
    #[must_use]
    pub fn many(original_model_task_id: impl Into<String>, animations: Vec<Animation>) -> Self {
        Self {
            original_model_task_id: original_model_task_id.into(),
            animation: None,
            animations: Some(animations),
            out_format: None,
            bake_animation: None,
            export_with_geometry: None,
            animate_in_place: None,
        }
    }
}
