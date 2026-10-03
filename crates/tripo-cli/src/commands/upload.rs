//! `upload` subcommand.

use anyhow::Result;
use clap::Args;

use crate::cli::GlobalArgs;

/// Arguments for the `upload` subcommand.
#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Path to the file.
    pub path: std::path::PathBuf,
    /// Upload through a presigned storage URL regardless of size. Files over
    /// 60 MiB always use it. Requires one of the extensions jpeg, jpg, png,
    /// webp, bmp, tiff, glb, gltf, fbx, obj, stl, 3mf, usdz.
    #[arg(long)]
    pub presign: bool,
}

/// Run `upload`: upload the file, print the resulting `file_token`.
pub async fn run(g: &GlobalArgs, a: UploadArgs) -> Result<()> {
    let client = crate::resolve::build_client(g)?;
    let up = if a.presign {
        client.upload_file_presigned(&a.path).await?
    } else {
        client.upload_file(&a.path).await?
    };
    if g.json {
        serde_json::to_writer_pretty(std::io::stdout(), &up)?;
        println!();
    } else {
        println!("{}", up.file_token);
    }
    Ok(())
}
