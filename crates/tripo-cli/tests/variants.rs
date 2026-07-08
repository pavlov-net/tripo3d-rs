use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "current_thread")]
async fn text_to_model_submit_only() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/generation/text-to-model"))
        .and(body_partial_json(serde_json::json!({
            "prompt": "a red robot"
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
        .and(path("/generation/image-to-model"))
        .and(body_partial_json(serde_json::json!({
            "input": "https://example.com/x.jpg"
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
            "--input",
            "https://example.com/x.jpg",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn image_to_model_with_local_path_uploads_first() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"file_token":"file_abc123"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/generation/image-to-model"))
        .and(body_partial_json(serde_json::json!({
            "input": "file_abc123"
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
            "--input",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn multiview_sends_inputs_array_with_empty_slot() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/generation/multiview-to-model"))
        .and(body_partial_json(serde_json::json!({
            "inputs":[
                "https://example.com/front.jpg",
                "",
                "https://example.com/back.jpg"
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
            "--input",
            "https://example.com/front.jpg",
            "--input",
            "",
            "--input",
            "https://example.com/back.jpg",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn convert_model_to_fbx() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/convert"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_src","format":"FBX","fbx_preset":"mixamo"
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
            "--input",
            "task_src",
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
        .and(path("/models/stylize"))
        .and(body_partial_json(serde_json::json!({
            "original_model_task_id":"task_src","style":"voxel","block_size":64
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
            "--input",
            "task_src",
            "--style",
            "voxel",
            "--block-size",
            "64",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn texture_model_nests_prompt() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/texture"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_src",
            "texture_prompt":{"text":"brass","style_image":"https://cdn/s.jpg"}
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"tx"}})),
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
            "texture-model",
            "--input",
            "task_src",
            "--text-prompt",
            "brass",
            "--style-image",
            "https://cdn/s.jpg",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn refine_and_check_riggable_wire_names_match() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/refine"))
        .and(body_partial_json(serde_json::json!({
            "draft_model_task_id":"task_d"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"r"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/animations/rig-check"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_m"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"cr"}})),
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
            "refine-model",
            "--input",
            "task_d",
        ])
        .assert()
        .success();
    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "check-riggable",
            "--input",
            "task_m",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn rig_model_posts_animations_rig() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/animations/rig"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_m","rig_type":"biped","spec":"mixamo"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"rm"}})),
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
            "rig-model",
            "--input",
            "task_m",
            "--rig-type",
            "biped",
            "--spec",
            "mixamo",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn retarget_single_vs_many() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/animations/retarget"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_m","animation":"preset:walk"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"r1"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/animations/retarget"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_m",
            "animations":["preset:walk","preset:run"]
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"r2"}})),
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
            "retarget-animation",
            "--input",
            "task_m",
            "--animation",
            "preset:walk",
        ])
        .assert()
        .success();
    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "retarget-animation",
            "--input",
            "task_m",
            "--animation",
            "preset:walk,preset:run",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn mesh_seg_and_completion() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/mesh/segment"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_m"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"ms"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/mesh/complete"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_m","part_names":["head","arm"]
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"mc"}})),
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
            "mesh-segmentation",
            "--input",
            "task_m",
        ])
        .assert()
        .success();
    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "mesh-completion",
            "--input",
            "task_m",
            "--part-names",
            "head,arm",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn mesh_decimate_posts_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/mesh/decimate"))
        .and(body_partial_json(serde_json::json!({
            "input":"task_m","face_limit":2000
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"sl"}})),
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
            "mesh-decimate",
            "--input",
            "task_m",
            "--face-limit",
            "2000",
        ])
        .assert()
        .success();
}

#[tokio::test(flavor = "current_thread")]
async fn text_to_model_wait_output_end_to_end() {
    let server = MockServer::start().await;
    let model_url = format!("{}/files/abc.glb", server.uri());

    Mock::given(method("POST"))
        .and(path("/generation/text-to-model"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"code":0,"data":{"task_id":"abc"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
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
            "code":0,"data":{"task_id":"abc","type":"text_to_model","status":"success","progress":100,"created_at":"2026-01-01T00:00:00Z",
                              "output":{"model_url": model_url }}
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/files/abc.glb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"glb" as &[u8]))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("tripo")
        .unwrap()
        .args([
            "--api-key",
            "tsk_test",
            "--base-url",
            &server.uri(),
            "--json",
            "text-to-model",
            "--prompt",
            "x",
            "--output",
            dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(std::fs::read(dir.path().join("abc.glb")).unwrap(), b"glb");
}
