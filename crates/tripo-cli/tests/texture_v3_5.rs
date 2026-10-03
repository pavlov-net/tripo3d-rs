use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const GENERATION: [(&str, &str, &str); 3] = [
    ("text-to-model", "--prompt", "chair"),
    ("image-to-model", "--input", "https://example.com/front.png"),
    (
        "multiview-to-model",
        "--input",
        "https://example.com/front.png",
    ),
];

fn tripo(server: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("tripo").unwrap();
    cmd.args(["--api-key", "tsk_test", "--base-url", &server.uri()]);
    cmd
}

#[tokio::test(flavor = "current_thread")]
async fn generation_texture_options_reach_all_endpoints() {
    let server = MockServer::start().await;
    for (command, input_flag, input) in GENERATION {
        Mock::given(method("POST"))
            .and(path(format!("/generation/{command}")))
            .and(body_partial_json(serde_json::json!({
                "texture_version": "v3.5-20260815",
                "texture_quality": "fast",
                "delight": false,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 0, "data": {"task_id": "created"}
            })))
            .expect(1)
            .mount(&server)
            .await;
        tripo(&server)
            .args([command, input_flag, input])
            .args([
                "--texture-version",
                "v3.5-20260815",
                "--texture-quality",
                "fast",
                "--delight",
                "false",
            ])
            .assert()
            .success();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn generation_fast_without_texture_version_is_rejected_locally() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    for (command, input_flag, input) in GENERATION {
        tripo(&server)
            .args([command, input_flag, input, "--texture-quality", "fast"])
            .assert()
            .code(2)
            .stderr(predicate::str::contains("texture_version"));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn texture_model_v3_5_with_four_images() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/texture"))
        .and(body_partial_json(serde_json::json!({
            "input": "task_src",
            "model": "v3.5-20260815",
            "texture_quality": "fast",
            "delight": false,
            "texture_prompt": {"images": [
                "https://cdn/f.jpg", "https://cdn/l.jpg", "https://cdn/b.jpg", "https://cdn/r.jpg"
            ]},
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code": 0, "data": {"task_id": "tx"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    tripo(&server)
        .args(["texture-model", "--input", "task_src"])
        .args(["--model", "v3.5-20260815", "--texture-quality", "fast"])
        .args(["--delight", "false"])
        .args([
            "--images-prompt",
            "https://cdn/f.jpg",
            "--images-prompt",
            "https://cdn/l.jpg",
        ])
        .args([
            "--images-prompt",
            "https://cdn/b.jpg",
            "--images-prompt",
            "https://cdn/r.jpg",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn texture_model_invalid_requests_are_rejected_locally() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    for (extra, expected) in [
        (&["--texture-quality", "fast"][..], "v3.5-20260815"),
        (
            &[
                "--images-prompt",
                "https://cdn/f.jpg",
                "--images-prompt",
                "https://cdn/l.jpg",
            ][..],
            "exactly 4",
        ),
        (
            &[
                "--text-prompt",
                "brass",
                "--image-prompt",
                "https://cdn/i.jpg",
            ][..],
            "mutually exclusive",
        ),
    ] {
        tripo(&server)
            .args(["texture-model", "--input", "task_src"])
            .args(extra)
            .assert()
            .code(2)
            .stderr(predicate::str::contains(expected));
    }
}
