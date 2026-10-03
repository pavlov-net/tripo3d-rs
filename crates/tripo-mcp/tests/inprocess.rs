//! In-process MCP client ↔ server tests. Each test wires a `TripoServer` over
//! a pair of `tokio::io::duplex` streams against a `wiremock` Tripo API.

use rmcp::model::CallToolRequestParams;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;
use common::{args, start_server, start_server_with};

#[tokio::test]
async fn calls_get_balance() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/account/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"balance":10.0,"frozen":0.0}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(CallToolRequestParams::new("get_balance"))
        .await
        .unwrap();
    let text = format!("{result:?}");
    assert!(text.contains("10"), "missing balance in {text}");
}

#[tokio::test]
async fn calls_mesh_seg_completion_decimate() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"task_id":"z"}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    for (name, argv) in [
        ("mesh_segmentation", json!({"input":"task_m"})),
        (
            "mesh_completion",
            json!({"input":"task_m","part_names":["a"]}),
        ),
        ("mesh_decimate", json!({"input":"task_m","face_limit":2000})),
    ] {
        let r = client
            .call_tool(CallToolRequestParams::new(name).with_arguments(args(argv)))
            .await;
        assert!(r.is_ok(), "{name} failed: {r:?}");
    }
}

#[tokio::test]
async fn calls_texture_refine_rigcheck_rig_retarget() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"task_id":"y"}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;

    for (name, argv) in [
        ("texture_model", json!({"input":"task_m"})),
        ("refine_model", json!({"draft_model_task_id":"d"})),
        ("check_riggable", json!({"input":"task_m"})),
        ("rig_model", json!({"input":"task_m"})),
        (
            "retarget_animation",
            json!({"input":"task_m","animation":"preset:walk"}),
        ),
    ] {
        let r = client
            .call_tool(CallToolRequestParams::new(name).with_arguments(args(argv)))
            .await;
        assert!(r.is_ok(), "{name} failed: {r:?}");
    }
}

#[tokio::test]
async fn calls_image_multiview_convert_stylize() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"task_id":"x"}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;

    for (name, argv) in [
        ("image_to_model", json!({"input":"https://e/x.jpg"})),
        ("multiview_to_model", json!({"inputs":["https://e/a.jpg"]})),
        ("convert_model", json!({"input":"task_m","format":"GLTF"})),
        (
            "stylize_model",
            json!({"original_model_task_id":"task_m","style":"voxel"}),
        ),
    ] {
        let r = client
            .call_tool(CallToolRequestParams::new(name).with_arguments(args(argv)))
            .await;
        assert!(r.is_ok(), "{name} failed: {r:?}");
    }
}

#[tokio::test]
async fn calls_import_splat_smart_segment() {
    let server = MockServer::start().await;
    for (endpoint, body) in [
        ("/models/import", json!({"input":"https://e/m.glb"})),
        (
            "/generation/image-to-splat",
            json!({"input":"https://e/x.png","model_seed":3}),
        ),
        (
            "/mesh/smartsegment",
            json!({"seg_type":"image","input":"file_x","granularity":"coarse"}),
        ),
    ] {
        Mock::given(method("POST"))
            .and(path(endpoint))
            .and(wiremock::matchers::body_json(body))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code":0,"data":{"task_id":"w"}
            })))
            .expect(1)
            .mount(&server)
            .await;
    }

    let client = start_server(&server).await;
    for (name, argv) in [
        ("import_model", json!({"input":"https://e/m.glb"})),
        (
            "image_to_splat",
            json!({"input":"https://e/x.png","model_seed":3}),
        ),
        (
            "mesh_smart_segment",
            json!({"seg_type":"image","input":"file_x","granularity":"coarse"}),
        ),
    ] {
        let r = client
            .call_tool(CallToolRequestParams::new(name).with_arguments(args(argv)))
            .await;
        assert!(r.is_ok(), "{name} failed: {r:?}");
    }
}

