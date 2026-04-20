use assert_cmd::Command;
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
