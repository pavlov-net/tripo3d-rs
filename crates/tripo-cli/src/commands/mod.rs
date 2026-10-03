//! Dispatch from `Command` to subcommand runners.

pub mod balance;
pub mod completions;
pub mod task;
pub mod upload;
pub mod usage;
pub mod variants;

use crate::cli::{Cli, Command};

/// Dispatch to the matching subcommand runner.
pub async fn dispatch(args: Cli) -> anyhow::Result<()> {
    let g = &args.global;
    match args.command {
        Command::Balance => balance::run(g).await,
        Command::Usage(a) => usage::run(g, a).await,
        Command::Upload(a) => upload::run(g, a).await,
        Command::Completions(a) => completions::run(&a),
        Command::Task(cmd) => task::run(g, cmd).await,
        Command::TextToModel(a) => variants::run_variant(g, a).await,
        Command::ImageToModel(a) => variants::run_variant(g, a).await,
        Command::MultiviewToModel(a) => variants::run_variant(g, a).await,
        Command::TextToImage(a) => variants::run_variant(g, a).await,
        Command::ImageToImage(a) => variants::run_variant(g, a).await,
        Command::ImageToMultiview(a) => variants::run_variant(g, a).await,
        Command::EditMultiview(a) => variants::run_variant(g, a).await,
        Command::ImageToSplat(a) => variants::run_variant(g, a).await,
        Command::ConvertModel(a) => variants::run_variant(g, a).await,
        Command::StylizeModel(a) => variants::run_variant(g, a).await,
        Command::TextureModel(a) => variants::run_variant(g, a).await,
        Command::RefineModel(a) => variants::run_variant(g, a).await,
        Command::ImportModel(a) => variants::run_variant(g, a).await,
        Command::CheckRiggable(a) => variants::run_variant(g, a).await,
        Command::RigModel(a) => variants::run_variant(g, a).await,
        Command::RetargetAnimation(a) => variants::run_variant(g, a).await,
        Command::MeshSegmentation(a) => variants::run_variant(g, a).await,
        Command::MeshSmartSegment(a) => variants::run_variant(g, a).await,
        Command::MeshCompletion(a) => variants::run_variant(g, a).await,
        Command::MeshDecimate(a) => variants::run_variant(g, a).await,
    }
}
