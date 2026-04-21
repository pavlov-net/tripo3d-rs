//! Locks down the set of registered tools and their metadata.

mod common;
use common::start_server_with;

#[tokio::test]
async fn tool_list_snapshot() {
    let client = start_server_with("http://127.0.0.1:1/").await;
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
