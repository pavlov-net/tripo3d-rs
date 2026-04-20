// Expanded in Task 4 (dispatcher uses this).

use clap::Args;
use clap_complete::Shell;

/// Arguments for the `completions` subcommand.
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}
