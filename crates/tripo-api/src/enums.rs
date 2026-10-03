//! Shared typed enums used across request and response structs.
//!
//! Two flavors:
//! - `string_enum!` — strict. Unknown wire values fail to deserialize. Used
//!   for request payloads so typos in user-supplied config (RON, CLI args)
//!   surface as errors instead of silently becoming an `Unknown` variant.
//! - `string_enum_open!` — forward-compatible. Adds a `#[serde(other)]
//!   Unknown` catchall so new server-side values don't break response parsing.

use serde::{Deserialize, Serialize};

macro_rules! string_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($(#[$vmeta:meta])* $variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        $vis enum $name {
            $(
                #[doc = concat!("Wire value: `", $wire, "`.")]
                $(#[$vmeta])*
                #[serde(rename = $wire)]
                $variant,
            )+
        }

        impl $name {
            /// The wire value.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire,)+
                }
            }
        }
    };
}

macro_rules! string_enum_open {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($(#[$vmeta:meta])* $variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        $vis enum $name {
            $(
                #[doc = concat!("Wire value: `", $wire, "`.")]
                $(#[$vmeta])*
                #[serde(rename = $wire)]
                $variant,
            )+
            /// Unknown value received from a forward-compatible server.
            #[serde(other)]
            Unknown,
        }
    };
}

string_enum! {
    /// Output mesh format accepted by `convert_model`.
    pub enum OutputFormat {
        Gltf => "GLTF",
        Usdz => "USDZ",
        Fbx  => "FBX",
        Obj  => "OBJ",
        Stl  => "STL",
        ThreeMf => "3MF",
    }
}

string_enum! {
    /// Texture image format for `convert_model`.
    pub enum TextureFormat {
        Bmp => "BMP", Dpx => "DPX", Hdr => "HDR", Jpeg => "JPEG",
        OpenExr => "OPEN_EXR", Png => "PNG", Targa => "TARGA",
        Tiff => "TIFF", Webp => "WEBP",
    }
}

string_enum! {
    /// Biological rig classification. Strict — used in `rig_model` requests.
    pub enum RigType {
        Biped => "biped", Quadruped => "quadruped", Hexapod => "hexapod",
        Octopod => "octopod", Avian => "avian", Serpentine => "serpentine",
        Aquatic => "aquatic", Others => "others",
    }
}

string_enum_open! {
    /// Rig classification reported by `check_riggable`. Forward-compatible —
    /// unrecognized server values deserialize as `Unknown`.
    pub enum RigTypeResponse {
        Biped => "biped", Quadruped => "quadruped", Hexapod => "hexapod",
        Octopod => "octopod", Avian => "avian", Serpentine => "serpentine",
        Aquatic => "aquatic", Others => "others",
    }
}

string_enum! {
    /// Target rigging convention.
    pub enum RigSpec {
        Mixamo => "mixamo",
        Tripo  => "tripo",
    }
}

string_enum! {
    /// Post-processing stylization preset.
    pub enum PostStyle {
        Lego => "lego", Voxel => "voxel", Voronoi => "voronoi", Minecraft => "minecraft",
    }
}

string_enum! {
    /// Animation preset (`retarget_animation`). Values are prefixed `preset:`.
    pub enum Animation {
        Idle => "preset:idle", Walk => "preset:walk", Run => "preset:run",
        Dive => "preset:dive", Climb => "preset:climb", Jump => "preset:jump",
        Slash => "preset:slash", Shoot => "preset:shoot", Hurt => "preset:hurt",
        Fall => "preset:fall", Turn => "preset:turn",
        QuadrupedWalk => "preset:quadruped:walk",
        HexapodWalk   => "preset:hexapod:walk",
        OctopodWalk   => "preset:octopod:walk",
        SerpentineMarch => "preset:serpentine:march",
        AquaticMarch    => "preset:aquatic:march",
    }
}

string_enum! {
    /// Texture quality level (`texture_quality`).
    ///
    /// `extreme` (added in API 1.9.7, June 2026) is the top tier — Tripo bills
    /// it as the highest-resolution PBR texture output. It is valid only for
    /// `texture_quality`, not `geometry_quality`; see [`GeometryQuality`].
    ///
    /// `fast` is the quickest tier; see [`TextureQuality::Fast`].
    pub enum TextureQuality {
        /// Quickest tier: same texture size and credit cost as `standard`,
        /// lower detail. Requires texture model v3.5
        /// ([`crate::versions::texture::V3_5`]); the server rejects it (code
        /// 1004) on any other texture version.
        Fast => "fast",
        Standard => "standard",
        Detailed => "detailed",
        Extreme  => "extreme",
    }
}

string_enum! {
    /// Geometry quality level (`geometry_quality`).
    ///
    /// Note: unlike [`TextureQuality`], there is no `extreme` geometry tier.
    pub enum GeometryQuality {
        Standard => "standard",
        Detailed => "detailed",
    }
}

string_enum! {
    /// Texture alignment strategy (image-to-model / texture-model).
    pub enum TextureAlignment {
        OriginalImage => "original_image",
        Geometry => "geometry",
    }
}

string_enum! {
    /// Output orientation hint (image-to-model).
    pub enum Orientation {
        Default => "default",
        AlignImage => "align_image",
    }
}

string_enum! {
    /// FBX preset selection (`convert_model`).
    pub enum FbxPreset {
        Blender => "blender", Mixamo => "mixamo", ThreeDsMax => "3dsmax",
    }
}

string_enum! {
    /// Export forward axis for generation and model conversion.
    pub enum ExportOrientation {
        PlusX => "+x", PlusY => "+y", MinusX => "-x", MinusY => "-y",
    }
}

string_enum! {
    /// Output file format for `rig_model` / `retarget_animation`.
    pub enum RigOutputFormat {
        Glb => "glb",
        Fbx => "fbx",
    }
}

string_enum! {
    /// Rendering tier for image generation (`quality`).
    ///
    /// Only `chat_image_2` (`low`/`medium`/`high`) and the two
    /// `chat_image_2.5_*` models (all tiers) accept it. Omitting it is
    /// equivalent to `low`. `high`, `xhigh`, and `max` cost more credits and
    /// take longer.
    pub enum ImageQuality {
        Low => "low",
        Medium => "medium",
        High => "high",
        Xhigh => "xhigh",
        Max => "max",
    }
}

string_enum! {
    /// Background handling for image generation (`background`). Honored only
    /// by the `chat_image_2.5_*` models; other models ignore it. `transparent`
    /// requires `output_format` `png`.
    pub enum ImageBackground {
        Auto => "auto",
        Opaque => "opaque",
        /// Requires `output_format` `png`.
        Transparent => "transparent",
    }
}

string_enum! {
    /// Output image file format (`output_format`) for image generation.
    pub enum ImageOutputFormat {
        Png => "png",
        Jpeg => "jpeg",
    }
}

string_enum! {
    /// Generation template for `generation/text-to-image`.
    pub enum TextToImageTemplate {
        /// Extract a subject from a scene.
        AssetExtraction => "asset_extraction",
        /// Complete a partial character.
        CharacterCompletion => "character_completion",
        /// Generate a character in T-pose.
        TPose => "t_pose",
        /// Generate design variations.
        Variants => "variants",
        /// Generate a full-body figure.
        Figure => "figure",
    }
}

string_enum! {
    /// Editing template for `generation/image-to-image`. When set, `prompt`
    /// becomes optional.
    pub enum ImageToImageTemplate {
        /// Convert to T-pose.
        TPose => "t_pose",
        /// Complete a partial character.
        CharacterCompletion => "character_completion",
        /// Enhance for 3D generation.
        Enhance3d => "3d_enhance",
        /// Generate design variations.
        Variants => "variants",
        /// Generate a full-body figure.
        Figure => "figure",
    }
}

string_enum! {
    /// One view of a four-view multiview image (`generation/edit-multiview`).
    #[derive(PartialOrd, Ord)]
    pub enum MultiviewView {
        Front => "front",
        Left => "left",
        Back => "back",
        Right => "right",
    }
}

string_enum! {
    /// Smart-segmentation pipeline (`mesh/smartsegment` `seg_type`).
    pub enum SegType {
        /// Image pipeline (PNG / JPEG / WebP input).
        Image => "image",
        /// Model pipeline (GLB input only; requires `transform`).
        Model => "model",
    }
}

string_enum! {
    /// Smart-segmentation granularity (`mesh/smartsegment`). Server default: `medium`.
    pub enum SegGranularity {
        Coarse => "coarse",
        Medium => "medium",
        Fine => "fine",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_serializes_with_preset_prefix() {
        let json = serde_json::to_string(&Animation::QuadrupedWalk).unwrap();
        assert_eq!(json, "\"preset:quadruped:walk\"");
    }

    #[test]
    fn post_style_roundtrips() {
        for s in [
            PostStyle::Lego,
            PostStyle::Voxel,
            PostStyle::Voronoi,
            PostStyle::Minecraft,
        ] {
            let j = serde_json::to_string(&s).unwrap();
            let back: PostStyle = serde_json::from_str(&j).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn strict_enum_rejects_unknown_value() {
        let err = serde_json::from_str::<PostStyle>("\"brand_new_style\"").unwrap_err();
        assert!(err.to_string().contains("unknown variant"), "{err}");
    }

    #[test]
    fn strict_enum_rejects_wrong_case() {
        // Guards against the footgun where e.g. `"Detailed"` silently became
        // `TextureQuality::Unknown` instead of erroring, since the wire value
        // is lowercase `"detailed"`.
        let err = serde_json::from_str::<TextureQuality>("\"Detailed\"").unwrap_err();
        assert!(err.to_string().contains("unknown variant"), "{err}");
    }

    #[test]
    fn texture_quality_has_extreme_tier() {
        let j = serde_json::to_string(&TextureQuality::Extreme).unwrap();
        assert_eq!(j, "\"extreme\"");
        let back: TextureQuality = serde_json::from_str("\"extreme\"").unwrap();
        assert_eq!(back, TextureQuality::Extreme);
        // `extreme` is texture-only — it must not deserialize as a geometry tier.
        assert!(serde_json::from_str::<GeometryQuality>("\"extreme\"").is_err());
    }

    #[test]
    fn open_enum_accepts_unknown_value() {
        let got: RigTypeResponse = serde_json::from_str("\"brand_new_rig\"").unwrap();
        assert_eq!(got, RigTypeResponse::Unknown);
    }

    #[test]
    fn output_format_uppercase_wire() {
        assert_eq!(
            serde_json::to_string(&OutputFormat::ThreeMf).unwrap(),
            "\"3MF\""
        );
        let gltf: OutputFormat = serde_json::from_str("\"GLTF\"").unwrap();
        assert_eq!(gltf, OutputFormat::Gltf);
    }

    #[test]
    fn export_orientation_sign_prefix() {
        assert_eq!(
            serde_json::to_string(&ExportOrientation::MinusY).unwrap(),
            "\"-y\""
        );
    }
}
