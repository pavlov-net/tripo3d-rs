//! Snapshot tests for the serialized JSON body of every `TaskRequest` variant.
//! These lock down byte-exact wire-format compatibility with the Python SDK.

use serde_json::Value;
use tripo_api::{
    enums::Quality, tasks::TaskRequest, CompressionMode, ConvertModelRequest, FbxPreset,
    ImageInput, ImageToModelRequest, MultiviewToModelRequest, OutputFormat, TextToModelRequest,
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

#[test]
fn image_to_model_file_token() {
    let req = TaskRequest::ImageToModel(ImageToModelRequest {
        image: ImageInput::FileToken(
            uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        ),
        texture: Some(true),
        pbr: Some(false),
        quad: Some(true),
        ..default_image_to_model()
    });
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn image_to_model_url() {
    let req = TaskRequest::ImageToModel(ImageToModelRequest {
        image: ImageInput::Url("https://example.com/x.jpg".parse().unwrap()),
        ..default_image_to_model()
    });
    insta::assert_json_snapshot!(json_of(&req));
}

fn default_image_to_model() -> ImageToModelRequest {
    ImageToModelRequest {
        image: ImageInput::FileToken(uuid::Uuid::nil()),
        model_version: None,
        face_limit: None,
        texture: None,
        pbr: None,
        model_seed: None,
        texture_seed: None,
        texture_quality: None,
        geometry_quality: None,
        texture_alignment: None,
        auto_size: None,
        orientation: None,
        quad: None,
        compress: None,
        generate_parts: None,
        smart_low_poly: None,
    }
}

#[test]
fn multiview_to_model_with_empty_slot() {
    let req = TaskRequest::MultiviewToModel(MultiviewToModelRequest {
        images: vec![
            Some(ImageInput::Url("https://example.com/front.jpg".parse().unwrap())),
            None,
            Some(ImageInput::FileToken(
                uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            )),
        ],
        model_version: None,
        face_limit: None,
        texture: None,
        pbr: None,
        model_seed: None,
        texture_seed: None,
        texture_quality: None,
        geometry_quality: None,
        texture_alignment: None,
        auto_size: None,
        orientation: None,
        quad: None,
        compress: None,
        generate_parts: None,
        smart_low_poly: None,
    });
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn convert_model_minimal_gltf() {
    let req = TaskRequest::ConvertModel(ConvertModelRequest {
        original_model_task_id: "src-task-1".into(),
        format: OutputFormat::Gltf,
        quad: None,
        force_symmetry: None,
        face_limit: None,
        flatten_bottom: None,
        flatten_bottom_threshold: None,
        texture_size: None,
        texture_format: None,
        scale_factor: None,
        pivot_to_center_bottom: None,
        with_animation: None,
        pack_uv: None,
        bake: None,
        part_names: None,
        export_vertex_colors: None,
        fbx_preset: None,
        export_orientation: None,
        animate_in_place: None,
    });
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn convert_model_fbx_with_preset() {
    let req = TaskRequest::ConvertModel(ConvertModelRequest {
        original_model_task_id: "src-task-1".into(),
        format: OutputFormat::Fbx,
        fbx_preset: Some(FbxPreset::Mixamo),
        part_names: Some(vec!["head".into(), "body".into()]),
        with_animation: Some(true),
        quad: None,
        force_symmetry: None,
        face_limit: None,
        flatten_bottom: None,
        flatten_bottom_threshold: None,
        texture_size: None,
        texture_format: None,
        scale_factor: None,
        pivot_to_center_bottom: None,
        pack_uv: None,
        bake: None,
        export_vertex_colors: None,
        export_orientation: None,
        animate_in_place: None,
    });
    insta::assert_json_snapshot!(json_of(&req));
}
