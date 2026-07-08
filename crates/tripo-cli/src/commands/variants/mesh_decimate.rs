//! `mesh-decimate` subcommand (retopology; replaces v2 `smart-lowpoly`).

use anyhow::Result;
use clap::Args;
use tripo_api::{MeshDecimateRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Reduce model polycount: smart retopology (v2.0, default) or basic
/// decimation (v1.0, requires `--face-limit`).
#[derive(Debug, Args)]
pub struct MeshDecimateArgs {
    /// Model source: task id, file token, or URL.
    #[arg(long)]
    pub input: String,
    /// Algorithm version (v2.0|v1.0).
    #[arg(long)]
    pub model: Option<String>,
    /// Produce a quad mesh.
    #[arg(long)]
    pub quad: Option<bool>,
    /// Restrict to named parts (comma-separated; v2.0 only).
    #[arg(long, value_delimiter = ',')]
    pub part_names: Option<Vec<String>>,
    /// Target face count (required for v1.0).
    #[arg(long)]
    pub face_limit: Option<i32>,
    /// Bake textures onto the low-poly model (v2.0 only).
    #[arg(long)]
    pub bake: Option<bool>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for MeshDecimateArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::MeshDecimate(MeshDecimateRequest {
            input: self.input,
            model: self.model,
            quad: self.quad,
            part_names: self.part_names,
            face_limit: self.face_limit,
            bake: self.bake,
        }))
    }
}
