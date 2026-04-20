//! Per-variant subcommand framework.
//!
//! Each variant has its own `Args` struct (typed clap flags) and implements
//! `VariantArgs` to build a `tripo_api::TaskRequest`. The generic `run_variant`
//! function handles submit → (optional) wait → (optional) download.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use is_terminal::IsTerminal;
use tripo_api::{DownloadOptions, TaskRequest, TaskStatus, WaitOptions};

use crate::cli::GlobalArgs;

pub mod check_riggable;
pub mod convert_model;
pub mod image_to_model;
pub mod mesh_completion;
pub mod mesh_segmentation;
pub mod multiview_to_model;
mod parsers;
pub mod refine_model;
pub mod retarget_animation;
pub mod rig_model;
pub mod smart_lowpoly;
pub mod stylize_model;
pub mod text_to_model;
pub mod texture_model;
pub use check_riggable::CheckRiggableArgs;
pub use convert_model::ConvertModelArgs;
pub use image_to_model::ImageToModelArgs;
pub use mesh_completion::MeshCompletionArgs;
pub use mesh_segmentation::MeshSegmentationArgs;
pub use multiview_to_model::MultiviewToModelArgs;
pub use refine_model::RefineModelArgs;
pub use retarget_animation::RetargetAnimationArgs;
pub use rig_model::RigModelArgs;
pub use smart_lowpoly::SmartLowpolyArgs;
pub use stylize_model::StylizeModelArgs;
pub use text_to_model::TextToModelArgs;
pub use texture_model::TextureModelArgs;

/// Shared flags attached to every variant command.
#[derive(Debug, Clone, Default, clap::Args)]
pub struct VariantRunOpts {
    /// Poll until the task reaches a terminal status.
    #[arg(long)]
    pub wait: bool,
    /// Download output files into this directory. Implies `--wait`.
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
    /// Overall timeout in seconds for `--wait`. No limit by default.
    #[arg(long)]
    pub timeout: Option<u64>,
    /// Cap on the polling interval in seconds.
    #[arg(long)]
    pub poll_interval: Option<u64>,
}

/// Per-variant Args able to yield a `TaskRequest` and surrender its
/// `VariantRunOpts` so `run_variant` can drive the lifecycle generically.
pub trait VariantArgs: Sized {
    /// Take ownership of the args' `VariantRunOpts`.
    fn take_run_opts(&mut self) -> VariantRunOpts;
    /// Build the API request body.
    fn into_request(self) -> Result<TaskRequest>;
}

/// Submit → (optional) wait → (optional) download.
pub async fn run_variant<A: VariantArgs>(g: &GlobalArgs, mut args: A) -> Result<()> {
    let opts = args.take_run_opts();
    let client = crate::resolve::build_client(g)?;
    let req = args.into_request()?;
    let id = client.create_task(req).await?;

    // --output implies --wait.
    let wait = opts.wait || opts.output.is_some();

    if !wait {
        if g.json {
            serde_json::to_writer_pretty(std::io::stdout(), &serde_json::json!({"task_id": id}))?;
            println!();
        } else {
            println!("{id}");
        }
        return Ok(());
    }

    let tty = std::io::stderr().is_terminal() && !g.json;
    let (bar, cb) = crate::progress::select_callback(id.as_str(), tty);
    let wait_opts = WaitOptions {
        timeout: opts.timeout.map(Duration::from_secs),
        max_interval: opts
            .poll_interval
            .map_or_else(|| Duration::from_secs(30), Duration::from_secs),
        on_progress: Some(cb),
        ..Default::default()
    };
    let cancel = crate::signals::global();
    let task = tokio::select! {
        res = client.wait_for_task(&id, wait_opts) => res?,
        () = cancel.cancelled() => {
            crate::progress::bar_finish(bar.as_ref(), None);
            eprintln!("interrupted — resume with: tripo task wait {id}");
            return Err(crate::signals::Interrupted.into());
        }
    };
    crate::progress::bar_finish(bar.as_ref(), Some(task.status));

    if task.status != TaskStatus::Success {
        serde_json::to_writer_pretty(std::io::stdout(), &task)?;
        println!();
        return Err(tripo_api::Error::TaskFailed(task.task_id.clone(), task.status).into());
    }

    if let Some(dir) = opts.output.as_ref() {
        let dl = DownloadOptions {
            overwrite: g.force,
            ..Default::default()
        };
        let files = tokio::select! {
            res = client.download_task_models(&task, dir, dl) => res?,
            () = cancel.cancelled() => {
                crate::cleanup::partial_files(dir).await;
                return Err(crate::signals::Interrupted.into());
            }
        };
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
    } else {
        serde_json::to_writer_pretty(std::io::stdout(), &task)?;
        println!();
    }
    Ok(())
}
