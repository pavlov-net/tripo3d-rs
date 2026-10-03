//! Webhook signature verification and typed event payloads.
//!
//! Tripo signs each delivery with the endpoint's signing secret (`whsec_…`,
//! shown once when the endpoint is created under Settings → Webhooks). The
//! [`HEADER_SIGNATURE`] header has the form `t=<unix>,v1=<hex>`, where `v1` is
//! the lowercase hex HMAC-SHA256 of `"{t}.{raw_body}"` keyed with the secret
//! string's UTF-8 bytes, prefix included.
//!
//! Verify against the raw request body bytes, before any JSON parsing:
//!
//! ```
//! use std::time::SystemTime;
//! use tripo_api::webhook::{self, WebhookEventData};
//!
//! fn handle(secret: &str, signature: &str, body: &[u8]) -> Result<(), webhook::WebhookError> {
//!     let event = webhook::verify_and_parse(
//!         secret,
//!         signature,
//!         body,
//!         Some(webhook::DEFAULT_TOLERANCE),
//!         SystemTime::now(),
//!     )?;
//!     match event.data {
//!         WebhookEventData::TaskCompleted(task) => println!("{} done", task.task_id),
//!         WebhookEventData::TaskFailed(task) => println!("{} failed", task.task_id),
//!         WebhookEventData::BalanceLow(low) => println!("balance {}", low.balance),
//!         other => println!("ignoring {}", other.event_type()),
//!     }
//!     Ok(())
//! }
//! ```

use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::types::{TaskId, TaskOutput, TaskStatus};

/// Header carrying the webhook endpoint id on Tripo.
pub const HEADER_ID: &str = "Tripo-Webhook-Id";
/// Header carrying the unique delivery id. Use it to deduplicate redeliveries.
pub const HEADER_DELIVERY: &str = "Tripo-Webhook-Delivery";
/// Header carrying the event type; same as the body's `type`.
pub const HEADER_EVENT: &str = "Tripo-Webhook-Event";
/// Header carrying the signature, `t=<unix>,v1=<hex>`.
pub const HEADER_SIGNATURE: &str = "Tripo-Webhook-Signature";

/// Event type sent when a task finishes successfully.
pub const EVENT_TASK_COMPLETED: &str = "task.completed";
/// Event type sent when a task fails or is cancelled.
pub const EVENT_TASK_FAILED: &str = "task.failed";
/// Event type sent when the balance drops below the Billing credit alert
/// threshold.
pub const EVENT_BALANCE_LOW: &str = "balance.low";

/// Replay window Tripo's reference implementations use: 5 minutes either side
/// of the current time.
pub const DEFAULT_TOLERANCE: Duration = Duration::from_mins(5);

/// Length in hex characters of an HMAC-SHA256 signature.
const SIGNATURE_HEX_LEN: usize = 64;

/// Reasons a webhook delivery fails verification or parsing.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum WebhookError {
    /// The signature header is not a comma-separated list of `key=value`
    /// pairs, repeats `t`, or has a `t` that is not Unix seconds.
    #[error("malformed webhook signature header: {0}")]
    MalformedHeader(String),

    /// The signature header has no `t=` entry.
    #[error("webhook signature header has no timestamp (`t=`)")]
    MissingTimestamp,

    /// The signature header has no `v1=` entry.
    #[error("webhook signature header has no signature (`v1=`)")]
    MissingSignature,

    /// No `v1` entry matches the HMAC computed with the secret.
    #[error("webhook signature does not match")]
    SignatureMismatch,

    /// The signed timestamp is further from `now` than the tolerance allows.
    #[error("webhook timestamp {timestamp} is outside the {tolerance:?} tolerance")]
    TimestampOutsideTolerance {
        /// Unix seconds from the `t=` entry.
        timestamp: u64,
        /// Tolerance that was applied.
        tolerance: Duration,
    },

    /// The body is not a valid webhook event.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Verify a delivery's [`HEADER_SIGNATURE`] against its raw body.
///
/// `secret` is the endpoint's full signing secret, `whsec_` prefix included.
/// Every `v1=` entry is tried, so a header carrying signatures from more than
/// one secret verifies if any matches; unknown keys are ignored. Comparison
/// is constant-time.
///
/// With `tolerance` set, the signed timestamp must be within that distance of
/// `now` in either direction; [`DEFAULT_TOLERANCE`] matches Tripo's samples.
/// `None` skips the replay check.
pub fn verify_signature(
    secret: &str,
    signature_header: &str,
    raw_body: &[u8],
    tolerance: Option<Duration>,
    now: SystemTime,
) -> Result<(), WebhookError> {
    let header = SignatureHeader::parse(signature_header)?;

    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(header.timestamp.as_bytes());
    mac.update(b".");
    mac.update(raw_body);
    let matched = header
        .signatures
        .iter()
        .filter_map(|sig| decode_lower_hex(sig))
        .any(|sig| mac.clone().verify_slice(&sig).is_ok());
    if !matched {
        return Err(WebhookError::SignatureMismatch);
    }

    if let Some(tolerance) = tolerance {
        let now = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        if now.abs_diff(Duration::from_secs(header.unix_secs)) > tolerance {
            return Err(WebhookError::TimestampOutsideTolerance {
                timestamp: header.unix_secs,
                tolerance,
            });
        }
    }
    Ok(())
}

/// [`verify_signature`], then parse the body as a [`WebhookEvent`].
pub fn verify_and_parse(
    secret: &str,
    signature_header: &str,
    raw_body: &[u8],
    tolerance: Option<Duration>,
    now: SystemTime,
) -> Result<WebhookEvent, WebhookError> {
    verify_signature(secret, signature_header, raw_body, tolerance, now)?;
    Ok(serde_json::from_slice(raw_body)?)
}

