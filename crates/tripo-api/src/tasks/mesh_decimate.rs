//! `mesh_decimate` task variant. Endpoint: `POST /mesh/decimate`.
//!
//! Retopology with three algorithm tiers: `model: "v2.0"` (default) is smart
//! highpoly-to-lowpoly retopology; `model: "v3.0"` is the newest tier and adds
//! `smart_uv`; `model: "v1.0"` is basic decimation (requires `face_limit`;
//! does not support `bake` / `part_names`).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::versions;

/// Request body for `POST /mesh/decimate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MeshDecimateRequest {
    /// Model source: `task_id`, `file_token`, or URL.
    pub input: String,
    /// Retopology algorithm version; see `versions::decimate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Produce a quad mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad: Option<bool>,
    /// Restrict to named parts (v2.0 and v3.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_names: Option<Vec<String>>,
    /// Target face count. Optional for v2.0 and v3.0 (adaptive when omitted;
    /// 500–20,000 triangles or 500–10,000 quads); required for v1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_limit: Option<i32>,
    /// Bake textures onto the low-poly model (v2.0 and v3.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake: Option<bool>,
    /// Unwrap UVs during retopology (v3.0 only; no extra cost). Produces no
    /// UV atlas preview.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smart_uv: Option<bool>,
}

impl MeshDecimateRequest {
    /// Reject combinations the server documents as errors. Unrecognized
    /// `model` strings pass through so newer server versions aren't blocked.
    pub(crate) fn validate(&self) -> Result<()> {
        use versions::decimate::{V1_0, V2_0, V3_0};

        let model = self.model.as_deref();
        if self.smart_uv == Some(true) && matches!(model, None | Some(V1_0 | V2_0)) {
            return Err(Error::InvalidRequest(format!(
                "smart_uv requires model {V3_0}"
            )));
        }
        if model == Some(V1_0) && self.face_limit.is_none() {
            return Err(Error::InvalidRequest(format!(
                "model {V1_0} requires face_limit"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use versions::decimate::{V1_0, V2_0, V3_0};

    fn req(
        model: Option<&str>,
        smart_uv: Option<bool>,
        face_limit: Option<i32>,
    ) -> MeshDecimateRequest {
        MeshDecimateRequest {
            input: "task_id".into(),
            model: model.map(str::to_owned),
            quad: None,
            part_names: None,
            face_limit,
            bake: None,
            smart_uv,
        }
    }

    #[test]
    fn smart_uv_requires_v3() {
        for model in [None, Some(V2_0)] {
            let err = req(model, Some(true), None).validate().unwrap_err();
            assert!(
                matches!(err, Error::InvalidRequest(ref msg) if msg.contains("smart_uv") && msg.contains(V3_0)),
                "{err}"
            );
        }
        req(Some(V1_0), Some(true), Some(1000))
            .validate()
            .unwrap_err();
        req(Some(V3_0), Some(true), None).validate().unwrap();
        req(Some("v3.1"), Some(true), None).validate().unwrap();
        req(None, Some(false), None).validate().unwrap();
    }

    #[test]
    fn v1_requires_face_limit() {
        let err = req(Some(V1_0), None, None).validate().unwrap_err();
        assert!(
            matches!(err, Error::InvalidRequest(ref msg) if msg.contains("face_limit")),
            "{err}"
        );
        req(Some(V1_0), None, Some(3000)).validate().unwrap();
        req(None, None, None).validate().unwrap();
        req(Some(V3_0), None, None).validate().unwrap();
    }
}
