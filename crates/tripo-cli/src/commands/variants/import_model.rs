//! `import-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::{ImageInput, ImportModelRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Import an external model file (GLB/GLTF/FBX/OBJ/STL, up to 150 MB).
#[derive(Debug, Args)]
pub struct ImportModelArgs {
    /// URL, `file_token`, or local path.
    #[arg(long)]
    pub input: String,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for ImportModelArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::ImportModel(ImportModelRequest {
            input: ImageInput::parse(&self.input),
        }))
    }
}
