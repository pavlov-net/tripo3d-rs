use rmcp::model::CallToolRequestParams;
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
mod common;
use common::{args, start_server};

#[tokio::test]
async fn generation_options_reach_all_endpoints() {
    let server = MockServer::start().await;
    let client = start_server(&server).await;
    for (name, input) in [
        ("text_to_model", json!({"prompt":"chair"})),
        (
            "image_to_model",
            json!({"input":"https://example.com/front.png"}),
        ),
        (
            "multiview_to_model",
            json!({"inputs":["https://example.com/front.png"]}),
        ),
    ] {
        let mut body = json!({"model":"P2-20260801","quad":true,"face_limit":25000});
        body.as_object_mut()
            .unwrap()
            .extend(input.as_object().unwrap().clone());
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
