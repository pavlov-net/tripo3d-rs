//! Snapshot tests for the serialized JSON body of every `TaskRequest` variant,
//! plus its endpoint path. These lock down byte-exact wire-format compatibility
//! with the Tripo v3 API.

use serde_json::Value;
use tripo_api::{
    CompressionMode, ConvertModelRequest, FbxPreset, ImageInput, ImageToModelRequest,
    MultiviewToModelRequest, OutputFormat, TextToModelRequest,
    enums::{GeometryQuality, TextureQuality},
    tasks::TaskRequest,
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
    assert_eq!(req.endpoint(), "generation/text-to-model");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "prompt": "a red robot"
    }
    "###);
}

#[test]
fn text_to_model_full() {
    let req = TaskRequest::TextToModel(TextToModelRequest {
        prompt: "a red robot".into(),
        negative_prompt: Some("low quality".into()),
        model: Some("v3.1-20260211".into()),
        texture_quality: Some(TextureQuality::Detailed),
        geometry_quality: Some(GeometryQuality::Standard),
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
        input: ImageInput::FileToken("file_abc123".into()),
        texture: Some(true),
        pbr: Some(false),
        quad: Some(true),
        ..default_image_to_model()
    });
    assert_eq!(req.endpoint(), "generation/image-to-model");
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn image_to_model_url() {
    let req = TaskRequest::ImageToModel(ImageToModelRequest {
        input: ImageInput::Url("https://example.com/x.jpg".parse().unwrap()),
        ..default_image_to_model()
    });
    insta::assert_json_snapshot!(json_of(&req));
}

fn default_image_to_model() -> ImageToModelRequest {
    ImageToModelRequest {
        input: ImageInput::FileToken("file_default".into()),
        model: None,
        enable_image_autofix: None,
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
        export_uv: None,
        export_orientation: None,
    }
}

#[test]
fn multiview_to_model_with_empty_slot() {
    let req = TaskRequest::MultiviewToModel(MultiviewToModelRequest {
        inputs: vec![
            Some(ImageInput::Url(
                "https://example.com/front.jpg".parse().unwrap(),
            )),
            None,
            Some(ImageInput::FileToken(
                "550e8400-e29b-41d4-a716-446655440000".into(),
            )),
        ],
        model: None,
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
        export_uv: None,
        export_orientation: None,
    });
    assert_eq!(req.endpoint(), "generation/multiview-to-model");
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn convert_model_minimal_gltf() {
    let req = TaskRequest::ConvertModel(ConvertModelRequest {
        input: "task_src1".into(),
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
    assert_eq!(req.endpoint(), "models/convert");
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn convert_model_fbx_with_preset() {
    let req = TaskRequest::ConvertModel(ConvertModelRequest {
        input: "task_src1".into(),
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
    // Legacy endpoint — keeps the v2 `original_model_task_id` field name.
    let req = TaskRequest::Stylize(StylizeModelRequest {
        original_model_task_id: "src-task".into(),
        style: PostStyle::Voxel,
        block_size: Some(80),
    });
    assert_eq!(req.endpoint(), "models/stylize");
    insta::assert_json_snapshot!(json_of(&req));
}

use tripo_api::{TextureModelRequest, TexturePrompt};

#[test]
fn texture_model_no_prompt() {
    let req = TaskRequest::TextureModel(TextureModelRequest {
        input: "task_src".into(),
        ..Default::default()
    });
    assert_eq!(req.endpoint(), "models/texture");
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn texture_model_with_text_and_style_image() {
    let req = TaskRequest::TextureModel(TextureModelRequest {
        input: "task_src".into(),
        texture_prompt: TexturePrompt {
            text: Some("brass and copper".into()),
            image: None,
            style_image: Some(ImageInput::Url("https://cdn/s.jpg".parse().unwrap())),
        },
        pbr: Some(true),
        ..Default::default()
    });
    insta::assert_json_snapshot!(json_of(&req));
}

use tripo_api::{CheckRiggableRequest, RefineModelRequest};

#[test]
fn refine_model() {
    // Legacy endpoint — keeps the v2 `draft_model_task_id` field name.
    let req = TaskRequest::Refine(RefineModelRequest {
        draft_model_task_id: "task_draft1".into(),
    });
    assert_eq!(req.endpoint(), "models/refine");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "draft_model_task_id": "task_draft1"
    }
    "###);
}

#[test]
fn check_riggable_body_and_endpoint() {
    let req = TaskRequest::CheckRiggable(CheckRiggableRequest {
        input: "task_src".into(),
    });
    assert_eq!(req.endpoint(), "animations/rig-check");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "input": "task_src"
    }
    "###);
}

use tripo_api::{RigModelRequest, RigOutputFormat, RigSpec, RigType};

#[test]
fn rig_model_with_spec() {
    let req = TaskRequest::Rig(RigModelRequest {
        input: "task_src".into(),
        model: Some("v2.5-20260210".into()),
        out_format: Some(RigOutputFormat::Fbx),
        rig_type: Some(RigType::Quadruped),
        spec: Some(RigSpec::Mixamo),
    });
    assert_eq!(req.endpoint(), "animations/rig");
    insta::assert_json_snapshot!(json_of(&req));
}

use tripo_api::{Animation, RetargetAnimationRequest};

#[test]
fn retarget_single_animation() {
    let req = TaskRequest::Retarget(RetargetAnimationRequest::single(
        "task_src",
        Animation::Walk,
    ));
    assert_eq!(req.endpoint(), "animations/retarget");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "animation": "preset:walk",
      "input": "task_src"
    }
    "###);
}

#[test]
fn retarget_multi_animation() {
    let req = TaskRequest::Retarget(RetargetAnimationRequest::many(
        "task_src",
        vec![Animation::Walk, Animation::Run],
    ));
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "animations": [
        "preset:walk",
        "preset:run"
      ],
      "input": "task_src"
    }
    "###);
}

use tripo_api::{MeshCompletionRequest, MeshSegmentationRequest};

#[test]
fn mesh_segmentation_minimal() {
    let req = TaskRequest::MeshSegmentation(MeshSegmentationRequest {
        input: "task_src".into(),
        model: None,
    });
    assert_eq!(req.endpoint(), "mesh/segment");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "input": "task_src"
    }
    "###);
}

#[test]
fn mesh_completion_with_parts() {
    let req = TaskRequest::MeshCompletion(MeshCompletionRequest {
        input: "task_src".into(),
        model: Some("v1.0-20250506".into()),
        part_names: Some(vec!["head".into()]),
    });
    assert_eq!(req.endpoint(), "mesh/complete");
    insta::assert_json_snapshot!(json_of(&req));
}

use tripo_api::MeshDecimateRequest;

#[test]
fn mesh_decimate_body_and_endpoint() {
    let req = TaskRequest::MeshDecimate(MeshDecimateRequest {
        input: "task_src".into(),
        quad: Some(true),
        face_limit: Some(2000),
        bake: Some(true),
        model: None,
        part_names: None,
    });
    assert_eq!(req.endpoint(), "mesh/decimate");
    insta::assert_json_snapshot!(json_of(&req));
}
