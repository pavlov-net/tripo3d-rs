use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_usage(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/account/usage"))
        .and(query_param("limit", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": [
                {"task_id": "task_a", "type": "text_to_model", "credits_consumed": 100.0,
                 "created_at": "2026-01-01T00:00:00Z"},
                {"task_id": "task_b", "type": "rig_model", "status": "success",
                 "credits_consumed": 25.5, "create_time": 1_767_225_600}
            ]
        })))
        .expect(1)
        .mount(server)
        .await;
}

fn tripo(server: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("tripo").unwrap();
    cmd.args(["--api-key", "tsk_test", "--base-url", &server.uri()]);
    cmd
}

#[tokio::test(flavor = "current_thread")]
async fn usage_text_output() {
    let server = MockServer::start().await;
    mock_usage(&server).await;
    tripo(&server)
        .args(["usage", "--limit", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "2026-01-01T00:00:00Z  task_a  text_to_model  -  100.00",
        ))
        .stdout(predicate::str::contains(
            "1767225600  task_b  rig_model  success  25.50",
        ));
}

#[tokio::test(flavor = "current_thread")]
async fn usage_json_output() {
    let server = MockServer::start().await;
    mock_usage(&server).await;
    let out = tripo(&server)
        .args(["--json", "usage", "--limit", "2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v[0]["task_id"], "task_a");
    assert_eq!(v[1]["created_at"], 1_767_225_600);
    assert_eq!(v[1]["status"], "success");
}
