use std::sync::{Arc, Mutex};
use std::time::Duration;

use tripo_api::{Client, TaskId, TaskStatus, WaitOptions};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Client {
    Client::builder()
        .api_key("tsk_test")
        .base_url(server.uri().parse().unwrap())
        .build()
        .unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn waits_until_terminal() {
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"task_id":"abc","type":"text_to_model","status":"running","progress":10,"create_time":0}
        })))
        .up_to_n_times(2)
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"task_id":"abc","type":"text_to_model","status":"success","progress":100,"create_time":0,
                              "output":{"model":"https://cdn/abc.glb"}}
        })))
        .mount(&server).await;

    let c = client(&server);
    let seen = Arc::new(Mutex::new(0u32));
    let s2 = seen.clone();
    let opts = WaitOptions {
        timeout: Some(Duration::from_mins(1)),
        initial_interval: Duration::from_millis(10),
        max_interval: Duration::from_millis(50),
        on_progress: Some(Box::new(move |t| {
            *s2.lock().unwrap() += 1;
            let _ = t;
        })),
    };
    let task = c.wait_for_task(&TaskId::new("abc"), opts).await.unwrap();
    assert_eq!(task.status, TaskStatus::Success);
    assert_eq!(task.output.model.as_deref(), Some("https://cdn/abc.glb"));
    assert!(*seen.lock().unwrap() >= 3);
}

#[tokio::test(flavor = "current_thread")]
async fn times_out() {
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/task/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0, "data":{"task_id":"abc","type":"text_to_model","status":"running","progress":0,"create_time":0}
        })))
        .mount(&server).await;

    let c = client(&server);
    let err = c
        .wait_for_task(
            &TaskId::new("abc"),
            WaitOptions {
                timeout: Some(Duration::from_millis(50)),
                initial_interval: Duration::from_millis(10),
                max_interval: Duration::from_millis(10),
                on_progress: None,
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, tripo_api::Error::WaitTimeout(_)));
}
