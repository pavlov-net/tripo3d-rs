//! `wait_for_task`: poll until terminal status with ETA-driven backoff.

use std::time::{Duration, Instant};

use crate::client::Client;
use crate::error::{Error, Result};
use crate::types::{Task, TaskId};

/// Callback invoked after each successful poll.
pub type ProgressCallback = Box<dyn Fn(&Task) + Send + Sync>;

/// Options for [`Client::wait_for_task`].
pub struct WaitOptions {
    /// Overall timeout. `None` → wait forever.
    pub timeout: Option<Duration>,
    /// Cap on the polling interval.
    pub max_interval: Duration,
    /// Initial polling interval when no ETA is available.
    pub initial_interval: Duration,
    /// Called after every poll.
    pub on_progress: Option<ProgressCallback>,
}

impl Default for WaitOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            max_interval: Duration::from_secs(30),
            initial_interval: Duration::from_secs(2),
            on_progress: None,
        }
    }
}

impl std::fmt::Debug for WaitOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaitOptions")
            .field("timeout", &self.timeout)
            .field("max_interval", &self.max_interval)
            .field("initial_interval", &self.initial_interval)
            .field("on_progress", &self.on_progress.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

/// Compute the next polling delay given a task.
///
/// Mirrors the Python SDK: if `running_left_time` is present, sleep `max(2s, eta/2)`;
/// otherwise double the previous interval, capped at `max_interval`.
pub(crate) fn next_interval(
    task: &Task,
    previous: Duration,
    initial: Duration,
    max_interval: Duration,
) -> Duration {
    if let Some(eta) = task.running_left_time {
        let eta_secs = u64::try_from(eta.max(0)).unwrap_or(0);
        let half = Duration::from_secs(eta_secs) / 2;
        half.max(initial).min(max_interval)
    } else {
        (previous * 2).min(max_interval)
    }
}

impl Client {
    /// Poll `GET /task/{id}` until the status is terminal or `opts.timeout` is reached.
    ///
    /// Returns the final `Task` even for non-success terminal statuses; callers can check
    /// `task.status`. Use `Error::WaitTimeout` if you want an error on timeout (returned here).
    #[tracing::instrument(skip(self, opts), fields(task_id = %id))]
    pub async fn wait_for_task(&self, id: &TaskId, opts: WaitOptions) -> Result<Task> {
        let started = Instant::now();
        let mut interval = opts.initial_interval;
        loop {
            let task = self.get_task(id).await?;
            if let Some(cb) = &opts.on_progress {
                cb(&task);
            }
            if task.status.is_terminal() {
                return Ok(task);
            }
            interval = next_interval(&task, interval, opts.initial_interval, opts.max_interval);
            if let Some(deadline) = opts.timeout {
                let elapsed = started.elapsed();
                let Some(remaining) = deadline.checked_sub(elapsed) else {
                    return Err(Error::WaitTimeout(id.clone()));
                };
                if remaining.is_zero() {
                    return Err(Error::WaitTimeout(id.clone()));
                }
                let to_sleep = interval.min(remaining);
                tokio::time::sleep(to_sleep).await;
            } else {
                tokio::time::sleep(interval).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::types::{TaskOutput, TaskStatus};

    fn task_with_eta(eta: Option<i64>) -> Task {
        Task {
            task_id: "x".into(),
            task_type: "text_to_model".into(),
            status: TaskStatus::Running,
            input: BTreeMap::new(),
            output: TaskOutput::default(),
            progress: 0,
            create_time: 0,
            running_left_time: eta,
            queuing_num: None,
            error_code: None,
            error_msg: None,
        }
    }

    #[test]
    fn eta_drives_half_of_remaining() {
        let t = task_with_eta(Some(40));
        let d = next_interval(
            &t,
            Duration::from_secs(2),
            Duration::from_secs(2),
            Duration::from_secs(30),
        );
        assert_eq!(d, Duration::from_secs(20));
    }

    #[test]
    fn eta_capped_by_max() {
        let t = task_with_eta(Some(600));
        let d = next_interval(
            &t,
            Duration::from_secs(2),
            Duration::from_secs(2),
            Duration::from_secs(30),
        );
        assert_eq!(d, Duration::from_secs(30));
    }

    #[test]
    fn eta_floor_is_initial() {
        let t = task_with_eta(Some(1));
        let d = next_interval(
            &t,
            Duration::from_secs(2),
            Duration::from_secs(2),
            Duration::from_secs(30),
        );
        assert_eq!(d, Duration::from_secs(2));
    }

    #[test]
    fn without_eta_exponential() {
        let t = task_with_eta(None);
        let d = next_interval(
            &t,
            Duration::from_secs(2),
            Duration::from_secs(2),
            Duration::from_secs(30),
        );
        assert_eq!(d, Duration::from_secs(4));
        let d2 = next_interval(
            &t,
            Duration::from_secs(20),
            Duration::from_secs(2),
            Duration::from_secs(30),
        );
        assert_eq!(d2, Duration::from_secs(30));
    }
}
