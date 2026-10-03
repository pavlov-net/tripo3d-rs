use std::time::Duration;

use tripo_api::{Client, ClientBuilder, Error, RetryPolicy};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const FILE_BYTES: &[u8] = b"glTF\x02\x00\x00\x00 binary model bytes";

fn builder(server: &MockServer) -> ClientBuilder {
    Client::builder()
        .api_key("tsk_test")
        .base_url(server.uri().parse().unwrap())
        .retry(RetryPolicy {
            max_attempts: 2,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
        })
}

fn model_file() -> tempfile::NamedTempFile {
    let tmp = tempfile::Builder::new().suffix(".GLB").tempfile().unwrap();
    std::fs::write(tmp.path(), FILE_BYTES).unwrap();
    tmp
}

/// Mount `POST /files/presign` for `format`, pointing the upload at
/// `/storage/obj` on the same mock server.
async fn mount_presign(server: &MockServer, format: &str, url_field: &str) {
    Mock::given(method("POST"))
        .and(path("/files/presign"))
        .and(header("authorization", "Bearer tsk_test"))
        .and(body_json(serde_json::json!({ "format": format })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": {
                url_field: format!("{}/storage/obj?X-Amz-Signature=sig", server.uri()),
                "file_token": "file_big",
                "expires_in": 1800
            }
        })))
        .expect(1)
        .mount(server)
        .await;
}

async fn storage_puts(server: &MockServer) -> Vec<Request> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.method.as_str() == "PUT")
        .collect()
}

fn assert_raw_put(req: &Request) {
    assert_eq!(req.url.path(), "/storage/obj");
    assert_eq!(req.url.query(), Some("X-Amz-Signature=sig"));
    assert!(
        !req.headers.contains_key("authorization"),
        "presigned PUT must not carry the API key"
    );
    assert_eq!(
        req.headers.get("content-type").unwrap(),
        "application/octet-stream"
    );
    assert_eq!(
        req.headers.get("content-length").unwrap(),
        &FILE_BYTES.len().to_string()
    );
    assert!(!req.headers.contains_key("transfer-encoding"));
    assert_eq!(req.body, FILE_BYTES);
}

#[tokio::test]
async fn presign_upload_posts_format_without_dot() {
    let server = MockServer::start().await;
    mount_presign(&server, "glb", "presigned_url").await;

    let c = builder(&server).build().unwrap();
    let p = c.presign_upload(".glb").await.unwrap();
    assert_eq!(p.file_token, "file_big");
    assert_eq!(p.expires_in, Some(1800));
    assert!(
        p.presigned_url
            .ends_with("/storage/obj?X-Amz-Signature=sig")
    );
}

#[tokio::test]
async fn upload_file_presigned_streams_raw_bytes_without_auth() {
    let server = MockServer::start().await;
    mount_presign(&server, "glb", "presigned_url").await;
    Mock::given(method("PUT"))
        .and(path("/storage/obj"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = model_file();
    let c = builder(&server).build().unwrap();
    let up = c.upload_file_presigned(tmp.path()).await.unwrap();
    assert_eq!(up.file_token, "file_big");
    assert_raw_put(&storage_puts(&server).await[0]);
}

#[tokio::test]
async fn upload_url_alias_is_followed() {
    let server = MockServer::start().await;
    mount_presign(&server, "glb", "upload_url").await;
    Mock::given(method("PUT"))
        .and(path("/storage/obj"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let tmp = model_file();
    let c = builder(&server).build().unwrap();
    assert_eq!(
        c.upload_file_presigned(tmp.path())
            .await
            .unwrap()
            .file_token,
        "file_big"
    );
}

#[tokio::test]
async fn presigned_put_retries_server_errors_with_full_body() {
    let server = MockServer::start().await;
    mount_presign(&server, "glb", "presigned_url").await;
    Mock::given(method("PUT"))
        .and(path("/storage/obj"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/obj"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let tmp = model_file();
    let c = builder(&server).build().unwrap();
    c.upload_file_presigned(tmp.path()).await.unwrap();
    let puts = storage_puts(&server).await;
    assert_eq!(puts.len(), 2);
    puts.iter().for_each(assert_raw_put);
}

#[tokio::test]
async fn presigned_put_failure_surfaces_storage_error() {
    let server = MockServer::start().await;
    mount_presign(&server, "glb", "presigned_url").await;
    Mock::given(method("PUT"))
        .and(path("/storage/obj"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_string("<Error><Code>SignatureDoesNotMatch</Code></Error>"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let tmp = model_file();
    let c = builder(&server).build().unwrap();
    let err = c.upload_file_presigned(tmp.path()).await.unwrap_err();
    match err {
        Error::Http { status, message } => {
            assert_eq!(status, 403);
            assert!(message.contains("SignatureDoesNotMatch"));
        }
        other => panic!("expected Http error, got {other:?}"),
    }
}

#[tokio::test]
async fn upload_file_presigned_rejects_unsupported_extension_before_any_request() {
    let server = MockServer::start().await;
    let tmp = tempfile::Builder::new()
        .suffix(".blend")
        .tempfile()
        .unwrap();
    std::fs::write(tmp.path(), FILE_BYTES).unwrap();

    let c = builder(&server).build().unwrap();
    let err = c.upload_file_presigned(tmp.path()).await.unwrap_err();
    assert!(matches!(err, Error::InvalidRequest(_)), "{err}");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn upload_file_switches_to_presign_above_threshold() {
    let server = MockServer::start().await;
    mount_presign(&server, "glb", "presigned_url").await;
    Mock::given(method("PUT"))
        .and(path("/storage/obj"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let tmp = model_file();
    let threshold = u64::try_from(FILE_BYTES.len()).unwrap() - 1;
    let c = builder(&server)
        .presign_threshold(threshold)
        .build()
        .unwrap();
    assert_eq!(
        c.upload_file(tmp.path()).await.unwrap().file_token,
        "file_big"
    );
}

#[tokio::test]
async fn upload_file_uses_multipart_at_or_below_threshold() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0, "data": { "file_token": "file_small" }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/files/presign"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let tmp = model_file();
    let threshold = u64::try_from(FILE_BYTES.len()).unwrap();
    let c = builder(&server)
        .presign_threshold(threshold)
        .build()
        .unwrap();
    assert_eq!(
        c.upload_file(tmp.path()).await.unwrap().file_token,
        "file_small"
    );
}
