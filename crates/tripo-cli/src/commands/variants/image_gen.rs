//! Flags shared by `text-to-image` and `image-to-image`.

use clap::Args;
use tripo_api::enums::{ImageBackground, ImageOutputFormat, ImageQuality};

use super::parsers::wire_enum;

/// Model and output options common to both image-generation commands.
#[derive(Debug, Args)]
pub struct ImageGenOpts {
    /// Image model (e.g. `seedream_v5`, `chat_image_2.5_flare`).
    #[arg(long)]
    pub model: Option<String>,
    /// Output size: a keyword (`2K`, `4K`) or `WIDTHxHEIGHT`.
    #[arg(long)]
    pub size: Option<String>,
    /// Rendering tier (low|medium|high|xhigh|max); model support per `tripo_api::ImageQuality`.
    #[arg(long, value_parser = wire_enum::<ImageQuality>)]
    pub quality: Option<ImageQuality>,
    /// Background (auto|opaque|transparent); model support per `tripo_api::ImageBackground`.
    #[arg(long, value_parser = wire_enum::<ImageBackground>)]
    pub background: Option<ImageBackground>,
    /// Aspect ratio such as `16:9`; banana models only.
    #[arg(long)]
    pub aspect_ratio: Option<String>,
    /// Output file format (png|jpeg).
    #[arg(long, value_parser = wire_enum::<ImageOutputFormat>)]
    pub output_format: Option<ImageOutputFormat>,
}
