//! Dispatch from `Command` to subcommand runners.

#![allow(clippy::unused_async)] // Several subcommand `run` stubs become async in later tasks.

pub mod balance;
pub mod completions;
pub mod task;
pub mod upload;

use crate::cli::{Cli, Command};

/// Dispatch to the matching subcommand runner.
pub async fn dispatch(args: Cli) -> anyhow::Result<()> {
    match args.command {
        Command::Balance => balance::run(&args.global).await,
        Command::Upload(a) => upload::run(&args.global, a).await,
        Command::Completions(a) => completions::run(&a),
        Command::Task(cmd) => task::run(&args.global, cmd).await,
    }
}
