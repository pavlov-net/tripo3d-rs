//! Top-level `clap` parser.

use clap::{Parser, Subcommand};

/// Command-line client for the Tripo 3D Generation API.
#[derive(Debug, Parser)]
#[command(name = "tripo", version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

/// Global flags that apply to every subcommand.
#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)] // CLI flag struct
pub struct GlobalArgs {
    /// Tripo API key. Must begin with `tsk_`.
    #[arg(long, env = "TRIPO_API_KEY", global = true, hide_env_values = true)]
    pub api_key: Option<String>,

    /// API region.
    #[arg(long, env = "TRIPO_REGION", value_enum, global = true, default_value_t = Region::Global)]
    pub region: Region,

    /// Override the API base URL (testing / staging).
    #[arg(long, global = true)]
    pub base_url: Option<url::Url>,

    /// Emit structured JSON instead of human-readable tables (default if stdout is not a TTY).
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable ANSI color output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Increase log verbosity (-v = debug, -vv = trace).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Silence info-level logs.
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Overwrite existing files at output paths instead of erroring.
    #[arg(long, global = true)]
    pub force: bool,
}

/// API region selector, mirroring `tripo_api::Region`.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Region {
    /// Global endpoint (default).
    Global,
    /// China mainland endpoint.
    Cn,
}

impl From<Region> for tripo_api::Region {
    fn from(r: Region) -> Self {
        match r {
            Region::Global => Self::Global,
            Region::Cn => Self::Cn,
        }
    }
}

/// Top-level subcommand.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show account balance.
    Balance,
    /// Upload a file, print the `file_token`.
    Upload(crate::commands::upload::UploadArgs),
    /// Generate shell completions.
    Completions(crate::commands::completions::CompletionsArgs),
    /// Raw task escape hatches: create from JSON, get, wait, download.
    #[command(subcommand)]
    Task(crate::commands::task::TaskCommand),
    /// Generate a 3D model from a text prompt.
    TextToModel(crate::commands::variants::TextToModelArgs),
    /// Generate a 3D model from a single image.
    ImageToModel(crate::commands::variants::ImageToModelArgs),
}
