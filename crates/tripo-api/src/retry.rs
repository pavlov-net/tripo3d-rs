//! Retry policy. Fully implemented in Task 13.

use std::time::Duration;

/// Controls retry behavior for idempotent-or-failing requests.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum retry attempts.
    pub max_attempts: u32,
    /// Initial backoff.
    pub base_delay: Duration,
    /// Maximum backoff between retries.
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
        }
    }
}
