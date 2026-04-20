// Expanded in Tasks 6–9.

use clap::Subcommand;

/// `task` subcommands: raw task escape hatches.
#[derive(Debug, Subcommand)]
pub enum TaskCommand {
    /// POST arbitrary JSON to /task.
    Create {
        /// Path to the JSON request file (or `-` for stdin).
        #[arg(long)]
        json: std::path::PathBuf,
    },
    /// Fetch a task's current state.
    Get {
        /// Task identifier.
        task_id: String,
    },
    /// Wait for a task to reach a terminal status.
    Wait {
        /// Task identifier.
        task_id: String,
        /// Overall timeout in seconds; no limit by default.
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Download a task's output models into `--output`.
    Download {
        /// Task identifier.
        task_id: String,
        /// Output directory.
        #[arg(long, short = 'o')]
        output: std::path::PathBuf,
    },
}
