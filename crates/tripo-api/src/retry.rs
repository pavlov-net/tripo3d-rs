//! Retry policy with exponential backoff + full jitter.

use std::time::Duration;

use reqwest::StatusCode;

/// Controls retry behavior.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum retry attempts beyond the initial request (`max_attempts = 3` → up to 4 total sends).
    pub max_attempts: u32,
    /// Base delay before the first retry.
    pub base_delay: Duration,
    /// Maximum delay between retries.
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

/// Outcome of inspecting one response for retry eligibility.
#[derive(Debug, Clone, Copy)]
pub(crate) enum RetryDecision {
    /// Retry after this delay.
    Retry(Duration),
    /// Do not retry — terminal.
    Stop,
}

impl RetryPolicy {
    /// Decide whether a status code is retryable.
    pub(crate) fn decide_status(
        &self,
        attempt: u32,
        status: StatusCode,
        retry_after: Option<Duration>,
    ) -> RetryDecision {
        if attempt >= self.max_attempts {
            return RetryDecision::Stop;
        }
        match status.as_u16() {
            429 => RetryDecision::Retry(retry_after.unwrap_or_else(|| self.backoff(attempt))),
            500..=599 => RetryDecision::Retry(self.backoff(attempt)),
            _ => RetryDecision::Stop,
        }
    }

    /// Decide whether a transport error is retryable.
    ///
    /// Only connect-time and timeout errors are retried — both are safe because
    /// either no bytes reached the server, or the client didn't observe the
    /// server's state change. Post-connect errors (partial body send, broken
    /// streams) are not retried to avoid duplicate side-effects on non-idempotent
    /// endpoints like `POST /task`.
    pub(crate) fn decide_transport(&self, attempt: u32, err: &reqwest::Error) -> RetryDecision {
        if attempt >= self.max_attempts {
            return RetryDecision::Stop;
        }
        // Retry only on connect/timeout — these are safe because no bytes reached the server
        // (or if they did, the server's state change hasn't been observed). Post-connect errors
        // (e.g. partial body send) might have side effects, so we stop.
        if err.is_connect() || err.is_timeout() {
            RetryDecision::Retry(self.backoff(attempt))
        } else {
            RetryDecision::Stop
        }
    }

    /// Exponential backoff with full jitter.
    fn backoff(&self, attempt: u32) -> Duration {
        let exp = 2u64.saturating_pow(attempt);
        let factor = u32::try_from(exp).unwrap_or(u32::MAX);
        let max = (self.base_delay * factor).min(self.max_delay);
        // Full jitter: sample uniformly in [0, max].
        let nanos = u64::try_from(max.as_nanos()).unwrap_or(u64::MAX);
        let jitter = rand_nanos(nanos);
        Duration::from_nanos(jitter)
    }
}

/// Deterministic-enough jitter source that avoids pulling in `rand`.
fn rand_nanos(max: u64) -> u64 {
    use std::time::SystemTime;
    if max == 0 {
        return 0;
    }
    let n = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX));
    n % max
}

/// Parse `Retry-After` header value (either seconds or HTTP date — seconds only for now).
pub(crate) fn parse_retry_after(v: &reqwest::header::HeaderValue) -> Option<Duration> {
    let s = v.to_str().ok()?;
    s.trim().parse::<u64>().ok().map(Duration::from_secs)
}
