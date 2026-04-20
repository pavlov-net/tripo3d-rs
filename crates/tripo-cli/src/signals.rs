//! SIGINT handler → cancellation token.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Notify;

/// A cancellation signal — cheap to clone, fires on the first SIGINT.
#[derive(Clone, Default)]
pub struct Cancel {
    notify: Arc<Notify>,
    fired: Arc<AtomicBool>,
}

impl Cancel {
    /// Create a fresh cancel token.
    #[must_use]
    #[allow(dead_code)] // Wired into main.rs in Task 25.
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a background task that flips the token on the first SIGINT.
    #[allow(dead_code)] // Wired into main.rs in Task 25.
    pub fn install(&self) {
        let c = self.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            c.fired.store(true, Ordering::SeqCst);
            c.notify.notify_waiters();
        });
    }

    /// True if a signal has been received.
    #[must_use]
    #[allow(dead_code)] // Wired into task/variant runners in Task 25.
    pub fn is_cancelled(&self) -> bool {
        self.fired.load(Ordering::SeqCst)
    }

    /// Resolves when the token fires.
    #[allow(dead_code)] // Wired into task/variant runners in Task 25.
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.notify.notified().await;
    }
}
