#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! Async Rust client for the Tripo 3D Generation API.

mod error;
mod envelope;
mod image;
pub mod enums;
pub mod types;
pub mod versions;

pub use error::{Error, Result};
pub use image::ImageInput;
pub use types::{Balance, Task, TaskId, TaskOutput, TaskStatus, UploadedFile};
pub use enums::{
    Animation, ExportOrientation, FbxPreset, Orientation, OutputFormat, PostStyle, Quality,
    RigOutputFormat, RigSpec, RigType, TextureAlignment, TextureFormat,
};
