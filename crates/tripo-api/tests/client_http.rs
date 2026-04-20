use tripo_api::{Client, Error};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Client {
    Client::builder()
        .api_key("tsk_test")
        .base_url(server.uri().parse().unwrap())
        .build()
        .unwrap()
}

#[tokio::test]
#[allow(clippy::float_cmp)] // exact literals round-trip losslessly through JSON
async fn get_balance_happy_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/balance"))
        .and(header("authorization", "Bearer tsk_test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": { "balance": 42.5, "frozen": 1.0 }
        })))
        .mount(&server)
        .await;

    let c = client(&server);
    let bal = c.get_balance().await.unwrap();
    assert_eq!(bal.balance, 42.5);
    assert_eq!(bal.frozen, 1.0);
}

#[tokio::test]
async fn get_balance_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/balance"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "code": 1001, "message": "bad key", "suggestion": "rotate"
        })))
        .mount(&server)
        .await;

    let err = client(&server).get_balance().await.unwrap_err();
    let Error::Api { code, message, suggestion } = err else { panic!("wrong variant: {err:?}") };
    assert_eq!(code, 1001);
    assert_eq!(message, "bad key");
    assert_eq!(suggestion.as_deref(), Some("rotate"));
}

#[tokio::test]
async fn get_task_parses_full_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/task/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": {
                "task_id": "abc123",
                "type": "text_to_model",
                "status": "running",
                "progress": 65,
                "create_time": 1_700_000_000,
                "running_left_time": 20,
                "output": {
                    "model": "https://cdn.example.com/abc123.glb",
                    "rendered_image": "https://cdn.example.com/abc123.jpg"
                }
            }
        })))
        .mount(&server)
        .await;

    let c = client(&server);
    let task = c.get_task(&"abc123".into()).await.unwrap();
    assert_eq!(task.task_id.as_str(), "abc123");
    assert_eq!(task.status, tripo_api::TaskStatus::Running);
    assert_eq!(task.progress, 65);
    assert_eq!(task.running_left_time, Some(20));
    assert_eq!(task.output.model.as_deref(), Some("https://cdn.example.com/abc123.glb"));
}

#[tokio::test]
async fn get_task_cn_region_header() {
    use tripo_api::Region;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/task/abc123"))
        .and(header("x-tripo-region", "rg2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": { "task_id": "abc123", "type": "text_to_model", "status": "queued", "progress": 0, "create_time": 0 }
        })))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("tsk_test")
        .region(Region::Cn)
        .base_url(server.uri().parse().unwrap())
        .build()
        .unwrap();
    c.get_task(&"abc123".into()).await.unwrap();
}

#[tokio::test]
async fn upload_file_roundtrip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/upload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": { "image_token": "550e8400-e29b-41d4-a716-446655440000" }
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"jpeg bytes").unwrap();

    let c = client(&server);
    let up = c.upload_file(tmp.path()).await.unwrap();
    assert_eq!(up.file_token.to_string(), "550e8400-e29b-41d4-a716-446655440000");
}

#[tokio::test]
async fn create_task_uploads_local_image_first() {
    use tripo_api::tasks::TaskRequest;
    use tripo_api::{ImageInput, ImageToModelRequest};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/upload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"image_token":"550e8400-e29b-41d4-a716-446655440000"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/task"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"task_id":"new-task"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"jpeg").unwrap();

    let req = TaskRequest::ImageToModel(ImageToModelRequest {
        image: ImageInput::Path(tmp.path().to_path_buf()),
        model_version: None,
        face_limit: None,
        texture: None,
        pbr: None,
        model_seed: None,
        texture_seed: None,
        texture_quality: None,
        geometry_quality: None,
        texture_alignment: None,
        auto_size: None,
        orientation: None,
        quad: None,
        compress: None,
        generate_parts: None,
        smart_low_poly: None,
    });
    let c = client(&server);
    let id = c.create_task(req).await.unwrap();
    assert_eq!(id.as_str(), "new-task");
}
