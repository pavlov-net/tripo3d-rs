use rmcp::model::CallToolRequestParams;
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
mod common;
use common::{args, start_server};

const GENERATION: [(&str, &str, &str); 3] = [
    ("text_to_model", "prompt", "chair"),
    ("image_to_model", "input", "https://example.com/front.png"),
    (
        "multiview_to_model",
        "inputs",
        "https://example.com/front.png",
    ),
];

fn generation_body(input_key: &str, input: &str, options: &serde_json::Value) -> serde_json::Value {
    let mut body = options.clone();
    body[input_key] = if input_key == "inputs" {
        json!([input])
    } else {
        json!(input)
    };
    body
}

#[tokio::test]
async fn generation_texture_options_reach_all_endpoints() {
    let server = MockServer::start().await;
    let client = start_server(&server).await;
    let options = json!({
        "texture_version": "v3.5-20260815",
        "texture_quality": "fast",
        "delight": false,
    });
    for (name, input_key, input) in GENERATION {
        let body = generation_body(input_key, input, &options);
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
async fn generation_fast_without_texture_version_is_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let client = start_server(&server).await;
    for (name, input_key, input) in GENERATION {
        let body = generation_body(input_key, input, &json!({"texture_quality": "fast"}));
        let err = client
            .call_tool(CallToolRequestParams::new(name).with_arguments(args(body)))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("texture_version"), "{err}");
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
async fn texture_model_invalid_requests_are_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let client = start_server(&server).await;
    for (body, expected) in [
        (
            json!({"input": "task_src", "texture_quality": "fast"}),
            "v3.5-20260815",
        ),
        (
            json!({"input": "task_src", "texture_prompt": {"images": ["https://cdn/f.jpg"]}}),
            "exactly 4",
        ),
        (
            json!({"input": "task_src", "texture_prompt": {"text": "brass", "image": "https://cdn/i.jpg"}}),
            "mutually exclusive",
        ),
    ] {
        let err = client
            .call_tool(CallToolRequestParams::new("texture_model").with_arguments(args(body)))
            .await
            .unwrap_err();
        assert!(err.to_string().contains(expected), "{err}");
    }
}
