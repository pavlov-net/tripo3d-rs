//! Shared clap `value_parser` helpers for enums reused across variants.
//!
//! Single-variant parsers stay private to their defining module.

use tripo_api::enums::{Orientation, Quality, RigOutputFormat, TextureAlignment};

pub fn quality(s: &str) -> Result<Quality, String> {
    match s {
        "standard" => Ok(Quality::Standard),
        "detailed" => Ok(Quality::Detailed),
        o => Err(format!("invalid quality `{o}` — use standard|detailed")),
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