/// Parsed `t=<unix>,v1=<hex>[,v1=<hex>…]` header.
struct SignatureHeader<'a> {
    /// The `t` value verbatim; it is part of the signed payload.
    timestamp: &'a str,
    /// The `t` value as Unix seconds.
    unix_secs: u64,
    signatures: Vec<&'a str>,
}

impl<'a> SignatureHeader<'a> {
    fn parse(header: &'a str) -> Result<Self, WebhookError> {
        let mut timestamp = None;
        let mut signatures = Vec::new();
        for part in header.split(',').map(str::trim) {
            let (key, value) = part
                .split_once('=')
                .ok_or_else(|| WebhookError::MalformedHeader(format!("entry {part:?}")))?;
            match key {
                "t" if timestamp.is_some() => {
                    return Err(WebhookError::MalformedHeader("repeated `t`".into()));
                }
                "t" => timestamp = Some(value),
                "v1" => signatures.push(value),
                _ => {}
            }
        }
        let timestamp = timestamp.ok_or(WebhookError::MissingTimestamp)?;
        let unix_secs = timestamp
            .parse()
            .map_err(|_| WebhookError::MalformedHeader(format!("timestamp {timestamp:?}")))?;
        if signatures.is_empty() {
            return Err(WebhookError::MissingSignature);
        }
        Ok(Self {
            timestamp,
            unix_secs,
            signatures,
        })
    }
}

/// Decode a 64-character lowercase hex signature. Anything else cannot equal
/// the lowercase hex digest the reference implementations compare against.
fn decode_lower_hex(s: &str) -> Option<[u8; SIGNATURE_HEX_LEN / 2]> {
    fn nibble(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            _ => None,
        }
    }
    let bytes = s.as_bytes();
    if bytes.len() != SIGNATURE_HEX_LEN {
        return None;
    }
    let mut out = [0u8; SIGNATURE_HEX_LEN / 2];
    for (byte, pair) in out.iter_mut().zip(bytes.chunks_exact(2)) {
        *byte = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Some(out)
}

/// Webhook request body.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "RawWebhookEvent")]
pub struct WebhookEvent {
    /// Composite event id, e.g. `task_abc123:task.completed`.
    pub id: String,
    /// ISO 8601 time the event was created.
    pub created_at: String,
    /// Event type and its payload.
    pub data: WebhookEventData,
}

impl WebhookEvent {
    /// Wire-format event type, e.g. `task.completed`.
    #[must_use]
    pub fn event_type(&self) -> &str {
        self.data.event_type()
    }
}

/// Event payload, keyed by the body's `type`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WebhookEventData {
    /// `task.completed`: the task finished successfully.
    TaskCompleted(TaskEventData),
    /// `task.failed`: the task failed or was cancelled.
    TaskFailed(TaskEventData),
    /// `balance.low`: the balance dropped below the credit alert threshold.
    BalanceLow(BalanceLowData),
    /// An event type this crate does not model, passed through as-is.
    Unknown {
        /// Wire-format event type.
        event_type: String,
        /// Raw `data` object.
        data: serde_json::Value,
    },
}

impl WebhookEventData {
    /// Wire-format event type, e.g. `task.completed`.
    #[must_use]
    pub fn event_type(&self) -> &str {
        match self {
            Self::TaskCompleted(_) => EVENT_TASK_COMPLETED,
            Self::TaskFailed(_) => EVENT_TASK_FAILED,
            Self::BalanceLow(_) => EVENT_BALANCE_LOW,
            Self::Unknown { event_type, .. } => event_type,
        }
    }
}

/// `data` of `task.completed` and `task.failed` events.
///
/// It mirrors a subset of [`crate::Task`], but failures carry an `error`
/// object with a string `code`, which the task query API does not return,
/// so it is a separate type.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskEventData {
    /// Task identifier.
    pub task_id: TaskId,
    /// Wire-format task type string (e.g. `text_to_model`).
    #[serde(rename = "type")]
    pub task_type: String,
    /// Terminal status.
    pub status: TaskStatus,
    /// Output URLs; present on success.
    #[serde(default)]
    pub output: Option<TaskOutput>,
    /// Failure reason; present on failure.
    #[serde(default)]
    pub error: Option<TaskEventError>,
    /// Other task fields Tripo includes.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Failure reason on a `task.failed` event.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskEventError {
    /// Machine-readable code, e.g. `task_failed`.
    #[serde(default)]
    pub code: String,
    /// Human-readable message.
    #[serde(default)]
    pub message: String,
}

/// `data` of a `balance.low` event.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceLowData {
    /// Current credit balance.
    pub balance: f64,
    /// Credit alert threshold configured under Billing.
    pub threshold: f64,
    /// Other balance fields Tripo includes.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct RawWebhookEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    created_at: String,
    data: serde_json::Value,
}

impl TryFrom<RawWebhookEvent> for WebhookEvent {
    type Error = serde_json::Error;

    fn try_from(raw: RawWebhookEvent) -> Result<Self, Self::Error> {
        let data = match raw.event_type.as_str() {
            EVENT_TASK_COMPLETED => {
                WebhookEventData::TaskCompleted(serde_json::from_value(raw.data)?)
            }
            EVENT_TASK_FAILED => WebhookEventData::TaskFailed(serde_json::from_value(raw.data)?),
            EVENT_BALANCE_LOW => WebhookEventData::BalanceLow(serde_json::from_value(raw.data)?),
            _ => WebhookEventData::Unknown {
                event_type: raw.event_type,
                data: raw.data,
            },
        };
        Ok(Self {
            id: raw.id,
            created_at: raw.created_at,
            data,
        })
    }
}