#[tokio::test]
async fn smart_segment_model_without_transform_errors() {
    let server = MockServer::start().await;
    let client = start_server(&server).await;
    let r = client
        .call_tool(
            CallToolRequestParams::new("mesh_smart_segment")
                .with_arguments(args(json!({"seg_type":"model","input":"https://e/m.glb"}))),
        )
        .await;
    assert!(
        format!("{r:?}").contains("transform"),
        "expected transform error, got {r:?}"
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn calls_text_to_model() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/generation/text-to-model"))
        .and(wiremock::matchers::body_partial_json(
            json!({"prompt":"a red robot"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"task_id":"t1"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("text_to_model")
                .with_arguments(args(json!({"prompt":"a red robot"}))),
        )
        .await
        .unwrap();
    assert!(format!("{result:?}").contains("t1"));
}

#[tokio::test]
async fn calls_download_task_models() {
    let server = MockServer::start().await;
    let model_url = format!("{}/files/abc.glb", server.uri());
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{
                "task_id":"abc","type":"text_to_model","status":"success","progress":100,"created_at":"2026-01-01T00:00:00Z",
                "output":{"model_url": model_url}
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/files/abc.glb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"glb" as &[u8]))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("download_task_models").with_arguments(args(json!({
                "task_id":"abc",
                "output_dir": dir.path(),
                "overwrite": false,
            }))),
        )
        .await
        .unwrap();
    assert!(format!("{result:?}").contains("abc.glb"));
    assert_eq!(std::fs::read(dir.path().join("abc.glb")).unwrap(), b"glb");
}

#[tokio::test]
async fn calls_wait_for_task() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{"task_id":"abc","type":"text_to_model","status":"running","progress":50,"created_at":"2026-01-01T00:00:00Z"}
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{"task_id":"abc","type":"text_to_model","status":"success","progress":100,"created_at":"2026-01-01T00:00:00Z"}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("wait_for_task").with_arguments(args(json!({
                "task_id":"abc","max_interval_seconds":1
            }))),
        )
        .await
        .unwrap();
    assert!(format!("{result:?}").contains("success"));
}

#[tokio::test]
async fn wait_for_task_reports_failure_detail() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{"task_id":"abc","type":"text_to_model","status":"failed","progress":0,"created_at":"2026-01-01T00:00:00Z",
                    "error_code":2018,"error_message":"model too complex"}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("wait_for_task")
                .with_arguments(args(json!({"task_id":"abc"}))),
        )
        .await
        .unwrap();
    let task = result.structured_content.unwrap();
    assert_eq!(task["error_code"], 2018);
    assert_eq!(task["error_message"], "model too complex");
}

#[tokio::test]
async fn calls_create_raw_task() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/generation/text-to-model"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"task_id":"raw"}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("create_raw_task").with_arguments(args(json!({
                "endpoint": "generation/text-to-model",
                "body": {"prompt":"x"}
            }))),
        )
        .await
        .unwrap();
    assert!(format!("{result:?}").contains("raw"));
}

#[tokio::test]
async fn calls_upload_file() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"file_token":"file_abc123"}
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"jpeg").unwrap();

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("upload_file")
                .with_arguments(args(json!({ "path": tmp.path() }))),
        )
        .await
        .unwrap();
    assert!(format!("{result:?}").contains("file_abc123"));
}

#[tokio::test]
async fn calls_upload_file_with_presign() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files/presign"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{
                "presigned_url": format!("{}/storage/obj", server.uri()),
                "file_token":"file_presigned",
                "expires_in":1800
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

    let tmp = tempfile::Builder::new().suffix(".glb").tempfile().unwrap();
    std::fs::write(tmp.path(), b"glb").unwrap();

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("upload_file")
                .with_arguments(args(json!({ "path": tmp.path(), "presign": true }))),
        )
        .await
        .unwrap();
    assert!(format!("{result:?}").contains("file_presigned"));
}

