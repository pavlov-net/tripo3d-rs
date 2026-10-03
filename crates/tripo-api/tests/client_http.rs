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
        texture_version: None,
        delight: None,
        geometry_quality: None,
        texture_alignment: None,
        auto_size: None,
        orientation: None,
        quad: None,
        compress: None,
        generate_parts: None,
        smart_low_poly: None,
        export_uv: None,
        export_orientation: None,
    });
    let c = client(&server);
    let id = c.create_task(req).await.unwrap();
    assert_eq!(id.as_str(), "new-task");
}

#[tokio::test]
async fn create_task_uploads_image_generation_inputs() {
    use serde_json::json;
    use tripo_api::ImageInput;
    use tripo_api::tasks::TaskRequest;
    use wiremock::matchers::body_partial_json;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"code":0,"data":{"file_token":"file_up"}})),
        )
        .expect(5)
        .mount(&server)
        .await;
    for endpoint in ["image-to-image", "image-to-multiview", "edit-multiview"] {
        Mock::given(method("POST"))
            .and(path(format!("/generation/{endpoint}")))
            .and(body_partial_json(json!({"input":"file_up"})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"code":0,"data":{"task_id":"t"}})),
            )
            .expect(1)
            .mount(&server)
            .await;
    }

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"png").unwrap();
    let local = || ImageInput::Path(tmp.path().to_path_buf());
    let c = client(&server);

    let mut i2i: tripo_api::ImageToImageRequest =
        serde_json::from_value(json!({"prompt":"merge"})).unwrap();
    i2i.input = Some(local());
    i2i.inputs = Some(vec![
        local(),
        local(),
        ImageInput::FileToken("file_kept".into()),
    ]);
    c.create_task(TaskRequest::ImageToImage(i2i)).await.unwrap();

    c.create_task(TaskRequest::ImageToMultiview(
        tripo_api::ImageToMultiviewRequest { input: local() },
    ))
    .await
    .unwrap();

    let edit = tripo_api::EditMultiviewRequest {
        input: local(),
        prompts: vec![tripo_api::MultiviewEdit {
            prompt: "red shirt".into(),
            view: tripo_api::MultiviewView::Front,
        }],
    };
    c.create_task(TaskRequest::EditMultiview(edit))
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();
    let i2i_body: serde_json::Value = requests
        .iter()
        .find(|r| r.url.path() == "/generation/image-to-image")
        .unwrap()
        .body_json()
        .unwrap();
    assert_eq!(
        i2i_body["inputs"],
        json!(["file_up", "file_up", "file_kept"])
    );
}

fn success_task(server: &MockServer, with_rendered: bool) -> tripo_api::Task {
    task_with_output(
        "abc",
        tripo_api::TaskOutput {
            model_url: Some(format!("{}/files/abc.glb", server.uri())),
            rendered_image_url: with_rendered
                .then(|| format!("{}/files/abc.jpg?sig=x", server.uri())),
            ..Default::default()
        },
    )
}

