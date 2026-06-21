//! Shared clap `value_parser` helpers for enums reused across variants.
//!
//! Single-variant parsers stay private to their defining module.

use tripo_api::enums::{
    GeometryQuality, Orientation, RigOutputFormat, TextureAlignment, TextureQuality,
};

pub fn texture_quality(s: &str) -> Result<TextureQuality, String> {
    match s {
        "standard" => Ok(TextureQuality::Standard),
        "detailed" => Ok(TextureQuality::Detailed),
        "extreme" => Ok(TextureQuality::Extreme),
        o => Err(format!(
            "invalid texture quality `{o}` — use standard|detailed|extreme"
        )),
    }
}

pub fn geometry_quality(s: &str) -> Result<GeometryQuality, String> {
    match s {
        "standard" => Ok(GeometryQuality::Standard),
        "detailed" => Ok(GeometryQuality::Detailed),
        o => Err(format!(
            "invalid geometry quality `{o}` — use standard|detailed"
        )),
    }
}

pub fn texture_alignment(s: &str) -> Result<TextureAlignment, String> {
    match s {
        "original_image" => Ok(TextureAlignment::OriginalImage),
        "geometry" => Ok(TextureAlignment::Geometry),
        o => Err(format!("invalid alignment `{o}`")),
    }
}

pub fn orientation(s: &str) -> Result<Orientation, String> {
    match s {
        "default" => Ok(Orientation::Default),
        "align_image" => Ok(Orientation::AlignImage),
        o => Err(format!("invalid orientation `{o}`")),
    }
}

pub fn rig_out_format(s: &str) -> Result<RigOutputFormat, String> {
    match s {
        "glb" => Ok(RigOutputFormat::Glb),
        "fbx" => Ok(RigOutputFormat::Fbx),
        o => Err(format!("invalid out_format `{o}`")),
    }
}
