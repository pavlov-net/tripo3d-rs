//! `retarget_animation` task variant. Wire `type`: `animate_retarget`.
//!
//! Uses `AnimationInput` to type-enforce the single-or-many wire-format
//! invariant: a single animation serializes under key `animation`, a list
//! under `animations`.

use serde::{Serialize, Serializer};

use crate::enums::{Animation, RigOutputFormat};

/// One animation or many. Serializes to the correct wire key (`animation`
/// or `animations`) — cannot be in an invalid "both set" state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum AnimationInput {
    /// One animation preset; serializes under key `animation`.
    Single(Animation),
    /// Multiple animation presets; serializes under key `animations`.
    Many(Vec<Animation>),
}

/// Request body for `retarget_animation`. Wire `type`: `animate_retarget`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct RetargetAnimationRequest {
    /// Source rigged task id.
    pub original_model_task_id: String,
    /// Animation(s) to retarget; serializes as `animation` (single) or `animations` (list).
    pub animation: AnimationInput,
    /// Output file format.
    pub out_format: Option<RigOutputFormat>,
    /// Bake animation samples.
    pub bake_animation: Option<bool>,
    /// Export with skinned geometry.
    pub export_with_geometry: Option<bool>,
    /// Animate in-place.
    pub animate_in_place: Option<bool>,
}

impl Serialize for RetargetAnimationRequest {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("original_model_task_id", &self.original_model_task_id)?;
        match &self.animation {
            AnimationInput::Single(a) => m.serialize_entry("animation", a)?,
            AnimationInput::Many(list) => m.serialize_entry("animations", list)?,
        }
        if let Some(v) = &self.out_format {
            m.serialize_entry("out_format", v)?;
        }
        if let Some(v) = &self.bake_animation {
            m.serialize_entry("bake_animation", v)?;
        }
        if let Some(v) = &self.export_with_geometry {
            m.serialize_entry("export_with_geometry", v)?;
        }
        if let Some(v) = &self.animate_in_place {
            m.serialize_entry("animate_in_place", v)?;
        }
        m.end()
    }
}

impl RetargetAnimationRequest {
    /// Build with a single animation preset.
    #[must_use]
    pub fn single(original_model_task_id: impl Into<String>, animation: Animation) -> Self {
        Self {
            original_model_task_id: original_model_task_id.into(),
            animation: AnimationInput::Single(animation),
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
            animation: AnimationInput::Many(animations),
            out_format: None,
            bake_animation: None,
            export_with_geometry: None,
            animate_in_place: None,
        }
    }
}
