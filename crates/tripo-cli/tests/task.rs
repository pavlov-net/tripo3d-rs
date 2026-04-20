use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "current_thread")]
async fn task_get_prints_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": {
                "task_id":"abc","type":"text_to_model","status":"success",
                "progress":100,"create_time":1_700_000_000,
                "output":{"model":"https://cdn/abc.glb"}
            }
        })))
        .mount(&server)
        .await;

    let out = Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "task",
            "get",
            "abc",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["task_id"], "abc");
    assert_eq!(v["status"], "success");
}

#[tokio::test(flavor = "current_thread")]
async fn task_wait_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"task_id":"abc","type":"text_to_model","status":"running","progress":10,"create_time":0}
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"task_id":"abc","type":"text_to_model","status":"success","progress":100,"create_time":0}
        })))
        .mount(&server)
        .await;

    let out = Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "--json",
            "task",
            "wait",
            "abc",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["status"], "success");
}

#[tokio::test(flavor = "current_thread")]
async fn task_wait_non_success_exit_6() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"task_id":"abc","type":"text_to_model","status":"failed","progress":100,"create_time":0,"error_msg":"boom"}
        })))
        .mount(&server)
        .await;
    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "--json",
            "task",
            "wait",
            "abc",
        ])
        .assert()
        .failure()
        .code(6);
}

#[tokio::test(flavor = "current_thread")]
async fn task_download_writes_model() {
    let server = MockServer::start().await;
    let url = format!("{}/files/abc.glb", server.uri());
    Mock::given(method("GET"))
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{
                "task_id":"abc","type":"text_to_model","status":"success","progress":100,"create_time":0,
                "output":{"model": url }
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/files/abc.glb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"glb-bytes" as &[u8]))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let out = Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "task",
            "download",
            "abc",
            "-o",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let target = dir.path().join("abc.glb");
    assert_eq!(std::fs::read(target).unwrap(), b"glb-bytes");
}

#[tokio::test(flavor = "current_thread")]
async fn task_create_raw_posts_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"task_id":"newtask"}
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), r#"{"type":"text_to_model","prompt":"a car"}"#).unwrap();

    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "task",
            "create",
            "--body",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("newtask"));
}
