use serde_json::json;
use tripo_api::{ImageToModelRequest, MultiviewToModelRequest, TextToModelRequest};

#[test]
fn generation_orientation_roundtrips_and_rejects_invalid_axes() {
    for axis in [
        None,
        Some("+x"),
        Some("+y"),
        Some("-x"),
        Some("-y"),
        Some("+z"),
    ] {
        let mut body = json!({});
        if let Some(axis) = axis {
            body["export_orientation"] = json!(axis);
        }
        for (field, input) in [
            ("prompt", json!("chair")),
            ("input", json!("https://example.com/front.png")),
            ("inputs", json!(["https://example.com/front.png"])),
        ] {
            let mut request = body.clone();
            request[field] = input;
            let output = match field {
                "prompt" => serde_json::from_value::<TextToModelRequest>(request)
                    .map(|r| serde_json::to_value(r).unwrap()),
                "input" => serde_json::from_value::<ImageToModelRequest>(request)
                    .map(|r| serde_json::to_value(r).unwrap()),
                _ => serde_json::from_value::<MultiviewToModelRequest>(request)
                    .map(|r| serde_json::to_value(r).unwrap()),
            };
            if axis == Some("+z") {
                assert!(output.is_err());
            } else {
                assert_eq!(
                    output.unwrap().get("export_orientation"),
                    body.get("export_orientation")
                );
            }
        }
    }
}
