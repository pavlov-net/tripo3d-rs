//! In-process MCP client ↔ server harness. Each test wires a `TripoServer` over
//! a pair of `tokio::io::duplex` streams, exercising the full rmcp stack
//! (initialize, tool listing, tool calls) against a `wiremock` Tripo API.

use rmcp::{
    ClientHandler, ServiceExt,
    model::{CallToolRequestParams, ClientInfo},
};
use serde_json::json;
use wiremock::MockServer;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[derive(Clone, Default)]
struct NoopClient;
impl ClientHandler for NoopClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}

/// Spin up a `TripoServer` pointed at `mock` and return a connected MCP client.
async fn start_server(
    mock: &MockServer,
) -> rmcp::service::RunningService<rmcp::RoleClient, NoopClient> {
    let (server_io, client_io) = tokio::io::duplex(8192);
    let tripo_client = tripo_api::Client::builder()
        .api_key("tsk_test")
        .base_url(mock.uri().parse().unwrap())
        .build()
        .unwrap();
    let server = tripo_mcp::server::TripoServer::new(tripo_client);

    tokio::spawn(async move {
        if let Ok(svc) = server.serve(server_io).await {
            let _ = svc.waiting().await;
        }
    });

    NoopClient.serve(client_io).await.unwrap()
}

fn args(v: serde_json::Value) -> rmcp::model::JsonObject {
    match v {
        serde_json::Value::Object(m) => m,
        other => panic!("arguments must be a JSON object, got {other}"),
    }
}

#[tokio::test]
async fn calls_get_balance() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/balance"))
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
async fn calls_text_to_model() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .and(wiremock::matchers::body_partial_json(
            json!({"type":"text_to_model","prompt":"a red robot"}),
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
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{
                "task_id":"abc","type":"text_to_model","status":"success","progress":100,"create_time":0,
                "output":{"model": model_url}
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
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{"task_id":"abc","type":"text_to_model","status":"running","progress":50,"create_time":0}
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{"task_id":"abc","type":"text_to_model","status":"success","progress":100,"create_time":0}
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
async fn calls_create_raw_task() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"task_id":"raw"}
        })))
        .mount(&server)
        .await;

    let client = start_server(&server).await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("create_raw_task").with_arguments(args(json!({
                "body": {"type":"text_to_model","prompt":"x"}
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
        .and(path("/upload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,"data":{"image_token":"550e8400-e29b-41d4-a716-446655440000"}
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
    assert!(format!("{result:?}").contains("550e8400"));
}

#[tokio::test]
async fn calls_get_task() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code":0,
            "data":{"task_id":"abc","type":"text_to_model","status":"success","progress":100,"create_time":0}
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
