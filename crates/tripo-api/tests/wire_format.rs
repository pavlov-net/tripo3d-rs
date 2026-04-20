//! Snapshot tests for the serialized JSON body of every `TaskRequest` variant.
//! These lock down byte-exact wire-format compatibility with the Python SDK.

use serde_json::Value;
use tripo_api::{
    enums::Quality, tasks::TaskRequest, CompressionMode, TextToModelRequest,
};

fn json_of<T: serde::Serialize>(t: &T) -> Value {
    serde_json::to_value(t).expect("serialize")
}

#[test]
fn text_to_model_minimal() {
    let req = TaskRequest::TextToModel(TextToModelRequest {
        prompt: "a red robot".into(),
        ..Default::default()
    });
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "prompt": "a red robot",
      "type": "text_to_model"
    }
    "###);
}

#[test]
fn text_to_model_full() {
    let req = TaskRequest::TextToModel(TextToModelRequest {
        prompt: "a red robot".into(),
        negative_prompt: Some("low quality".into()),
        model_version: Some("v2.5-20250123".into()),
        texture_quality: Some(Quality::Detailed),
        geometry_quality: Some(Quality::Standard),
        auto_size: Some(true),
        quad: Some(false),
        compress: Some(CompressionMode::Geometry),
        ..Default::default()
    });
    insta::assert_json_snapshot!(json_of(&req));
}
