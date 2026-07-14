//! `mesh_decimate` task variant. Endpoint: `POST /mesh/decimate`.
//!
//! Retopology with two algorithm tiers: `model: "v2.0"` (default) is smart
//! highpoly-to-lowpoly retopology; `model: "v1.0"` is basic decimation
//! (requires `face_limit`; does not support `bake` / `part_names`).

use serde::{Deserialize, Serialize};

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
    /// Restrict to named parts (v2.0 only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_names: Option<Vec<String>>,
    /// Target face count. Optional for v2.0 (adaptive); required for v1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_limit: Option<i32>,
    /// Bake textures onto the low-poly model (v2.0 only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake: Option<bool>,
}
