//! `task` subcommands: raw task escape hatches.

use anyhow::Result;
use clap::Subcommand;

use crate::cli::GlobalArgs;

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

/// Dispatch to the matching `task` subcommand runner.
pub async fn run(g: &GlobalArgs, cmd: TaskCommand) -> Result<()> {
    match cmd {
        TaskCommand::Get { task_id } => get(g, &task_id).await,
        TaskCommand::Wait { task_id, timeout } => wait(g, &task_id, timeout).await,
        TaskCommand::Download { task_id, output } => download(g, &task_id, &output).await,
        TaskCommand::Create { json } => create(g, &json).await,
    }
}

async fn get(g: &GlobalArgs, id: &str) -> Result<()> {
    let client = crate::resolve::build_client(g)?;
    let task = client.get_task(&id.into()).await?;
    serde_json::to_writer_pretty(std::io::stdout(), &task)?;
    println!();
    Ok(())
}

async fn wait(_g: &GlobalArgs, _id: &str, _timeout: Option<u64>) -> Result<()> {
    anyhow::bail!("task wait implemented in Task 7")
}

async fn download(_g: &GlobalArgs, _id: &str, _out: &std::path::Path) -> Result<()> {
    anyhow::bail!("task download implemented in Task 8")
}

async fn create(_g: &GlobalArgs, _json: &std::path::Path) -> Result<()> {
    anyhow::bail!("task create implemented in Task 9")
}
