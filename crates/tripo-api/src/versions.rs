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

/// Models accepted by `generation/text-to-image` and `generation/image-to-image`.
/// [`image::SEEDREAM_V4`] is documented for text-to-image only.
///
/// The two endpoints have different server defaults:
/// [`image::TEXT_TO_IMAGE_DEFAULT`] and [`image::IMAGE_TO_IMAGE_DEFAULT`].
pub mod image {
    /// Seedream v5. Strongest prompt following, editing, and multi-image fusion.
    pub const SEEDREAM_V5: &str = "seedream_v5";
    /// Seedream v4. Balanced text-to-image model. The image-to-image docs do
    /// not list it; prefer [`SEEDREAM_V5`] there.
    pub const SEEDREAM_V4: &str = "seedream_v4";
    /// Banana. Fast generation.
    pub const BANANA: &str = "banana";
    /// Banana Pro. Higher quality than [`BANANA`].
    pub const BANANA_PRO: &str = "banana_pro";
    /// Banana 2. Latest fast option; supports extra extreme aspect ratios.
    pub const BANANA2: &str = "banana2";
    /// GPT Image 1. Tripo retires this model on 2026-10-23.
    #[deprecated(
        note = "Tripo retires chat_image_1 on 2026-10-23; use CHAT_IMAGE_2 or a 2.5 model"
    )]
    pub const CHAT_IMAGE_1: &str = "chat_image_1";
    /// GPT Image 1.5. Tripo retires this model on 2026-12-01.
    #[deprecated(
        note = "Tripo retires chat_image_1.5 on 2026-12-01; use CHAT_IMAGE_2 or a 2.5 model"
    )]
    pub const CHAT_IMAGE_1_5: &str = "chat_image_1.5";
    /// GPT Image 2. Accepts `quality` `low` / `medium` / `high` and custom
    /// `WIDTHxHEIGHT` sizes.
    pub const CHAT_IMAGE_2: &str = "chat_image_2";
    /// GPT Image 2.5, speed tier. Same cost as [`CHAT_IMAGE_2_5_SUNBURST`],
    /// noticeably faster. Accepts all five `quality` tiers and `background`.
    pub const CHAT_IMAGE_2_5_FLARE: &str = "chat_image_2.5_flare";
    /// GPT Image 2.5, fidelity tier. Best at preserving untouched regions when
    /// editing. Accepts all five `quality` tiers and `background`.
    pub const CHAT_IMAGE_2_5_SUNBURST: &str = "chat_image_2.5_sunburst";
    /// Server default for `generation/text-to-image`.
    pub const TEXT_TO_IMAGE_DEFAULT: &str = SEEDREAM_V4;
    /// Server default for `generation/image-to-image`.
    pub const IMAGE_TO_IMAGE_DEFAULT: &str = SEEDREAM_V5;
}

/// Texture model versions. Accepted as `model` on `models/texture` and as
/// `texture_version` on the generation endpoints (`generation/text-to-model`,
/// `image-to-model`, `multiview-to-model`), where it pins the texture model
/// independently of the geometry `model`.
///
/// When a generation request omits `texture_version`, the server derives it
/// from `model`: v2.5 geometry uses [`V2_5`](texture::V2_5), every other
/// geometry version (v3.x, P1, P2) uses [`V3_0`](texture::V3_0). Features that
/// need [`V3_5`](texture::V3_5) therefore need it set explicitly.
pub mod texture {
    /// v3.5 (August 2026). Required for
    /// [`TextureQuality::Fast`](crate::enums::TextureQuality::Fast); the only
    /// version that reads `delight`.
    pub const V3_5: &str = "v3.5-20260815";
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
