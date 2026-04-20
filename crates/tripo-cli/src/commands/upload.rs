// Expanded in Task 5.

use anyhow::Result;
use clap::Args;

use crate::cli::GlobalArgs;

/// Arguments for the `upload` subcommand.
#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Path to the file to upload.
    pub path: std::path::PathBuf,
}

/// Run `upload`. Placeholder — real implementation in Task 5.
pub async fn run(_g: &GlobalArgs, _a: UploadArgs) -> Result<()> {
    anyhow::bail!("upload is implemented in Task 5")
}
