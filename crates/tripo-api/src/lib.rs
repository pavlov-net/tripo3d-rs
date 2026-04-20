#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! Async Rust client for the Tripo 3D Generation API.

mod client;
mod upload;
mod download;
mod error;
mod envelope;
mod image;
mod compress;
mod retry;
mod wait;
pub mod enums;
pub mod tasks;
pub mod types;
pub mod versions;

pub use client::{Client, ClientBuilder, Region, API_KEY_ENV, BASE_URL_CN, BASE_URL_GLOBAL, REGION_ENV};
pub use error::{Error, Result};
pub use image::ImageInput;
pub use compress::CompressionMode;
pub use download::{DownloadOptions, DownloadedFiles, OutputKind};
pub use retry::RetryPolicy;
pub use tasks::{
    CheckRiggableRequest, ConvertModelRequest, ImageToModelRequest, MeshCompletionRequest,
    MeshSegmentationRequest, MultiviewToModelRequest, RefineModelRequest, RetargetAnimationRequest,
    RigModelRequest, SmartLowpolyRequest, StylizeModelRequest, TaskRequest, TextToModelRequest,
    TextureModelRequest, TexturePrompt,
};
pub use types::{Balance, Task, TaskId, TaskOutput, TaskStatus, UploadedFile};
pub use wait::{ProgressCallback, WaitOptions};
pub use enums::{
    Animation, ExportOrientation, FbxPreset, Orientation, OutputFormat, PostStyle, Quality,
    RigOutputFormat, RigSpec, RigType, TextureAlignment, TextureFormat,
};
