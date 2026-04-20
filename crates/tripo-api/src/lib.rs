#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! Async Rust client for the Tripo 3D Generation API.

mod error;
mod envelope;
mod types; // will be created in Task 4 — add the `mod types;` stub now so error.rs compiles

pub use error::{Error, Result};
