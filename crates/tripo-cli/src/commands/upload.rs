// Expanded in Task 5.

use clap::Args;

/// Arguments for the `upload` subcommand.
#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Path to the file to upload.
    pub path: std::path::PathBuf,
}
