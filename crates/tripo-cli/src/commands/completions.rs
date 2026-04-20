//! `completions` subcommand.

use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::Shell;

/// Arguments for the `completions` subcommand.
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}

/// Run `completions`: emit the requested shell completion script to stdout.
#[allow(clippy::unnecessary_wraps)] // Uniform signature across subcommand runners.
pub fn run(a: &CompletionsArgs) -> Result<()> {
    let mut cmd = crate::cli::Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(a.shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}
