//! SIGINT handler → cancellation token.

use std::sync::Arc;
use std::sync::OnceLock;
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a background task that flips the token on the first SIGINT.
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
    pub fn is_cancelled(&self) -> bool {
        self.fired.load(Ordering::SeqCst)
    }

    /// Resolves when the token fires.
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.notify.notified().await;
    }
}

/// Sentinel error type mapped to exit code 130 by `exit::code_for_error`.
#[derive(Debug)]
pub struct Interrupted;

impl std::fmt::Display for Interrupted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("interrupted")
    }
}

impl std::error::Error for Interrupted {}

static GLOBAL: OnceLock<Cancel> = OnceLock::new();

/// Install the global cancel token with a SIGINT listener. Idempotent.
pub fn install_global() {
    GLOBAL.get_or_init(|| {
        let c = Cancel::new();
        c.install();
        c
    });
}

/// Access the global cancel token. Panics if `install_global` wasn't called.
#[must_use]
pub fn global() -> &'static Cancel {
    GLOBAL.get().expect("signals::install_global not called")
}