#[tokio::test]
async fn calls_get_task() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tasks/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{"task_id":"abc","type":"text_to_model","status":"success","progress":100,"created_at":"2026-01-01T00:00:00Z"}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("get_task").with_arguments(args(json!({"task_id":"abc"}))),
        )
        .await
        .unwrap();
    let txt = format!("{result:?}");
    assert!(txt.contains("abc"));
    assert!(txt.contains("success"));
}

#[tokio::test]
async fn calls_list_tasks() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/tasks/list"))
        .and(wiremock::matchers::body_json(json!({"task_ids":["abc","gone"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{
                "tasks":{"abc":{"task_id":"abc","type":"text_to_model","status":"success","progress":100}},
                "missed":["gone"]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("list_tasks")
                .with_arguments(args(json!({"task_ids":["abc","gone"]}))),
        )
        .await
        .unwrap();
    let out = result.structured_content.expect("structured output");
    assert_eq!(out["tasks"][0]["task_id"], "abc");
    assert_eq!(out["missed"], json!(["gone"]));
}

#[tokio::test]
async fn list_tasks_over_limit_is_tool_error() {
    let client = start_server_with("http://127.0.0.1:1/").await;
    let ids: Vec<String> = (0..=100).map(|i| format!("t{i}")).collect();
    let err = client
        .call_tool(
            CallToolRequestParams::new("list_tasks").with_arguments(args(json!({"task_ids":ids}))),
        )
        .await
        .unwrap_err();
    assert!(format!("{err:?}").contains("1..=100"), "{err:?}");
}

#[tokio::test]
async fn calls_get_usage() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/account/usage"))
        .and(wiremock::matchers::query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":[{"task_id":"abc","type":"text_to_model","credits_consumed":100.0,"created_at":"2026-01-01T00:00:00Z"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(CallToolRequestParams::new("get_usage").with_arguments(args(json!({"limit":5}))))
        .await
        .unwrap();
    let out = result.structured_content.expect("structured output");
    assert_eq!(out["records"][0]["task_id"], "abc");
    assert_eq!(out["records"][0]["credits_consumed"], 100.0);
}

#[tokio::test]
async fn calls_image_generation_tools() {
    let server = MockServer::start().await;
    let client = start_server(&server).await;
    for (name, argv) in [
        (
            "text_to_image",
            json!({"prompt":"icon","model":"chat_image_2.5_sunburst","quality":"max","background":"transparent"}),
        ),
        (
            "image_to_image",
            json!({"inputs":["https://e/a.png","file_b"],"prompt":"merge image[1] and image[2]"}),
        ),
        ("image_to_multiview", json!({"input":"https://e/a.png"})),
        (
            "edit_multiview",
            json!({"input":"task_mv","prompts":[{"prompt":"red shirt","view":"front"}]}),
        ),
    ] {
        Mock::given(method("POST"))
            .and(path(format!("/generation/{}", name.replace('_', "-"))))
            .and(wiremock::matchers::body_json(argv.clone()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code":0,"data":{"task_id":"img"}
            })))
            .expect(1)
            .mount(&server)
            .await;
        let r = client
            .call_tool(CallToolRequestParams::new(name).with_arguments(args(argv)))
            .await
            .unwrap();
        assert_ne!(r.is_error, Some(true), "{name}: {r:?}");
        assert!(format!("{r:?}").contains("img"), "{name}: {r:?}");
    }
}

#[tokio::test]
async fn image_tool_validation_errors_before_submit() {
    let server = MockServer::start().await;
    let client = start_server(&server).await;
    let r = client
        .call_tool(
            CallToolRequestParams::new("text_to_image").with_arguments(args(
                json!({"prompt":"x","model":"seedream_v4","quality":"high"}),
            )),
        )
        .await;
    assert!(
        format!("{r:?}").contains("does not accept quality"),
        "{r:?}"
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}
