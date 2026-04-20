//! `smart-lowpoly` subcommand (wire: `highpoly_to_lowpoly`).

use anyhow::Result;
use clap::Args;
use tripo_api::{SmartLowpolyRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Reduce a high-poly model to a lowpoly one.
#[derive(Debug, Args)]
pub struct SmartLowpolyArgs {
    /// Source (high-poly) task id.
    #[arg(long)]
    pub original_model_task_id: String,
    /// Model version.
    #[arg(long)]
    pub model_version: Option<String>,
    /// Produce a quad mesh.
    #[arg(long)]
    pub quad: Option<bool>,
    /// Restrict to named parts (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub part_names: Option<Vec<String>>,
    /// Face count limit.
    #[arg(long)]
    pub face_limit: Option<i32>,
    /// Bake textures.
    #[arg(long)]
    pub bake: Option<bool>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for SmartLowpolyArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::SmartLowpoly(SmartLowpolyRequest {
            original_model_task_id: self.original_model_task_id,
            model_version: self.model_version,
            quad: self.quad,
            part_names: self.part_names,
            face_limit: self.face_limit,
            bake: self.bake,
        }))
    }
}
