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
