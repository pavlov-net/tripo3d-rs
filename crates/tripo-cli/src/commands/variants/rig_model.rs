//! `rig-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{RigOutputFormat, RigSpec, RigType};
use tripo_api::{RigModelRequest, TaskRequest};

use super::parsers::wire_enum;
use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Generate a skeletal rig for an existing model.
#[derive(Debug, Args)]
pub struct RigModelArgs {
    /// Model source: task id, file token, or URL.
    #[arg(long)]
    pub input: String,
    /// Model version (v1.0-20240301 default, biped only; v2.5-20260210;
    /// v3.0-20260909, required for `--rig-type others` and the
    /// actorcore/unity/unreal/vrm specs).
    #[arg(long)]
    pub model: Option<String>,
    /// Output file format (glb|fbx).
    #[arg(long, value_parser = super::parsers::rig_out_format)]
    pub out_format: Option<RigOutputFormat>,
    /// Rig classification.
    #[arg(long, value_parser = wire_enum::<RigType>)]
    pub rig_type: Option<RigType>,
    /// Skeleton spec, biped only (tripo|mixamo|actorcore|unity|unreal|vrm).
    #[arg(long, value_parser = wire_enum::<RigSpec>)]
    pub spec: Option<RigSpec>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for RigModelArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::Rig(RigModelRequest {
            input: self.input,
            model: self.model,
            out_format: self.out_format,
            rig_type: self.rig_type,
            spec: self.spec,
        }))
    }
}
