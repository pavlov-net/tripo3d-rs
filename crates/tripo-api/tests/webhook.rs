//! Webhook signature verification and event parsing.
//!
//! Expected signatures were computed outside this crate:
//! `{ printf '1714992000.'; cat <fixture>; } | openssl dgst -sha256 -hmac <secret>`
//! and cross-checked against the Python sample in Tripo's webhook docs.
#![cfg(feature = "webhook")]

use std::time::{Duration, SystemTime};

use tripo_api::TaskStatus;
use tripo_api::webhook::{
    DEFAULT_TOLERANCE, WebhookError, WebhookEventData, verify_and_parse, verify_signature,
};

const SECRET: &str = "whsec_test_secret";
const OTHER_SECRET: &str = "whsec_rotated_secret";
const T: u64 = 1_714_992_000;

const TASK_COMPLETED: &[u8] = include_bytes!("fixtures/webhook/task_completed.json");
const TASK_FAILED: &[u8] = include_bytes!("fixtures/webhook/task_failed.json");
const BALANCE_LOW: &[u8] = include_bytes!("fixtures/webhook/balance_low.json");

const SIG_COMPLETED: &str = "a28aa2c6adab24730075621bac4408be837297c50861481bc70bd6e9f348e7ef";
const SIG_COMPLETED_OTHER: &str =
    "899406e6977fb56b07336901e78e9d366d148079c94362d482edf78e91788c11";
const SIG_FAILED: &str = "6f0cf7dd5cec42ed04749bf35d90c13488d41e1ff7ebc2090979ac51e9b9eb8c";
const SIG_BALANCE: &str = "010413804d5f69f24afe03f3ce5309dd4371da12feeaa523d65bd8ccc6b10859";
const SIG_COMPLETED_MAX_T: &str =
    "fc8fbd1e145a69f6ace6dae2ca10a453b9264366e3d6c2040e5179c991f0de58";

fn at(secs: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
}

fn header(sig: &str) -> String {
    format!("t={T},v1={sig}")
}

fn verify(secret: &str, header: &str, body: &[u8]) -> Result<(), WebhookError> {
    verify_signature(secret, header, body, Some(DEFAULT_TOLERANCE), at(T))
}

#[test]
fn known_answer_vectors_verify() {
    for (body, sig) in [
        (TASK_COMPLETED, SIG_COMPLETED),
        (TASK_FAILED, SIG_FAILED),
        (BALANCE_LOW, SIG_BALANCE),
    ] {
        verify(SECRET, &header(sig), body).unwrap();
    }
    verify(OTHER_SECRET, &header(SIG_COMPLETED_OTHER), TASK_COMPLETED).unwrap();
}

#[test]
fn tampered_body_is_rejected() {
    let mut body = TASK_COMPLETED.to_vec();
    body[0] = b' ';
    let err = verify(SECRET, &header(SIG_COMPLETED), &body).unwrap_err();
    assert!(matches!(err, WebhookError::SignatureMismatch), "{err:?}");
}

#[test]
fn tampered_timestamp_is_rejected() {
    let h = format!("t={},v1={SIG_COMPLETED}", T + 1);
    let err = verify(SECRET, &h, TASK_COMPLETED).unwrap_err();
    assert!(matches!(err, WebhookError::SignatureMismatch), "{err:?}");
}

#[test]
fn wrong_secret_is_rejected() {
    let err = verify(OTHER_SECRET, &header(SIG_COMPLETED), TASK_COMPLETED).unwrap_err();
    assert!(matches!(err, WebhookError::SignatureMismatch), "{err:?}");
}

#[test]
fn uppercase_or_truncated_hex_is_rejected() {
    for sig in [
        SIG_COMPLETED.to_uppercase(),
        SIG_COMPLETED[..62].to_string(),
        format!("{SIG_COMPLETED}00"),
        format!("{}zz", &SIG_COMPLETED[..62]),
    ] {
        let err = verify(SECRET, &header(&sig), TASK_COMPLETED).unwrap_err();
        assert!(
            matches!(err, WebhookError::SignatureMismatch),
            "{sig}: {err:?}"
        );
    }
}

