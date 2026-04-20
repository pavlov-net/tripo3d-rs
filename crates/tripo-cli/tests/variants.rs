use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "current_thread")]
async fn text_to_model_submit_only() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .and(body_partial_json(serde_json::json!({
            "type": "text_to_model", "prompt": "a red robot"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data": {"task_id":"new-id"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "text-to-model",
            "--prompt",
            "a red robot",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("new-id"));
}
