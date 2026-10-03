use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "current_thread")]
async fn upload_prints_file_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0, "data": { "file_token": "file_abc123" }
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"jpeg").unwrap();

    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "upload",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("file_abc123"));
}

#[tokio::test(flavor = "current_thread")]
async fn upload_presign_puts_to_storage() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files/presign"))
        .and(body_json(serde_json::json!({ "format": "png" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": {
                "presigned_url": format!("{}/storage/obj", server.uri()),
                "file_token": "file_presigned",
                "expires_in": 1800
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/obj"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    std::fs::write(tmp.path(), b"png").unwrap();

    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "upload",
            "--presign",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("file_presigned"));
}
