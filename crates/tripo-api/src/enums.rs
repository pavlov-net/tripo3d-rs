//! Shared typed enums used across multiple request structs.
//!
//! Each enum carries an `Unknown(String)` variant via `#[serde(other)]` so
//! forward-compatible servers don't break deserialization when new values are added.

use serde::{Deserialize, Serialize};

macro_rules! string_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($(#[$vmeta:meta])* $variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Biological rig classification.
    pub enum RigType {
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
    /// Texture / geometry quality level.
    pub enum Quality {
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
    /// Export orientation vector (`convert_model`).
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
        for s in [PostStyle::Lego, PostStyle::Voxel, PostStyle::Voronoi, PostStyle::Minecraft] {
            let j = serde_json::to_string(&s).unwrap();
            let back: PostStyle = serde_json::from_str(&j).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn unknown_variant_accepted_on_deserialize() {
        let got: PostStyle = serde_json::from_str("\"brand_new_style\"").unwrap();
        assert_eq!(got, PostStyle::Unknown);
    }

    #[test]
    fn output_format_uppercase_wire() {
        assert_eq!(serde_json::to_string(&OutputFormat::ThreeMf).unwrap(), "\"3MF\"");
        let gltf: OutputFormat = serde_json::from_str("\"GLTF\"").unwrap();
        assert_eq!(gltf, OutputFormat::Gltf);
    }

    #[test]
    fn export_orientation_sign_prefix() {
        assert_eq!(serde_json::to_string(&ExportOrientation::MinusY).unwrap(), "\"-y\"");
    }
}
