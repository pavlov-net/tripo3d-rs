//! `POST /tasks/list` and `GET /account/usage` against every response shape
//! the docs and SDKs disagree on.

use serde_json::json;
use tripo_api::{
    Client, Error, MAX_LIST_TASK_IDS, RetryPolicy, TaskId, TaskStatus, Timestamp, UsageQuery,
};
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Client {
    Client::builder()
        .api_key("tsk_test")
        .base_url(server.uri().parse().unwrap())
        .retry(RetryPolicy {
            max_attempts: 3,
            base_delay: std::time::Duration::from_millis(1),
            max_delay: std::time::Duration::from_millis(10),
        })
        .build()
        .unwrap()
}

fn ids(raw: &[&str]) -> Vec<TaskId> {
    raw.iter().copied().map(TaskId::from).collect()
}

fn task_json(id: &str, status: &str) -> serde_json::Value {
    json!({"task_id": id, "type": "text_to_model", "status": status, "progress": 100})
}

async fn mock_list(server: &MockServer, requested: &[&str], data: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path("/tasks/list"))
        .and(body_json(json!({ "task_ids": requested })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code": 0, "data": data})))
        .expect(1)
        .mount(server)
        .await;
}

fn task_ids(list: &tripo_api::TaskList) -> Vec<&str> {
    list.tasks.iter().map(|t| t.task_id.as_str()).collect()
}

#[tokio::test]
async fn list_tasks_map_form_orders_by_request() {
    let server = MockServer::start().await;
    let requested = ["task_b", "task_gone", "task_a"];
    mock_list(
        &server,
        &requested,
        json!({
            "tasks": {
                "task_a": task_json("task_a", "success"),
                // Key supplies the id when the value omits it.
                "task_b": {"type": "image_to_model", "status": "running", "progress": 60}
            },
            "missed": ["task_gone"]
        }),
    )
    .await;

    let list = client(&server).list_tasks(&ids(&requested)).await.unwrap();
    assert_eq!(task_ids(&list), ["task_b", "task_a"]);
    assert_eq!(list.tasks[0].status, TaskStatus::Running);
    assert_eq!(list.missed, ids(&["task_gone"]));
}

#[tokio::test]
async fn list_tasks_array_form_derives_missed() {
    let server = MockServer::start().await;
    let requested = ["task_a", "task_gone"];
    mock_list(
        &server,
        &requested,
        json!({"tasks": [task_json("task_a", "success")]}),
    )
    .await;

    let list = client(&server).list_tasks(&ids(&requested)).await.unwrap();
    assert_eq!(task_ids(&list), ["task_a"]);
    assert_eq!(list.missed, ids(&["task_gone"]));
}

#[tokio::test]
async fn list_tasks_bare_array_form() {
    let server = MockServer::start().await;
    let requested = ["task_a", "task_b"];
    mock_list(
        &server,
        &requested,
        json!([
            task_json("task_a", "success"),
            task_json("task_b", "failed")
        ]),
    )
    .await;

    let list = client(&server).list_tasks(&ids(&requested)).await.unwrap();
    assert_eq!(task_ids(&list), ["task_a", "task_b"]);
    assert!(list.missed.is_empty());
}

#[tokio::test]
async fn list_tasks_rejects_empty_and_oversized_without_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    let c = client(&server);

    let err = c.list_tasks(&[]).await.unwrap_err();
    assert!(matches!(err, Error::InvalidRequest(_)), "{err:?}");

    let too_many: Vec<TaskId> = (0..=MAX_LIST_TASK_IDS)
        .map(|i| TaskId::new(format!("task_{i}")))
        .collect();
    let err = c.list_tasks(&too_many).await.unwrap_err();
    assert!(matches!(err, Error::InvalidRequest(_)), "{err:?}");
}

#[tokio::test]
async fn list_tasks_accepts_exactly_max_ids() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/tasks/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0, "data": {"tasks": {}, "missed": []}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let max: Vec<TaskId> = (0..MAX_LIST_TASK_IDS)
        .map(|i| TaskId::new(format!("task_{i}")))
        .collect();
    let list = client(&server).list_tasks(&max).await.unwrap();
    assert!(list.tasks.is_empty());
    assert!(list.missed.is_empty());
}

#[tokio::test]
async fn list_tasks_retries_server_errors() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/tasks/list"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    mock_list(
        &server,
        &["task_a"],
        json!({"tasks": {"task_a": task_json("task_a", "success")}, "missed": []}),
    )
    .await;

    let list = client(&server).list_tasks(&ids(&["task_a"])).await.unwrap();
    assert_eq!(task_ids(&list), ["task_a"]);
}

async fn mock_usage(server: &MockServer, data: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path("/account/usage"))
        .and(query_param_is_missing("limit"))
        .and(query_param_is_missing("offset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code": 0, "data": data})))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
#[allow(clippy::float_cmp)] // exact literals round-trip losslessly through JSON
async fn get_usage_documented_iso_shape() {
    let server = MockServer::start().await;
    mock_usage(
        &server,
        json!([{
            "task_id": "task_abc123", "type": "text_to_model",
            "credits_consumed": 100.25, "created_at": "2025-01-01T00:00:00Z"
        }]),
    )
    .await;

    let usage = client(&server)
        .get_usage(UsageQuery::default())
        .await
        .unwrap();
    assert_eq!(usage.len(), 1);
    let rec = &usage[0];
    assert_eq!(rec.task_id, Some(TaskId::new("task_abc123")));
    assert_eq!(rec.task_type.as_deref(), Some("text_to_model"));
    assert_eq!(rec.credits_consumed, Some(100.25));
    assert_eq!(
        rec.created_at,
        Some(Timestamp::Text("2025-01-01T00:00:00Z".into()))
    );
    assert!(rec.status.is_none());
    assert!(rec.extra.is_empty());
}

#[tokio::test]
async fn get_usage_epoch_create_time_status_and_extras() {
    let server = MockServer::start().await;
    mock_usage(
        &server,
        json!([{
            "task_id": "task_x", "type": "rig_model", "status": "success",
            "create_time": 1_767_225_600, "credits_consumed": 25, "model_version": "v3"
        }]),
    )
    .await;

    let usage = client(&server)
        .get_usage(UsageQuery::default())
        .await
        .unwrap();
    let rec = &usage[0];
    assert_eq!(rec.status.as_deref(), Some("success"));
    assert_eq!(
        rec.created_at,
        Some(Timestamp::Epoch(1_767_225_600_u64.into()))
    );
    assert_eq!(rec.created_at.as_ref().unwrap().to_string(), "1767225600");
    assert_eq!(rec.credits_consumed, Some(25.0));
    assert_eq!(rec.extra["model_version"], "v3");
    // Extras survive re-serialization.
    assert_eq!(serde_json::to_value(rec).unwrap()["model_version"], "v3");
}

#[tokio::test]
async fn get_usage_paging_params() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/account/usage"))
        .and(query_param("limit", "20"))
        .and(query_param("offset", "40"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0,
            "data": [{"task_id": "task_y", "credits_consumed": 10.0}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let usage = client(&server)
        .get_usage(UsageQuery {
            limit: Some(20),
            offset: Some(40),
        })
        .await
        .unwrap();
    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].task_id, Some(TaskId::new("task_y")));
}
