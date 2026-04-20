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