#[test]
fn multiple_v1_entries_accept_any_match() {
    let wrong = "0".repeat(64);
    for h in [
        format!("t={T},v1={wrong},v1={SIG_COMPLETED}"),
        format!("t={T},v1={SIG_COMPLETED},v1={wrong}"),
        format!("t={T},v1={SIG_COMPLETED_OTHER},v1={SIG_COMPLETED}"),
        format!("v1={SIG_COMPLETED},t={T}"),
        format!("t={T}, v0=ignored, v1={SIG_COMPLETED}"),
    ] {
        verify(SECRET, &h, TASK_COMPLETED).unwrap_or_else(|e| panic!("{h}: {e:?}"));
    }
    let h = format!("t={T},v1={wrong},v1={SIG_COMPLETED_OTHER}");
    let err = verify(SECRET, &h, TASK_COMPLETED).unwrap_err();
    assert!(matches!(err, WebhookError::SignatureMismatch), "{err:?}");
}

#[test]
fn bad_headers_are_rejected() {
    for h in [
        "",
        "garbage",
        "t=1714992000;v1=ab",
        "t=abc,v1=00",
        "t=-5,v1=00",
        "t=1,t=2,v1=00",
    ] {
        let err = verify(SECRET, h, TASK_COMPLETED).unwrap_err();
        assert!(
            matches!(err, WebhookError::MalformedHeader(_)),
            "{h:?}: {err:?}"
        );
    }
    let err = verify(SECRET, "v1=00", TASK_COMPLETED).unwrap_err();
    assert!(matches!(err, WebhookError::MissingTimestamp), "{err:?}");
    for h in ["t=1714992000", "t=1714992000,v0=00"] {
        let err = verify(SECRET, h, TASK_COMPLETED).unwrap_err();
        assert!(
            matches!(err, WebhookError::MissingSignature),
            "{h:?}: {err:?}"
        );
    }
}

#[test]
fn timestamp_tolerance_is_inclusive_and_symmetric() {
    let h = header(SIG_COMPLETED);
    for now in [T - 300, T + 300] {
        verify_signature(SECRET, &h, TASK_COMPLETED, Some(DEFAULT_TOLERANCE), at(now)).unwrap();
    }
    for now in [T - 301, T + 301] {
        let err = verify_signature(SECRET, &h, TASK_COMPLETED, Some(DEFAULT_TOLERANCE), at(now))
            .unwrap_err();
        assert!(
            matches!(
                err,
                WebhookError::TimestampOutsideTolerance { timestamp: T, tolerance }
                    if tolerance == DEFAULT_TOLERANCE
            ),
            "{now}: {err:?}"
        );
    }
}

#[test]
fn signature_is_checked_before_timestamp() {
    let h = header(&"0".repeat(64));
    let err =
        verify_signature(SECRET, &h, TASK_COMPLETED, Some(DEFAULT_TOLERANCE), at(0)).unwrap_err();
    assert!(matches!(err, WebhookError::SignatureMismatch), "{err:?}");
}

