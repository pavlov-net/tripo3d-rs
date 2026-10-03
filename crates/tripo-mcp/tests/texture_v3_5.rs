use rmcp::model::CallToolRequestParams;
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
mod common;
use common::{args, generation_bodies, start_server, tool_error_text};

#[tokio::test]
async fn generation_texture_options_reach_all_endpoints() {
    let server = MockServer::start().await;
    let client = start_server(&server).await;
    let options = json!({
        "texture_version": "v3.5-20260815",
        "texture_quality": "fast",
        "delight": false,
    });
    for (name, body) in generation_bodies(&options) {
        Mock::given(method("POST"))
            .and(path(format!("/generation/{}", name.replace('_', "-"))))
            .and(body_partial_json(body.clone()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0, "data": {"task_id": "created"}
            })))
            .expect(1)
            .mount(&server)
            .await;
        let result = client
            .call_tool(CallToolRequestParams::new(name).with_arguments(args(body)))
            .await
            .unwrap();
        assert_ne!(result.is_error, Some(true), "{result:?}");
    }
}

#[tokio::test]
async fn texture_model_v3_5_with_four_images() {
    let server = MockServer::start().await;
    let body = json!({
        "input": "task_src",
        "model": "v3.5-20260815",
        "texture_quality": "fast",
        "delight": true,
        "texture_prompt": {"images": [
            "https://cdn/f.jpg", "https://cdn/l.jpg", "https://cdn/b.jpg", "https://cdn/r.jpg"
        ]},
    });
    Mock::given(method("POST"))
        .and(path("/models/texture"))
        .and(body_partial_json(body.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0, "data": {"task_id": "tex"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let client = start_server(&server).await;
    let result = client
        .call_tool(CallToolRequestParams::new("texture_model").with_arguments(args(body)))
        .await
        .unwrap();
    assert_ne!(result.is_error, Some(true), "{result:?}");
}

#[tokio::test]
async fn texture_model_fast_without_v3_5_is_rejected_before_http() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let client = start_server(&server).await;
    let body = json!({"input": "task_src", "texture_quality": "fast"});
    let result = client
        .call_tool(CallToolRequestParams::new("texture_model").with_arguments(args(body)))
        .await
        .unwrap();
    let text = tool_error_text(&result);
    assert!(text.contains("v3.5-20260815"), "{text}");
}
