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
