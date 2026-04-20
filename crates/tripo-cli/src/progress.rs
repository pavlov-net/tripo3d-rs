//! Indicatif progress bar adapter for `wait_for_task`.

use indicatif::{ProgressBar, ProgressStyle};

use tripo_api::{ProgressCallback, Task};

/// Build a stderr-bound progress bar + matching `ProgressCallback`.
fn spawn_progress_bar(task_id: &str) -> (ProgressBar, ProgressCallback) {
    let bar = ProgressBar::new(100).with_style(
        ProgressStyle::with_template("{prefix} [{bar:20.cyan/blue}] {pos:>3}%  {msg}")
            .expect("valid template")
            .progress_chars("=> "),
    );
    bar.set_prefix(format!("task {task_id}"));
    bar.set_draw_target(indicatif::ProgressDrawTarget::stderr());
    let bar_clone = bar.clone();
    let cb: ProgressCallback = Box::new(move |t: &Task| {
        bar_clone.set_position(u64::try_from(t.progress.clamp(0, 100)).unwrap_or(0));
        let msg = match (t.status, t.running_left_time) {
            (_, Some(eta)) => format!("{:?}, ~{eta}s", t.status),
            (s, None) => format!("{s:?}"),
        };
        bar_clone.set_message(msg);
    });
    (bar, cb)
}

/// JSON per-poll update for non-TTY.
fn spawn_json_progress() -> ProgressCallback {
    Box::new(|t: &Task| {
        let line = serde_json::json!({
            "task_id": t.task_id,
            "status":  t.status,
            "progress": t.progress,
            "running_left_time": t.running_left_time,
        });
        eprintln!("{line}");
    })
}

/// Pick the right callback for the environment.
pub fn select_callback(task_id: &str, tty: bool) -> (Option<ProgressBar>, ProgressCallback) {
    if tty {
        let (bar, cb) = spawn_progress_bar(task_id);
        (Some(bar), cb)
    } else {
        (None, spawn_json_progress())
    }
}

/// Finalize the bar (if any) with a status message.
pub fn bar_finish(b: Option<&ProgressBar>, status: Option<tripo_api::TaskStatus>) {
    if let Some(b) = b {
        let msg = match status {
            Some(s) => format!("{s:?}"),
            None => String::new(),
        };
        b.finish_with_message(msg);
    }
}
