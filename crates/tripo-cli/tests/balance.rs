use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_balance(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/user/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0, "data": { "balance": 42.5, "frozen": 1.5 }
        })))
        .mount(server)
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn balance_text_output() {
    let server = MockServer::start().await;
    mock_balance(&server).await;
    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "balance",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("balance: 42.50"))
        .stdout(predicate::str::contains("frozen:  1.50"));
}

#[tokio::test(flavor = "current_thread")]
async fn balance_json_output() {
    let server = MockServer::start().await;
    mock_balance(&server).await;
    let out = Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "--json",
            "balance",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["balance"], 42.5);
    assert_eq!(v["frozen"], 1.5);
}

#[tokio::test(flavor = "current_thread")]
async fn missing_api_key_is_usage_error() {
    Command::cargo_bin("tripo")
        .unwrap()
        .env_remove("TRIPO_API_KEY")
        .args(["balance"])
        .assert()
        .failure()
        .code(2);
}
