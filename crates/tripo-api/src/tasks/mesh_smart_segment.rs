//! `mesh_smart_segment` task variant. Endpoint: `POST /mesh/smartsegment`.
//!
//! End-to-end segmentation that includes auto modeling, unlike
//! `mesh/segment`, which only segments an existing model.

use serde::{Deserialize, Serialize};

use crate::enums::{SegGranularity, SegType};
use crate::error::{Error, Result};
use crate::image::ImageInput;

/// Request body for `POST /mesh/smartsegment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MeshSmartSegmentRequest {
    /// Pipeline: `image` (PNG/JPEG/WebP input) or `model` (GLB input only).
    pub seg_type: SegType,
    /// Source: URL, `file_token`, or local path (uploaded first).
    pub input: ImageInput,
    /// Segmentation granularity (server default `medium`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<SegGranularity>,
    /// Hint text describing the parts to segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Column-major 4×4 transform matrix. Required when `seg_type` is `model`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<[f64; 16]>,
}

impl MeshSmartSegmentRequest {
    /// `seg_type: model` requires `transform`, and its values must be finite
    /// (`serde_json` would send NaN/infinity as `null`).
    pub(crate) fn validate(&self) -> Result<()> {
        match (&self.seg_type, &self.transform) {
            (_, Some(t)) if !t.iter().all(|v| v.is_finite()) => Err(Error::InvalidRequest(
                "transform values must be finite".into(),
            )),
            (SegType::Model, None) => Err(Error::InvalidRequest(
                "seg_type model requires transform (16 numbers, column-major 4x4)".into(),
            )),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(seg_type: SegType, transform: Option<[f64; 16]>) -> MeshSmartSegmentRequest {
        MeshSmartSegmentRequest {
            seg_type,
            input: ImageInput::FileToken("file_x".into()),
            granularity: None,
            hint: None,
            transform,
        }
    }

    fn identity() -> [f64; 16] {
        let mut m = [0.0; 16];
        for i in 0..4 {
            m[i * 5] = 1.0;
        }
        m
    }

    #[test]
    fn image_without_transform_ok() {
        req(SegType::Image, None).validate().unwrap();
    }

    #[test]
    fn model_with_16_floats_ok() {
        req(SegType::Model, Some(identity())).validate().unwrap();
    }

    #[test]
    fn model_without_transform_rejected() {
        let err = req(SegType::Model, None).validate().unwrap_err();
        assert!(matches!(err, Error::InvalidRequest(ref m) if m.contains("transform")));
    }

    #[test]
    fn non_finite_transform_rejected() {
        let mut t = identity();
        t[3] = f64::NAN;
        let err = req(SegType::Model, Some(t)).validate().unwrap_err();
        assert!(matches!(err, Error::InvalidRequest(ref m) if m.contains("finite")));
    }

    #[test]
    fn transform_length_enforced_on_deserialize() {
        let body =
            serde_json::json!({"seg_type": "model", "input": "file_x", "transform": vec![1.0; 9]});
        assert!(serde_json::from_value::<MeshSmartSegmentRequest>(body).is_err());
    }
}
