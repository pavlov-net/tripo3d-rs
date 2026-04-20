//! `retarget-animation` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{Animation, RigOutputFormat};
use tripo_api::{AnimationInput, RetargetAnimationRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Retarget one or more animations onto a rigged model.
#[derive(Debug, Args)]
pub struct RetargetAnimationArgs {
    /// Source rigged task id.
    #[arg(long)]
    pub original_model_task_id: String,
    /// One or more animation presets (comma-separated). If one: `animation` field. If many: `animations`.
    #[arg(long, value_delimiter = ',', value_parser = parse_animation, required = true)]
    pub animation: Vec<Animation>,
    /// Output file format.
    #[arg(long, value_parser = super::parsers::rig_out_format)]
    pub out_format: Option<RigOutputFormat>,
    /// Bake animation samples.
    #[arg(long)]
    pub bake_animation: Option<bool>,
    /// Export with skinned geometry.
    #[arg(long)]
    pub export_with_geometry: Option<bool>,
    /// Animate in-place.
    #[arg(long)]
    pub animate_in_place: Option<bool>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

fn parse_animation(s: &str) -> Result<Animation, String> {
    use Animation::{
        AquaticMarch, Climb, Dive, Fall, HexapodWalk, Hurt, Idle, Jump, OctopodWalk, QuadrupedWalk,
        Run, SerpentineMarch, Shoot, Slash, Turn, Walk,
    };
    Ok(match s {
        "preset:idle" => Idle,
        "preset:walk" => Walk,
        "preset:run" => Run,
        "preset:dive" => Dive,
        "preset:climb" => Climb,
        "preset:jump" => Jump,
        "preset:slash" => Slash,
        "preset:shoot" => Shoot,
        "preset:hurt" => Hurt,
        "preset:fall" => Fall,
        "preset:turn" => Turn,
        "preset:quadruped:walk" => QuadrupedWalk,
        "preset:hexapod:walk" => HexapodWalk,
        "preset:octopod:walk" => OctopodWalk,
        "preset:serpentine:march" => SerpentineMarch,
        "preset:aquatic:march" => AquaticMarch,
        o => return Err(format!("invalid animation `{o}`")),
    })
}

impl VariantArgs for RetargetAnimationArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(mut self) -> Result<TaskRequest> {
        let animation = if self.animation.len() == 1 {
            AnimationInput::Single(self.animation.remove(0))
        } else {
            AnimationInput::Many(self.animation)
        };
        Ok(TaskRequest::Retarget(RetargetAnimationRequest {
            original_model_task_id: self.original_model_task_id,
            animation,
            out_format: self.out_format,
            bake_animation: self.bake_animation,
            export_with_geometry: self.export_with_geometry,
            animate_in_place: self.animate_in_place,
        }))
    }
}
