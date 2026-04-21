//! Locks down the set of registered tools and their metadata.

use rmcp::{ClientHandler, ServiceExt, model::ClientInfo};

#[derive(Clone, Default)]
struct NoopClient;
impl ClientHandler for NoopClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}

async fn start_server() -> rmcp::service::RunningService<rmcp::RoleClient, NoopClient> {
    let (server_io, client_io) = tokio::io::duplex(8192);
    let tripo_client = tripo_api::Client::builder()
        .api_key("tsk_test")
        .base_url("http://127.0.0.1:1/".parse().unwrap())
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
async fn tool_list_snapshot() {
    let client = start_server().await;
    let list = client.list_all_tools().await.unwrap();
    let mut names: Vec<String> = list.iter().map(|t| t.name.to_string()).collect();
    names.sort();
    insta::assert_debug_snapshot!("tool_list_names", names);

    let mut critical: Vec<_> = list
        .iter()
        .filter(|t| {
            matches!(
                t.name.as_ref(),
                "get_balance" | "wait_for_task" | "text_to_model"
            )
        })
        .collect();
    critical.sort_by(|a, b| a.name.cmp(&b.name));
    insta::assert_json_snapshot!(
        "tool_list_critical",
        serde_json::to_value(&critical).unwrap()
    );
}