/// A finished task with the given id and output.
fn task_with_output(id: &str, output: tripo_api::TaskOutput) -> tripo_api::Task {
    use std::collections::BTreeMap;
    use tripo_api::{Task, TaskId, TaskStatus};
    Task {
        task_id: TaskId::new(id),
        task_type: String::new(),
        status: TaskStatus::Success,
        input: BTreeMap::new(),
        output,
        progress: 100,
        error_code: None,
        error_message: None,
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
    let gets = server.received_requests().await.unwrap();
    assert_eq!(gets.len(), 2);
    for req in &gets {
        assert!(
            !req.headers.contains_key("authorization"),
            "download of {} must not carry the API key",
            req.url
        );
    }
}

#[tokio::test]
async fn downloads_generated_image_and_multiview_views() {
    use tripo_api::{DownloadOptions, MultiviewView, TaskOutput};

    let server = MockServer::start().await;
    for name in [
        "image.png",
        "front.png",
        "left.png",
        "back.png",
        "right.png",
    ] {
        Mock::given(method("GET"))
            .and(path(format!("/cdn/{name}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(name.as_bytes()))
            .expect(1)
            .mount(&server)
            .await;
    }
    let url = |name: &str| Some(format!("{}/cdn/{name}", server.uri()));
    let c = client(&server);
    let dir = tempfile::tempdir().unwrap();

    let image = task_with_output(
        "task_img",
        TaskOutput {
            generated_image_url: url("image.png"),
            ..Default::default()
        },
    );
    let files = c
        .download_task_models(&image, dir.path(), DownloadOptions::default())
        .await
        .unwrap();
    let generated = files.generated_image.unwrap();
    assert_eq!(generated, dir.path().join("task_img_generated.png"));
    assert_eq!(std::fs::read(generated).unwrap(), b"image.png");

    let multiview = task_with_output(
        "task_mv",
        TaskOutput {
            front_view_url: url("front.png"),
            left_view_url: url("left.png"),
            back_view_url: url("back.png"),
            right_view_url: url("right.png"),
            ..Default::default()
        },
    );
    let files = c
        .download_task_models(&multiview, dir.path(), DownloadOptions::default())
        .await
        .unwrap();
    assert_eq!(
        files.views[&MultiviewView::Back],
        dir.path().join("task_mv_back.png")
    );
    let names: Vec<_> = files
        .paths()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        names,
        [
            "task_mv_front.png",
            "task_mv_left.png",
            "task_mv_back.png",
            "task_mv_right.png"
        ]
    );
    for view in ["front", "left", "back", "right"] {
        let bytes = std::fs::read(dir.path().join(format!("task_mv_{view}.png"))).unwrap();
        assert_eq!(bytes, format!("{view}.png").as_bytes());
    }
}

#[tokio::test]
async fn downloads_smart_segment_outputs() {
    use std::collections::BTreeMap;
    use tripo_api::{DownloadOptions, Task, TaskId, TaskOutput, TaskStatus};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/out/seg_model.glb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"seg" as &[u8]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/out/mask.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"mask" as &[u8]))
        .mount(&server)
        .await;

    let task = Task {
        task_id: TaskId::new("seg"),
        task_type: "smartsegment_image".into(),
        status: TaskStatus::Success,
        input: BTreeMap::new(),
        output: TaskOutput {
            seg_model_url: Some(format!("{}/out/seg_model.glb", server.uri())),
            mask_url: Some(format!("{}/out/mask.png", server.uri())),
            ..Default::default()
        },
        progress: 100,
        error_code: None,
        error_message: None,
        created_at: String::new(),
        completed_at: None,
        credits_consumed: None,
        running_left_time: None,
        queuing_num: None,
    };
    let dir = tempfile::tempdir().unwrap();
    let out = client(&server)
        .download_task_models(&task, dir.path(), DownloadOptions::default())
        .await
        .unwrap();
    assert!(out.model.is_none());
    assert_eq!(out.seg_model, Some(dir.path().join("seg_seg.glb")));
    assert_eq!(out.mask, Some(dir.path().join("seg_mask.png")));
    assert_eq!(
        std::fs::read(dir.path().join("seg_mask.png")).unwrap(),
        b"mask"
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

#[tokio::test]
async fn import_model_uploads_local_model_first() {
    use tripo_api::tasks::TaskRequest;
    use tripo_api::{ImageInput, ImportModelRequest};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"file_token":"file_model"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/import"))
        .and(wiremock::matchers::body_json(
            serde_json::json!({"input":"file_model"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"task_id":"imported"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("chair.glb");
    std::fs::write(&model, b"glTF").unwrap();

    let req = TaskRequest::ImportModel(ImportModelRequest {
        input: ImageInput::Path(model),
    });
    let id = client(&server).create_task(req).await.unwrap();
    assert_eq!(id.as_str(), "imported");
}

#[tokio::test]
async fn smart_segment_model_without_transform_is_rejected_before_post() {
    use tripo_api::tasks::TaskRequest;
    use tripo_api::{ImageInput, MeshSmartSegmentRequest, SegType};

    let server = MockServer::start().await;
    let req = TaskRequest::MeshSmartSegment(MeshSmartSegmentRequest {
        seg_type: SegType::Model,
        input: ImageInput::parse("https://example.com/character.glb"),
        granularity: None,
        hint: None,
        transform: None,
    });
    let err = client(&server).create_task(req).await.unwrap_err();
    assert!(matches!(err, Error::InvalidRequest(_)), "{err}");
    assert!(server.received_requests().await.unwrap().is_empty());
}
