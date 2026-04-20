//! `rig-model` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{RigOutputFormat, RigSpec, RigType};
use tripo_api::{RigModelRequest, TaskRequest};

use crate::cli::GlobalArgs;
use crate::commands::variants::{IntoRequest, VariantRunOpts};

/// Generate a skeletal rig for an existing model.
#[derive(Debug, Args)]
pub struct RigModelArgs {
    /// Source task id.
    #[arg(long)]
    pub original_model_task_id: String,
    /// Model version.
    #[arg(long)]
    pub model_version: Option<String>,
    /// Output file format (glb|fbx).
    #[arg(long, value_parser = parse_out_format)]
    pub out_format: Option<RigOutputFormat>,
    /// Rig classification.
    #[arg(long, value_parser = parse_rig_type)]
    pub rig_type: Option<RigType>,
    /// Skeleton spec.
    #[arg(long, value_parser = parse_rig_spec)]
    pub spec: Option<RigSpec>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

pub(super) fn parse_out_format(s: &str) -> Result<RigOutputFormat, String> {
    Ok(match s {
        "glb" => RigOutputFormat::Glb,
        "fbx" => RigOutputFormat::Fbx,
        o => return Err(format!("invalid out_format `{o}`")),
    })
}

fn parse_rig_type(s: &str) -> Result<RigType, String> {
    Ok(match s {
        "biped" => RigType::Biped,
        "quadruped" => RigType::Quadruped,
        "hexapod" => RigType::Hexapod,
        "octopod" => RigType::Octopod,
        "avian" => RigType::Avian,
        "serpentine" => RigType::Serpentine,
        "aquatic" => RigType::Aquatic,
        "others" => RigType::Others,
        o => return Err(format!("invalid rig_type `{o}`")),
    })
}

fn parse_rig_spec(s: &str) -> Result<RigSpec, String> {
    Ok(match s {
        "mixamo" => RigSpec::Mixamo,
        "tripo" => RigSpec::Tripo,
        o => return Err(format!("invalid rig spec `{o}`")),
    })
}

impl IntoRequest for RigModelArgs {
    fn into_request(self) -> Result<TaskRequest> {
        Ok(TaskRequest::Rig(RigModelRequest {
            original_model_task_id: self.original_model_task_id,
            model_version: self.model_version,
            out_format: self.out_format,
            rig_type: self.rig_type,
            spec: self.spec,
        }))
    }
}

/// Run `rig-model`.
pub async fn run(g: &GlobalArgs, a: RigModelArgs) -> Result<()> {
    let opts = VariantRunOpts {
        wait: a.run.wait || a.run.output.is_some(),
        output: a.run.output.clone(),
        timeout: a.run.timeout,
        poll_interval: a.run.poll_interval,
    };
    let inner = RigModelArgs {
        run: VariantRunOpts {
            wait: false,
            output: None,
            timeout: None,
            poll_interval: None,
        },
        ..a
    };
    super::run_variant(g, opts, inner).await
}
