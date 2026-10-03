use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "current_thread")]
async fn task_get_prints_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": {
                "task_id":"abc","type":"text_to_model","status":"success",
                "progress":100,"created_at":"2026-01-01T00:00:00Z",
                "output":{"model_url":"https://cdn/abc.glb"}
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
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"task_id":"abc","type":"text_to_model","status":"running","progress":10,"created_at":"2026-01-01T00:00:00Z"}
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"task_id":"abc","type":"text_to_model","status":"success","progress":100,"created_at":"2026-01-01T00:00:00Z"}
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
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"task_id":"abc","type":"text_to_model","status":"failed","progress":100,"created_at":"2026-01-01T00:00:00Z",
                     "error_code":2018,"error_message":"model too complex"}
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
        .code(6)
        .stdout(predicate::str::contains(r#""error_code": 2018"#))
        .stderr(predicate::str::contains(
            "ended with status Failed (error 2018: model too complex)",
        ));
}

#[tokio::test(flavor = "current_thread")]
async fn task_download_writes_model() {
    let server = MockServer::start().await;
    let url = format!("{}/files/abc.glb", server.uri());
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{
                "task_id":"abc","type":"text_to_model","status":"success","progress":100,"created_at":"2026-01-01T00:00:00Z",
                "output":{"model_url": url }
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
async fn task_download_refuses_existing_file_with_force_hint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{
                "task_id":"abc","type":"text_to_model","status":"success","progress":100,"created_at":"2026-01-01T00:00:00Z",
                "output":{"model_url": format!("{}/files/abc.glb", server.uri())}
            }
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("abc.glb"), b"keep").unwrap();
    Command::cargo_bin("tripo")
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
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("file already exists"))
        .stderr(predicate::str::contains("hint: pass --force to overwrite"));
    assert_eq!(std::fs::read(dir.path().join("abc.glb")).unwrap(), b"keep");
}

#[tokio::test(flavor = "current_thread")]
async fn task_create_raw_posts_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/generation/text-to-model"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"task_id":"newtask"}
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), r#"{"prompt":"a car"}"#).unwrap();

    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "task",
            "create",
            "--endpoint",
            "generation/text-to-model",
            "--body",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("newtask"));
}

#[tokio::test(flavor = "current_thread")]
async fn task_list_prints_tasks_and_missed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/tasks/list"))
        .and(wiremock::matchers::body_json(
            serde_json::json!({"task_ids": ["b", "gone", "a"]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": {
                "tasks": {
                    "a": {"task_id":"a","type":"text_to_model","status":"success","progress":100},
                    "b": {"task_id":"b","type":"text_to_model","status":"running","progress":40}
                },
                "missed": ["gone"]
            }
        })))
        .expect(1)
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
            "list",
            "b",
            "gone",
            "a",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["tasks"][0]["task_id"], "b");
    assert_eq!(v["tasks"][1]["task_id"], "a");
    assert_eq!(v["missed"], serde_json::json!(["gone"]));
}

#[tokio::test(flavor = "current_thread")]
async fn task_list_over_limit_is_usage_error() {
    let ids: Vec<String> = (0..=100).map(|i| format!("t{i}")).collect();
    Command::cargo_bin("tripo")
        .unwrap()
        .args(["--api-key", "tsk_test", "--base-url", "http://127.0.0.1:1"])
        .args(["task", "list"])
        .args(&ids)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("1..=100"));
}