#[test]
fn far_future_timestamp_is_outside_tolerance() {
    // Signed with SECRET over "18446744073709551615." + task_completed.json.
    let h = format!("t={},v1={SIG_COMPLETED_MAX_T}", u64::MAX);
    let err = verify(SECRET, &h, TASK_COMPLETED).unwrap_err();
    assert!(
        matches!(
            err,
            WebhookError::TimestampOutsideTolerance {
                timestamp: u64::MAX,
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn tolerance_disabled_accepts_old_timestamps() {
    verify_signature(
        SECRET,
        &header(SIG_COMPLETED),
        TASK_COMPLETED,
        None,
        at(T + 86_400 * 365),
    )
    .unwrap();
}

#[test]
fn parses_task_completed() {
    let event =
        verify_and_parse(SECRET, &header(SIG_COMPLETED), TASK_COMPLETED, None, at(T)).unwrap();
    assert_eq!(event.id, "task_abc123:task.completed");
    assert_eq!(event.created_at, "2026-05-06T12:00:00Z");
    assert_eq!(event.event_type(), "task.completed");
    let WebhookEventData::TaskCompleted(task) = event.data else {
        panic!("{:?}", event.data);
    };
    assert_eq!(task.task_id.as_str(), "task_abc123");
    assert_eq!(task.task_type, "text_to_model");
    assert_eq!(task.status, TaskStatus::Success);
    assert_eq!(
        task.output.unwrap().model_url.as_deref(),
        Some("https://cdn.tripo3d.ai/output/model_pbr.glb")
    );
    assert!(task.error.is_none());
    assert!(task.extra.is_empty());
}

#[test]
fn parses_task_failed() {
    let event = verify_and_parse(SECRET, &header(SIG_FAILED), TASK_FAILED, None, at(T)).unwrap();
    assert_eq!(event.id, "task_abc123:task.failed");
    assert_eq!(event.created_at, "2026-05-06T12:00:05Z");
    assert_eq!(event.event_type(), "task.failed");
    let WebhookEventData::TaskFailed(task) = event.data else {
        panic!("{:?}", event.data);
    };
    assert_eq!(task.task_id.as_str(), "task_abc123");
    assert_eq!(task.status, TaskStatus::Failed);
    assert!(task.output.is_none());
    let error = task.error.unwrap();
    assert_eq!(error.code, "task_failed");
    assert_eq!(error.message, "Generation failed");
}

#[test]
fn parses_balance_low() {
    let event = verify_and_parse(SECRET, &header(SIG_BALANCE), BALANCE_LOW, None, at(T)).unwrap();
    assert_eq!(event.id, "evt_balance_low:balance.low");
    assert_eq!(event.event_type(), "balance.low");
    let WebhookEventData::BalanceLow(low) = event.data else {
        panic!("{:?}", event.data);
    };
    assert!((low.balance - 1200.0).abs() < f64::EPSILON);
    assert!((low.threshold - 5000.0).abs() < f64::EPSILON);
}

#[test]
fn unknown_event_type_passes_through() {
    let body = br#"{"id":"x:task.queued","type":"task.queued","created_at":"2026-05-06T12:00:00Z","data":{"task_id":"x"}}"#;
    let event: tripo_api::webhook::WebhookEvent = serde_json::from_slice(body).unwrap();
    assert_eq!(event.event_type(), "task.queued");
    let WebhookEventData::Unknown { event_type, data } = event.data else {
        panic!("{:?}", event.data);
    };
    assert_eq!(event_type, "task.queued");
    assert_eq!(data["task_id"], "x");
}

#[test]
fn task_event_keeps_extra_fields() {
    let body = br#"{"id":"t:task.completed","type":"task.completed","created_at":"","data":{"task_id":"t","type":"text_to_model","status":"success","progress":100,"credits_consumed":30.0}}"#;
    let event: tripo_api::webhook::WebhookEvent = serde_json::from_slice(body).unwrap();
    let WebhookEventData::TaskCompleted(task) = event.data else {
        panic!("{:?}", event.data);
    };
    assert_eq!(task.extra["progress"], 100);
    assert_eq!(task.extra["credits_consumed"], 30.0);
}

#[test]
fn malformed_known_event_data_is_an_error() {
    let body =
        br#"{"id":"b:balance.low","type":"balance.low","created_at":"","data":{"balance":"lots"}}"#;
    let err = serde_json::from_slice::<tripo_api::webhook::WebhookEvent>(body).unwrap_err();
    assert!(err.is_data(), "{err:?}");
}

#[test]
fn verify_and_parse_rejects_bad_signature_before_parsing() {
    let err =
        verify_and_parse(SECRET, &header(SIG_COMPLETED), b"not json", None, at(T)).unwrap_err();
    assert!(matches!(err, WebhookError::SignatureMismatch), "{err:?}");
}

#[test]
fn webhook_error_converts_into_crate_error() {
    let err: tripo_api::Error = WebhookError::SignatureMismatch.into();
    assert!(matches!(err, tripo_api::Error::Webhook(_)));
}
