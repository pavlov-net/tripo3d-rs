use serde_json::json;
use tripo_api::{TaskRequest, versions};

fn requests(model: &str, quad: Option<bool>, face_limit: Option<i32>) -> Vec<TaskRequest> {
    let mut body = json!({"model": model, "quad": quad, "face_limit": face_limit});
    body["prompt"] = json!("a wooden chair");
    let text = TaskRequest::TextToModel(serde_json::from_value(body.clone()).unwrap());
    body.as_object_mut().unwrap().remove("prompt");
    body["input"] = json!("https://example.com/front.png");
    let image = TaskRequest::ImageToModel(serde_json::from_value(body.clone()).unwrap());
    body.as_object_mut().unwrap().remove("input");
    body["inputs"] = json!(["https://example.com/front.png"]);
    let multiview = TaskRequest::MultiviewToModel(serde_json::from_value(body).unwrap());
    vec![text, image, multiview]
}

#[test]
fn p2_ranges_and_adaptive_sizing_on_all_generation_endpoints() {
    for quad in [None, Some(false), Some(true)] {
        let maximum = if quad == Some(true) { 25_000 } else { 50_000 };
        for limit in [None, Some(48), Some(maximum)] {
            for req in requests(versions::text_image::P2, quad, limit) {
                req.validate().unwrap();
                let body = serde_json::to_value(req).unwrap();
                assert_eq!(body["model"], "P2-20260801");
                assert_eq!(body.get("quad"), quad.map(|v| json!(v)).as_ref());
                assert_eq!(body.get("face_limit"), limit.map(|v| json!(v)).as_ref());
            }
        }
        for limit in [-1, 0, 47, maximum + 1] {
            for req in requests(versions::multiview::P2, quad, Some(limit)) {
                assert!(
                    req.validate()
                        .unwrap_err()
                        .to_string()
                        .contains("face_limit")
                );
            }
        }
    }
}

#[test]
fn p1_alias_preserves_unsupported_parameter_checks() {
    for model in [versions::text_image::P1, "tripo-p1"] {
        for field in [
            "quad",
            "smart_low_poly",
            "generate_parts",
            "geometry_quality",
        ] {
            let mut body = json!({"prompt": "chair", "model": model});
            body[field] = if field == "geometry_quality" {
                json!("standard")
            } else {
                json!(true)
            };
            let req = TaskRequest::TextToModel(serde_json::from_value(body).unwrap());
            assert!(req.validate().unwrap_err().to_string().contains(field));
        }
    }
}

#[test]
fn p2_limits_do_not_restrict_other_models() {
    for model in [versions::text_image::V3_1, "future-model"] {
        for req in requests(model, Some(true), Some(100_000)) {
            req.validate().unwrap();
        }
    }
}
