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

use tripo_api::{PostStyle, StylizeModelRequest};

#[test]
fn stylize_model_voxel() {
    let req = TaskRequest::Stylize(StylizeModelRequest {
        original_model_task_id: "src-task".into(),
        style: PostStyle::Voxel,
        block_size: Some(80),
    });
    insta::assert_json_snapshot!(json_of(&req));
}

use tripo_api::{TextureModelRequest, TexturePrompt};

#[test]
fn texture_model_no_prompt() {
    let req = TaskRequest::TextureModel(TextureModelRequest {
        original_model_task_id: "src".into(),
        ..Default::default()
    });
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn texture_model_with_text_and_style_image() {
    let req = TaskRequest::TextureModel(TextureModelRequest {
        original_model_task_id: "src".into(),
        texture_prompt: TexturePrompt {
            text: Some("brass and copper".into()),
            image: None,
            style_image: Some(ImageInput::Url("https://cdn/s.jpg".parse().unwrap())),
        },
        texture: Some(true),
        pbr: Some(true),
        ..Default::default()
    });
    insta::assert_json_snapshot!(json_of(&req));
}

use tripo_api::{CheckRiggableRequest, RefineModelRequest};

#[test]
fn refine_model() {
    let req = TaskRequest::Refine(RefineModelRequest {
        draft_model_task_id: "draft-1".into(),
    });
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "draft_model_task_id": "draft-1",
      "type": "refine_model"
    }
    "###);
}

#[test]
fn check_riggable_uses_rename() {
    let req = TaskRequest::CheckRiggable(CheckRiggableRequest {
        original_model_task_id: "src".into(),
    });
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "original_model_task_id": "src",
      "type": "animate_prerigcheck"
    }
    "###);
}

use tripo_api::{RigModelRequest, RigOutputFormat, RigSpec, RigType};

#[test]
fn rig_model_with_spec() {
    let req = TaskRequest::Rig(RigModelRequest {
        original_model_task_id: "src".into(),
        model_version: Some("v2.0-20250506".into()),
        out_format: Some(RigOutputFormat::Fbx),
        rig_type: Some(RigType::Quadruped),
        spec: Some(RigSpec::Mixamo),
    });
    insta::assert_json_snapshot!(json_of(&req));
}

use tripo_api::{Animation, RetargetAnimationRequest};

#[test]
fn retarget_single_animation() {
    let req = TaskRequest::Retarget(RetargetAnimationRequest::single(
        "src",
        Animation::Walk,
    ));
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "animation": "preset:walk",
      "original_model_task_id": "src",
      "type": "animate_retarget"
    }
    "###);
}

#[test]
fn retarget_multi_animation() {
    let req = TaskRequest::Retarget(RetargetAnimationRequest::many(
        "src",
        vec![Animation::Walk, Animation::Run],
    ));
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "animations": [
        "preset:walk",
        "preset:run"
      ],
      "original_model_task_id": "src",
      "type": "animate_retarget"
    }
    "###);
}

use tripo_api::{MeshCompletionRequest, MeshSegmentationRequest};

#[test]
fn mesh_segmentation_minimal() {
    let req = TaskRequest::MeshSegmentation(MeshSegmentationRequest {
        original_model_task_id: "src".into(),
        model_version: None,
    });
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "original_model_task_id": "src",
      "type": "mesh_segmentation"
    }
    "###);
}

#[test]
fn mesh_completion_with_parts() {
    let req = TaskRequest::MeshCompletion(MeshCompletionRequest {
        original_model_task_id: "src".into(),
        model_version: Some("v1.0-20250506".into()),
        part_names: Some(vec!["head".into()]),
    });
    insta::assert_json_snapshot!(json_of(&req));
}
