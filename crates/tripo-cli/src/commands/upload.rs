//! `upload` subcommand.

use anyhow::Result;
use clap::Args;

use crate::cli::GlobalArgs;

/// Arguments for the `upload` subcommand.
#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Path to the file.
    pub path: std::path::PathBuf,
}

/// Run `upload`: POST the file, print the resulting `file_token`.
pub async fn run(g: &GlobalArgs, a: UploadArgs) -> Result<()> {
    let client = crate::resolve::build_client(g)?;
    let up = client.upload_file(&a.path).await?;
    if g.json {
        serde_json::to_writer_pretty(std::io::stdout(), &up)?;
        println!();
    } else {
        println!("{}", up.file_token);
    }
    Ok(())
}
