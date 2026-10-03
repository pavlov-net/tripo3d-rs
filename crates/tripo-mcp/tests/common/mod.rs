//! Shared test harness: spins up a `TripoServer` over an in-process
//! `tokio::io::duplex` and returns a connected MCP client.
//!
//! Integration test files compile independently, so items used by only one
//! file get flagged as dead code in the other — suppress at module level.

#![allow(dead_code)]

use rmcp::{RoleClient, ServiceExt, service::RunningService};
use wiremock::MockServer;

/// Build a `TripoServer` against `base_url` and return a connected MCP client.
pub async fn start_server_with(base_url: &str) -> RunningService<RoleClient, ()> {
    let (server_io, client_io) = tokio::io::duplex(8192);
    let tripo_client = tripo_api::Client::builder()
        .api_key("tsk_test")
        .base_url(base_url.parse().unwrap())
        .build()
        .unwrap();
    let server = tripo_mcp::server::TripoServer::new(tripo_client);

    tokio::spawn(async move {
        if let Ok(svc) = server.serve(server_io).await {
            let _ = svc.waiting().await;
        }
    });

    ().serve(client_io).await.unwrap()
}

/// Spin up a `TripoServer` pointed at `mock`.
pub async fn start_server(mock: &MockServer) -> RunningService<RoleClient, ()> {
    start_server_with(&mock.uri()).await
}

/// Text content of a tool-error result. Panics unless `is_error` is set.
pub fn tool_error_text(result: &rmcp::model::CallToolResult) -> String {
    assert_eq!(
        result.is_error,
        Some(true),
        "expected a tool error: {result:?}"
    );
    assert!(
        result.structured_content.is_none(),
        "tool error carries structured content: {result:?}"
    );
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Panic if `result` is a tool error.
pub fn assert_tool_ok(name: &str, result: &rmcp::model::CallToolResult) {
    assert_ne!(result.is_error, Some(true), "{name} failed: {result:?}");
}

/// Coerce a JSON value into the `JsonObject` that `CallToolRequestParams`
/// expects as its `arguments` field.
pub fn args(v: serde_json::Value) -> rmcp::model::JsonObject {
    match v {
        serde_json::Value::Object(m) => m,
        other => panic!("arguments must be a JSON object, got {other}"),
    }
}

/// One `(tool name, arguments)` pair per generation tool: `options` merged
/// with that tool's minimal input.
pub fn generation_bodies(options: &serde_json::Value) -> Vec<(&'static str, serde_json::Value)> {
    use serde_json::json;
    [
        ("text_to_model", json!({"prompt": "chair"})),
        (
            "image_to_model",
            json!({"input": "https://example.com/front.png"}),
        ),
        (
            "multiview_to_model",
            json!({"inputs": ["https://example.com/front.png"]}),
        ),
    ]
    .into_iter()
    .map(|(name, input)| {
        let mut body = options.clone();
        body.as_object_mut()
            .unwrap()
            .extend(input.as_object().unwrap().clone());
        (name, body)
    })
    .collect()
}
