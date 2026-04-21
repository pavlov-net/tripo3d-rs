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

/// Coerce a JSON value into the `JsonObject` that `CallToolRequestParams`
/// expects as its `arguments` field.
pub fn args(v: serde_json::Value) -> rmcp::model::JsonObject {
    match v {
        serde_json::Value::Object(m) => m,
        other => panic!("arguments must be a JSON object, got {other}"),
    }
}
