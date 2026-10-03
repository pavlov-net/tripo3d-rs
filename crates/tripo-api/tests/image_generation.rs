//! Image-generation request validation through the public `TaskRequest`
//! surface. The per-model rule matrix is unit-tested in `tasks::image_rules`.

use serde_json::json;
use tripo_api::{TaskRequest, versions};

fn text_to_image(body: serde_json::Value) -> TaskRequest {
    TaskRequest::TextToImage(serde_json::from_value(body).unwrap())
}

fn image_to_image(body: serde_json::Value) -> TaskRequest {
    TaskRequest::ImageToImage(serde_json::from_value(body).unwrap())
}

fn err_of(req: &TaskRequest) -> String {
    req.validate().unwrap_err().to_string()
}

#[test]
fn quality_checked_only_for_explicit_models() {
    // Without a model the server picks one, so nothing is checked locally.
    text_to_image(json!({"prompt":"x","quality":"low"}))
        .validate()
        .unwrap();
    assert!(
        err_of(&image_to_image(
            json!({"input":"https://e/a.png","prompt":"x","model":"seedream_v5","quality":"low"})
        ))
        .contains("does not accept quality")
    );
}

#[test]
fn transparent_jpeg_rejected_on_both_endpoints() {
    let body = json!({
        "prompt": "x", "model": versions::image::CHAT_IMAGE_2_5_FLARE,
        "background": "transparent", "output_format": "jpeg"
    });
    assert!(err_of(&text_to_image(body.clone())).contains("png"));
    let mut body = body;
    body["input"] = json!("https://e/a.png");
    assert!(err_of(&image_to_image(body)).contains("png"));
}

#[test]
fn image_to_image_input_and_prompt_rules() {
    assert!(err_of(&image_to_image(json!({"prompt":"x"}))).contains("input"));
    assert!(err_of(&image_to_image(json!({"inputs":[],"prompt":"x"}))).contains("input"));
    assert!(err_of(&image_to_image(json!({"input":"https://e/a.png"}))).contains("template"));
    image_to_image(json!({"input":"https://e/a.png","template":"t_pose"}))
        .validate()
        .unwrap();
    image_to_image(json!({"inputs":["https://e/a.png"],"prompt":"x"}))
        .validate()
        .unwrap();
}

#[test]
fn image_to_image_counts_inputs() {
    let five: Vec<String> = (0..5).map(|i| format!("https://e/{i}.png")).collect();
    assert!(
        err_of(&image_to_image(
            json!({"inputs":five,"prompt":"x","model":"seedream_v5"})
        ))
        .contains("at most 4")
    );
    image_to_image(json!({"inputs":five,"prompt":"x"}))
        .validate()
        .unwrap();
}

#[test]
fn edit_multiview_needs_prompts() {
    let req = TaskRequest::EditMultiview(
        serde_json::from_value(json!({"input":"task_mv","prompts":[]})).unwrap(),
    );
    assert!(err_of(&req).contains("prompt"));
    assert!(
        serde_json::from_value::<tripo_api::EditMultiviewRequest>(
            json!({"input":"task_mv","prompts":[{"prompt":"x","view":"top"}]})
        )
        .is_err()
    );
}

#[test]
fn edit_multiview_rejects_more_than_four_prompts() {
    let req = |prompts: serde_json::Value| {
        TaskRequest::EditMultiview(
            serde_json::from_value(json!({"input":"task_mv","prompts":prompts})).unwrap(),
        )
    };
    let all = json!([
        {"prompt":"a","view":"front"},
        {"prompt":"b","view":"left"},
        {"prompt":"c","view":"back"},
        {"prompt":"d","view":"right"},
    ]);
    req(all).validate().unwrap();
    let five = json!([
        {"prompt":"a","view":"front"},
        {"prompt":"b","view":"left"},
        {"prompt":"c","view":"back"},
        {"prompt":"d","view":"right"},
        {"prompt":"e","view":"front"},
    ]);
    assert!(err_of(&req(five)).contains("at most 4"));
}

#[test]
fn image_to_image_allows_input_and_inputs() {
    image_to_image(json!({
        "input":"https://e/a.png",
        "inputs":["https://e/b.png"],
        "prompt":"x",
        "model":"seedream_v5"
    }))
    .validate()
    .unwrap();
}
