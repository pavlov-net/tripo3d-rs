use tripo_api::{Client, RetryPolicy};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "current_thread")]
#[allow(clippy::float_cmp)] // exact literals round-trip losslessly through JSON
async fn retries_on_500_then_succeeds() {
    let server = MockServer::start().await;

    // First response: 500. Subsequent: 200.
    Mock::given(method("GET"))
        .and(path("/user/balance"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code":0,"data":{"balance":1.0,"frozen":0.0}
        })))
        .mount(&server)
        .await;

    let c = Client::builder()
        .api_key("tsk_test")
        .base_url(server.uri().parse().unwrap())
        .retry(RetryPolicy {
            max_attempts: 3,
            base_delay: std::time::Duration::from_millis(1),
            max_delay: std::time::Duration::from_millis(10),
        })
        .build()
        .unwrap();
    let bal = c.get_balance().await.unwrap();
    assert_eq!(bal.balance, 1.0);
}

#[tokio::test(flavor = "current_thread")]
async fn no_retry_on_400() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user/balance"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "code": 4000, "message": "bad"
        })))
        .expect(1) // must be called exactly once
        .mount(&server)
        .await;

    let c = Client::builder()
        .api_key("tsk_test")
        .base_url(server.uri().parse().unwrap())
        .retry(RetryPolicy {
            max_attempts: 3,
            base_delay: std::time::Duration::from_millis(1),
            max_delay: std::time::Duration::from_millis(10),
        })
        .build()
        .unwrap();
    let _ = c.get_balance().await;
}
