//! Known `model_version` string constants, grouped by the variant they apply to.
//!
//! Fields of type `model_version` are serialized as bare strings; these constants
//! are a convenience layer over the wire format. Servers may introduce new versions
//! between SDK releases — callers can always pass any `&str` or `String`.

/// Versions accepted by `text_to_model`, `image_to_model`.
pub mod text_image {
    /// Latest reasonable default (February 2026).
    pub const V3_1: &str = "v3.1-20260211";
    /// v3.0 (August 2025).
    pub const V3_0: &str = "v3.0-20250812";
    /// v2.5 (January 2025). Python SDK's default.
    pub const V2_5: &str = "v2.5-20250123";
    /// v2.0 (September 2024).
    pub const V2_0: &str = "v2.0-20240919";
    /// v1.4 (June 2024).
    pub const V1_4: &str = "v1.4-20240625";
    /// Turbo v1.0 (May 2025).
    pub const TURBO_V1_0: &str = "Turbo-v1.0-20250506";
    /// Python SDK default.
    pub const DEFAULT: &str = V2_5;
}

/// Versions accepted by `multiview_to_model` (no Turbo / v1.4).
pub mod multiview {
    /// Latest (February 2026).
    pub const V3_1: &str = "v3.1-20260211";
    /// v3.0 (August 2025).
    pub const V3_0: &str = "v3.0-20250812";
    /// v2.5 (January 2025).
    pub const V2_5: &str = "v2.5-20250123";
    /// v2.0 (September 2024).
    pub const V2_0: &str = "v2.0-20240919";
    /// Python SDK default.
    pub const DEFAULT: &str = V2_5;
}

/// Versions accepted by `texture_model`.
pub mod texture {
    /// v3.0 (August 2025).
    pub const V3_0: &str = "v3.0-20250812";
    /// v2.5 (January 2025).
    pub const V2_5: &str = "v2.5-20250123";
    /// Python SDK default.
    pub const DEFAULT: &str = V2_5;
}

/// Versions accepted by `rig_model`.
pub mod rig {
    /// v1.0 (March 2024).
    pub const V1_0: &str = "v1.0-20240301";
    /// v2.0 (May 2025).
    pub const V2_0: &str = "v2.0-20250506";
    /// Python SDK default.
    pub const DEFAULT: &str = V1_0;
}

/// Single known version for `mesh_segmentation` and `mesh_completion`.
pub mod mesh {
    /// v1.0 (May 2025).
    pub const V1_0: &str = "v1.0-20250506";
    /// Python SDK default.
    pub const DEFAULT: &str = V1_0;
}

/// Single known version for `smart_lowpoly`.
pub mod lowpoly {
    /// P-v2.0 (December 2025).
    pub const P_V2_0: &str = "P-v2.0-20251226";
    /// Python SDK default.
    pub const DEFAULT: &str = P_V2_0;
}
