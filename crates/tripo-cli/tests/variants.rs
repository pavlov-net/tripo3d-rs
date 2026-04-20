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

#[tokio::test(flavor = "current_thread")]
async fn image_to_model_with_url() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .and(body_partial_json(serde_json::json!({
            "type":"image_to_model",
            "file":{"type":"jpg","url":"https://example.com/x.jpg"}
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"i2m"}})),
        )
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
            "image-to-model",
            "--image",
            "https://example.com/x.jpg",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn image_to_model_with_local_path_uploads_first() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/upload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"image_token":"550e8400-e29b-41d4-a716-446655440000"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .and(body_partial_json(serde_json::json!({
            "type":"image_to_model",
            "file":{"type":"jpg","file_token":"550e8400-e29b-41d4-a716-446655440000"}
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"i2m"}})),
        )
        .expect(1)
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
            "image-to-model",
            "--image",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn multiview_sends_files_array_with_empty_slot() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .and(body_partial_json(serde_json::json!({
            "type":"multiview_to_model",
            "files":[
                {"type":"jpg","url":"https://example.com/front.jpg"},
                {},
                {"type":"jpg","url":"https://example.com/back.jpg"}
            ]
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"mv"}})),
        )
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
            "multiview-to-model",
            "--image",
            "https://example.com/front.jpg",
            "--image",
            "",
            "--image",
            "https://example.com/back.jpg",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn convert_model_to_fbx() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .and(body_partial_json(serde_json::json!({
            "type":"convert_model","original_model_task_id":"src","format":"FBX","fbx_preset":"mixamo"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"cv"}})),
        )
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
            "convert-model",
            "--original-model-task-id",
            "src",
            "--format",
            "FBX",
            "--fbx-preset",
            "mixamo",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn stylize_voxel() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .and(body_partial_json(serde_json::json!({
            "type":"stylize_model","original_model_task_id":"src","style":"voxel","block_size":64
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"sv"}})),
        )
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
            "stylize-model",
            "--original-model-task-id",
            "src",
            "--style",
            "voxel",
            "--block-size",
            "64",
        ])
        .assert()
        .success();
}
