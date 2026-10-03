use serde_json::json;
use tripo_api::{Client, TaskRequest, versions};
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// One request per generation endpoint, built from the same option fields.
fn generation_requests(options: &serde_json::Value) -> Vec<TaskRequest> {
    let with = |extra: serde_json::Value| {
        let mut body = options.clone();
        body.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        body
    };
    vec![
        TaskRequest::TextToModel(serde_json::from_value(with(json!({"prompt": "chair"}))).unwrap()),
        TaskRequest::ImageToModel(
            serde_json::from_value(with(json!({"input": "https://example.com/front.png"})))
                .unwrap(),
        ),
        TaskRequest::MultiviewToModel(
            serde_json::from_value(with(json!({"inputs": ["https://example.com/front.png"]})))
                .unwrap(),
        ),
    ]
}

fn texture_request(mut body: serde_json::Value) -> TaskRequest {
    body["input"] = json!("task_src");
    TaskRequest::TextureModel(serde_json::from_value(body).unwrap())
}

fn invalid_message(req: &TaskRequest) -> String {
    match req.validate().unwrap_err() {
        tripo_api::Error::InvalidRequest(msg) => msg,
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
}

#[test]
fn generation_fast_requires_texture_version_v3_5() {
    for texture_version in [
        None,
        Some(versions::texture_version::V3_0),
        Some(versions::texture_version::V2_5),
    ] {
        let options = json!({"texture_quality": "fast", "texture_version": texture_version});
        for req in generation_requests(&options) {
            let msg = invalid_message(&req);
            assert!(
                msg.contains("texture_version") && msg.contains("v3.5-20260815"),
                "{msg}"
            );
        }
    }
}

#[test]
fn generation_fast_with_v3_5_serializes_texture_fields() {
    let options = json!({
        "model": versions::text_image::P2,
        "texture_quality": "fast",
        "texture_version": versions::texture_version::V3_5,
        "delight": false,
    });
    for req in generation_requests(&options) {
        req.validate().unwrap();
        let body = serde_json::to_value(&req).unwrap();
        assert_eq!(body["texture_quality"], "fast");
        assert_eq!(body["texture_version"], "v3.5-20260815");
        assert_eq!(body["delight"], false);
    }
}

#[test]
fn generation_texture_version_without_fast_is_unrestricted() {
    for texture_version in [
        versions::texture_version::V2_5,
        versions::texture_version::V3_0,
        "future-texture",
    ] {
        let options = json!({"texture_quality": "extreme", "texture_version": texture_version, "delight": true});
        for req in generation_requests(&options) {
            req.validate().unwrap();
        }
    }
    for req in generation_requests(&json!({})) {
        let body = serde_json::to_value(&req).unwrap();
        assert!(body.get("texture_version").is_none());
        assert!(body.get("delight").is_none());
    }
}

#[test]
fn texture_endpoint_fast_requires_model_v3_5() {
    for model in [
        None,
        Some(versions::texture::V3_0),
        Some(versions::texture::V2_5),
    ] {
        let req = texture_request(json!({"texture_quality": "fast", "model": model}));
        let msg = invalid_message(&req);
        assert!(
            msg.contains("model") && msg.contains("v3.5-20260815"),
            "{msg}"
        );
    }
    texture_request(json!({"texture_quality": "fast", "model": versions::texture::V3_5}))
        .validate()
        .unwrap();
}

#[test]
fn texture_prompt_images_requires_exactly_four() {
    let urls = |n: usize| {
        (0..n)
            .map(|i| format!("https://example.com/{i}.png"))
            .collect::<Vec<_>>()
    };
    for n in [0, 1, 3, 5] {
        let req = texture_request(json!({"texture_prompt": {"images": urls(n)}}));
        assert!(invalid_message(&req).contains("exactly 4"));
    }
    texture_request(json!({"texture_prompt": {"images": urls(4)}}))
        .validate()
        .unwrap();
}

#[test]
fn texture_prompt_modes_are_mutually_exclusive() {
    let images = json!([
        "https://example.com/f.png",
        "https://example.com/l.png",
        "https://example.com/b.png",
        "https://example.com/r.png",
    ]);
    let image = json!("https://example.com/ref.png");
    for prompt in [
        json!({"text": "brass", "image": image}),
        json!({"text": "brass", "images": images}),
        json!({"image": image, "images": images}),
        json!({"text": "brass", "image": image, "images": images}),
    ] {
        let req = texture_request(json!({"texture_prompt": prompt}));
        assert!(invalid_message(&req).contains("mutually exclusive"));
    }
    for prompt in [
        json!({"style_image": image}),
        json!({"image": image, "style_image": image}),
        json!({"images": images, "style_image": image}),
    ] {
        let req = texture_request(json!({"texture_prompt": prompt}));
        assert!(invalid_message(&req).contains("style_image"));
    }
    for prompt in [
        json!({"text": "brass"}),
        json!({"text": "brass", "style_image": image}),
        json!({"image": image}),
        json!({"images": images}),
    ] {
        texture_request(json!({"texture_prompt": prompt}))
            .validate()
            .unwrap();
    }
}

#[tokio::test]
async fn create_task_uploads_local_texture_prompt_images() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0, "data": {"file_token": "file_up"}
        })))
        .expect(4)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/texture"))
        .and(body_partial_json(json!({
            "model": "v3.5-20260815",
            "texture_quality": "fast",
            "delight": true,
            "texture_prompt": {"images": ["file_up", "file_up", "file_up", "file_up"]},
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0, "data": {"task_id": "tex"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let paths: Vec<_> = ["front", "left", "back", "right"]
        .iter()
        .map(|name| {
            let p = dir.path().join(format!("{name}.png"));
            std::fs::write(&p, b"png").unwrap();
            p
        })
        .collect();
    let req = texture_request(json!({
        "model": versions::texture::V3_5,
        "texture_quality": "fast",
        "delight": true,
        "texture_prompt": {"images": paths},
    }));
    let client = Client::builder()
        .api_key("tsk_test")
        .base_url(server.uri().parse().unwrap())
        .build()
        .unwrap();
    let id = client.create_task(req).await.unwrap();
    assert_eq!(id.as_str(), "tex");
}
