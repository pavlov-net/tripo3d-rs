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

async fn wait(g: &GlobalArgs, id: &str, timeout: Option<u64>) -> Result<()> {
    use std::time::Duration;

    use is_terminal::IsTerminal;
    use tripo_api::{TaskStatus, WaitOptions};

    let client = crate::resolve::build_client(g)?;
    let tty = std::io::stderr().is_terminal() && !g.json;
    let (bar, cb) = crate::progress::select_callback(id, tty);

    let opts = WaitOptions {
        timeout: timeout.map(Duration::from_secs),
        on_progress: Some(cb),
        ..Default::default()
    };
    let task = client.wait_for_task(&id.into(), opts).await?;
    crate::progress::bar_finish(bar.as_ref(), Some(task.status));

    serde_json::to_writer_pretty(std::io::stdout(), &task)?;
    println!();
    if task.status != TaskStatus::Success {
        return Err(tripo_api::Error::TaskFailed(task.task_id.clone(), task.status).into());
    }
    Ok(())
}

async fn download(g: &GlobalArgs, id: &str, out_dir: &std::path::Path) -> Result<()> {
    use tripo_api::DownloadOptions;
    let client = crate::resolve::build_client(g)?;
    let task = client.get_task(&id.into()).await?;
    let opts = DownloadOptions {
        overwrite: g.force,
        ..Default::default()
    };
    let files = client.download_task_models(&task, out_dir, opts).await?;
    for p in [
        &files.model,
        &files.base_model,
        &files.pbr_model,
        &files.rendered_image,
    ]
    .into_iter()
    .flatten()
    {
        println!("{}", p.display());
    }
    Ok(())
}

async fn create(_g: &GlobalArgs, _json: &std::path::Path) -> Result<()> {
    anyhow::bail!("task create implemented in Task 9")
}
