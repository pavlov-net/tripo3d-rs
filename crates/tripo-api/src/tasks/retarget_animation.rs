//! `retarget_animation` task variant. Endpoint: `POST /animations/retarget`.
//!
//! Uses `AnimationInput` to type-enforce the single-or-many wire-format
//! invariant: a single animation serializes under key `animation`, a list
//! under `animations`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

/// Request body for `POST /animations/retarget`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct RetargetAnimationRequest {
    /// `task_id` of the rigged model.
    pub input: String,
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
        m.serialize_entry("input", &self.input)?;
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

impl<'de> Deserialize<'de> for RetargetAnimationRequest {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            input: String,
            #[serde(default)]
            animation: Option<Animation>,
            #[serde(default)]
            animations: Option<Vec<Animation>>,
            #[serde(default)]
            out_format: Option<RigOutputFormat>,
            #[serde(default)]
            bake_animation: Option<bool>,
            #[serde(default)]
            export_with_geometry: Option<bool>,
            #[serde(default)]
            animate_in_place: Option<bool>,
        }

        let w = Wire::deserialize(d)?;
        let animation = match (w.animation, w.animations) {
            (Some(a), None) => AnimationInput::Single(a),
            (None, Some(list)) => AnimationInput::Many(list),
            (Some(_), Some(_)) => {
                return Err(D::Error::custom(
                    "set either `animation` or `animations`, not both",
                ));
            }
            (None, None) => {
                return Err(D::Error::custom(
                    "missing required field `animation` or `animations`",
                ));
            }
        };
        Ok(Self {
            input: w.input,
            animation,
            out_format: w.out_format,
            bake_animation: w.bake_animation,
            export_with_geometry: w.export_with_geometry,
            animate_in_place: w.animate_in_place,
        })
    }
}

impl RetargetAnimationRequest {
    /// Build with a single animation preset.
    #[must_use]
    pub fn single(input: impl Into<String>, animation: Animation) -> Self {
        Self {
            input: input.into(),
            animation: AnimationInput::Single(animation),
            out_format: None,
            bake_animation: None,
            export_with_geometry: None,
            animate_in_place: None,
        }
    }
    /// Build with multiple animations (list).
    #[must_use]
    pub fn many(input: impl Into<String>, animations: Vec<Animation>) -> Self {
        Self {
            input: input.into(),
            animation: AnimationInput::Many(animations),
            out_format: None,
            bake_animation: None,
            export_with_geometry: None,
            animate_in_place: None,
        }
    }
}
