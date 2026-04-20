#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

//! Async Rust client for the Tripo 3D Generation API.
//!
//! # Features
//!
//! - `schemars` (default off): derive `schemars::JsonSchema` on public types
//!   so `tripo-mcp` can expose them as MCP tool schemas.

mod client;
mod compress;
mod download;
pub mod enums;
mod envelope;
mod error;
mod image;
mod retry;
pub mod tasks;
pub mod types;
mod upload;
pub mod versions;
mod wait;

pub use client::{
    Client, ClientBuilder, Region, API_KEY_ENV, BASE_URL_CN, BASE_URL_GLOBAL, REGION_ENV,
};
pub use compress::CompressionMode;
pub use download::{DownloadOptions, DownloadedFiles, OutputKind};
pub use enums::{
    Animation, ExportOrientation, FbxPreset, Orientation, OutputFormat, PostStyle, Quality,
    RigOutputFormat, RigSpec, RigType, TextureAlignment, TextureFormat,
};
pub use error::{Error, Result};
pub use image::ImageInput;
pub use retry::RetryPolicy;
pub use tasks::{
    CheckRiggableRequest, ConvertModelRequest, ImageToModelRequest, MeshCompletionRequest,
    MeshSegmentationRequest, MultiviewToModelRequest, RefineModelRequest, RetargetAnimationRequest,
    RigModelRequest, SmartLowpolyRequest, StylizeModelRequest, TaskRequest, TextToModelRequest,
    TextureModelRequest, TexturePrompt,
};
pub use types::{Balance, Task, TaskId, TaskOutput, TaskStatus, UploadedFile};
pub use wait::{ProgressCallback, WaitOptions};
