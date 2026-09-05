use assert_cmd::Command;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "current_thread")]
async fn generation_options_reach_all_endpoints() {
    let server = MockServer::start().await;
    for (command, input_flag, input) in [
        ("text-to-model", "--prompt", "chair"),
        ("image-to-model", "--input", "https://example.com/front.png"),
        (
            "multiview-to-model",
            "--input",
            "https://example.com/front.png",
        ),
    ] {
        Mock::given(method("POST"))
            .and(path(format!("/generation/{command}")))
            .and(body_partial_json(
                serde_json::json!({"model":"P2-20260801","quad":true,"face_limit":25000}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 0, "data": {"task_id": "created"}
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
                command,
                input_flag,
                input,
            ])
            .args([
                "--model",
                "P2-20260801",
                "--quad",
                "true",
                "--face-limit",
                "25000",
            ])
            .assert()
            .success();
    }
}
