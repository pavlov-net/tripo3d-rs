//! Known `model` version string constants, grouped by the endpoint they apply to.
//!
//! `model` fields are serialized as bare strings; these constants are a
//! convenience layer over the wire format. Servers may introduce new versions
//! between SDK releases — callers can always pass any `&str` or `String`.
//! v3 also accepts short aliases like `tripo-v3.1` / `tripo-p1` for the
//! generation endpoints.

/// Versions accepted by `generation/text-to-model`, `generation/image-to-model`.
pub mod text_image {
    /// P2 (August 2026, preview) — low-poly generation with optional quad output.
    pub const P2: &str = "P2-20260801";
    /// P1 (March 2026) — low-poly-optimized P series. Doesn't support `quad`,
    /// `smart_low_poly`, `generate_parts`, or `geometry_quality`.
    pub const P1: &str = "P1-20260311";
    /// v3.1 (February 2026). Latest, best quality. Server default.
    pub const V3_1: &str = "v3.1-20260211";
    /// v3.0 (August 2025). Stable, advanced features.
    pub const V3_0: &str = "v3.0-20250812";
    /// v2.5 (January 2025). Balanced legacy version.
    pub const V2_5: &str = "v2.5-20250123";
    /// Server default.
    pub const DEFAULT: &str = V3_1;
}

/// Versions accepted by `generation/multiview-to-model`.
pub mod multiview {
    /// P2 (August 2026, preview) — low-poly generation with optional quad output.
    pub const P2: &str = super::text_image::P2;
    /// v3.1 (February 2026). Latest, best quality. Server default.
    pub const V3_1: &str = "v3.1-20260211";
    /// v3.0 (August 2025).
    pub const V3_0: &str = "v3.0-20250812";
    /// v2.5 (January 2025).
    pub const V2_5: &str = "v2.5-20250123";
    /// Server default.
    pub const DEFAULT: &str = V3_1;
}

/// Versions accepted by `models/texture`.
pub mod texture {
    /// v3.0 (August 2025). Server default; recommended for models generated
    /// with v3.0 or v3.1.
    pub const V3_0: &str = "v3.0-20250812";
    /// v2.5 (January 2025). Recommended for models generated with v2.5.
    pub const V2_5: &str = "v2.5-20250123";
    /// Server default.
    pub const DEFAULT: &str = V3_0;
}

/// Versions accepted by `animations/rig`.
pub mod rig {
    /// v1.0 (March 2024). Only supports `rig_type: biped`; 90+ animation
    /// presets. Server default.
    pub const V1_0: &str = "v1.0-20240301";
    /// v2.5 (February 2026). Supports all `rig_type` values.
    pub const V2_5: &str = "v2.5-20260210";
    /// Server default (note: v1.0 is biped-only — use `V2_5` for any
    /// non-biped `rig_type`).
    pub const DEFAULT: &str = V1_0;
}

/// Single known version for `mesh/segment` and `mesh/complete`.
pub mod mesh {
    /// v1.0 (May 2025).
    pub const V1_0: &str = "v1.0-20250506";
    /// Server default.
    pub const DEFAULT: &str = V1_0;
}

/// Versions accepted by `mesh/decimate` (retopology).
pub mod decimate {
    /// v2.0 — smart retopology (P-series AI model, 30 credits). Server default.
    pub const V2_0: &str = "v2.0";
    /// v1.0 — basic decimation (10 credits). Requires `face_limit`.
    pub const V1_0: &str = "v1.0";
    /// Server default.
    pub const DEFAULT: &str = V2_0;
}
