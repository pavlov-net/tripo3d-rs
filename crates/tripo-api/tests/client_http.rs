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
        .and(path("/account/balance"))
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
        .and(path("/account/balance"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "code": 1001, "message": "bad key", "suggestion": "rotate",
            "request_id": "req_123"
        })))
        .mount(&server)
        .await;

    let err = client(&server).get_balance().await.unwrap_err();
    let Error::Api {
        code,
        message,
        suggestion,
        request_id,
    } = err
    else {
        panic!("wrong variant: {err:?}")
    };
    assert_eq!(code, 1001);
    assert_eq!(message, "bad key");
    assert_eq!(suggestion.as_deref(), Some("rotate"));
    assert_eq!(request_id.as_deref(), Some("req_123"));
}

#[tokio::test]
async fn get_task_parses_full_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tasks/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "status": "success",
            "data": {
                "task_id": "abc123",
                "type": "text_to_model",
                "status": "running",
                "progress": 65,
                "created_at": "2026-04-28T12:00:00Z",
                "running_left_time": 20,
                "output": {
                    "model_url": "https://cdn.example.com/abc123.glb",
                    "rendered_image_url": "https://cdn.example.com/abc123.jpg"
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
    assert_eq!(task.created_at, "2026-04-28T12:00:00Z");
    assert_eq!(
        task.output.model_url.as_deref(),
        Some("https://cdn.example.com/abc123.glb")
    );
}

#[tokio::test]
async fn upload_file_roundtrip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": { "file_token": "file_abc123" }
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"jpeg bytes").unwrap();

    let c = client(&server);
    let up = c.upload_file(tmp.path()).await.unwrap();
    assert_eq!(up.file_token, "file_abc123");
}

#[tokio::test]
async fn create_task_uploads_local_image_first() {
    use tripo_api::tasks::TaskRequest;
    use tripo_api::{ImageInput, ImageToModelRequest};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"file_token":"file_abc123"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/generation/image-to-model"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"task_id":"new-task"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"jpeg").unwrap();

    let req = TaskRequest::ImageToModel(ImageToModelRequest {
        input: ImageInput::Path(tmp.path().to_path_buf()),
        model: None,
        enable_image_autofix: None,
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
        export_uv: None,
    });
    let c = client(&server);
    let id = c.create_task(req).await.unwrap();
    assert_eq!(id.as_str(), "new-task");
}

fn success_task(server: &MockServer, with_rendered: bool) -> tripo_api::Task {
    use std::collections::BTreeMap;
    use tripo_api::{Task, TaskId, TaskOutput, TaskStatus};
    Task {
        task_id: TaskId::new("abc"),
        task_type: "text_to_model".into(),
        status: TaskStatus::Success,
        input: BTreeMap::new(),
        output: TaskOutput {
            model_url: Some(format!("{}/files/abc.glb", server.uri())),
            rendered_image_url: with_rendered
                .then(|| format!("{}/files/abc.jpg?sig=x", server.uri())),
            generated_image_url: None,
            riggable: None,
            rig_type: None,
        },
        progress: 100,
        created_at: "2026-04-28T12:00:00Z".into(),
        completed_at: None,
        credits_consumed: None,
        running_left_time: None,
        queuing_num: None,
    }
}

#[tokio::test]
async fn downloads_model_and_rendered_image() {
    use tripo_api::DownloadOptions;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/abc.glb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"model-bytes" as &[u8]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/files/abc.jpg"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"jpg-bytes" as &[u8]))
        .mount(&server)
        .await;

    let c = client(&server);
    let task = success_task(&server, true);

    let dir = tempfile::tempdir().unwrap();
    let out = c
        .download_task_models(&task, dir.path(), DownloadOptions::default())
        .await
        .unwrap();
    assert!(out.model.is_some());
    assert_eq!(std::fs::read(out.model.unwrap()).unwrap(), b"model-bytes");
    assert_eq!(
        std::fs::read(out.rendered_image.unwrap()).unwrap(),
        b"jpg-bytes"
    );
}

#[tokio::test]
async fn download_errors_on_existing_file_without_overwrite() {
    use tripo_api::DownloadOptions;
    let server = MockServer::start().await;
    let c = client(&server);

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("abc.glb"), b"pre-existing").unwrap();

    let task = success_task(&server, false);
    let err = c
        .download_task_models(&task, dir.path(), DownloadOptions::default())
        .await
        .unwrap_err();
    assert!(matches!(err, tripo_api::Error::FileExists(_)));
}

#[tokio::test]
#[allow(clippy::float_cmp)] // compare JSON numbers against the same parsed literals
async fn task_credits_preserve_fractional_and_whole_numbers() {
    let server = MockServer::start().await;
    for (index, credits) in ["48.00", "48.27", "48", "0.01", "0"].iter().enumerate() {
        let id = format!("credits-{index}");
        Mock::given(method("GET"))
            .and(path(format!("/tasks/{id}")))
            // Keep the original number spelling to cover whole-valued decimal JSON.
            .respond_with(ResponseTemplate::new(200).set_body_raw(format!(
                r#"{{"code":0,"data":{{"task_id":"{id}","type":"text_to_model","status":"success","progress":100,"created_at":"2026-08-01T00:00:00Z","credits_consumed":{credits}}}}}"#
            ), "application/json"))
            .expect(1).mount(&server).await;
        let task = client(&server).get_task(&id.into()).await.unwrap();
        let expected: f64 = credits.parse().unwrap();
        assert_eq!(task.credits_consumed, Some(expected));
        assert_eq!(
            serde_json::to_value(task).unwrap()["credits_consumed"],
            expected
        );
    }
}
