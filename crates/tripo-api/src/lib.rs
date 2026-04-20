#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! Async Rust client for the Tripo 3D Generation API.

mod error;
mod envelope;
mod enums;
pub mod types;

pub use error::{Error, Result};
pub use types::{Balance, Task, TaskId, TaskOutput, TaskStatus, UploadedFile};
