#![cfg(feature = "schemars")]

use schemars::schema_for;
use tripo_api::TextToModelRequest;

#[test]
fn text_to_model_request_has_schema() {
    let schema = schema_for!(TextToModelRequest);
    let v = serde_json::to_value(&schema).unwrap();
    assert!(v["properties"]["prompt"].is_object());
}
