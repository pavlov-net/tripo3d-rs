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
        texture_version: None,
        delight: None,
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
        texture_version: None,
        delight: None,
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
            images: None,
            style_image: Some(ImageInput::Url("https://cdn/s.jpg".parse().unwrap())),
        },
        pbr: Some(true),
        ..Default::default()
    });
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn texture_model_v3_5_fast_delight() {
    let req = TaskRequest::TextureModel(TextureModelRequest {
        input: "task_src".into(),
        model: Some(tripo_api::versions::texture::V3_5.into()),
        texture_quality: Some(TextureQuality::Fast),
        delight: Some(false),
        pbr: Some(true),
        ..Default::default()
    });
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn texture_model_with_multiview_images() {
    let req = TaskRequest::TextureModel(TextureModelRequest {
        input: "task_src".into(),
        model: Some(tripo_api::versions::texture::V3_5.into()),
        texture_prompt: TexturePrompt {
            images: Some(vec![
                ImageInput::Url("https://cdn/front.jpg".parse().unwrap()),
                ImageInput::FileToken("file_left".into()),
                ImageInput::Url("https://cdn/back.jpg".parse().unwrap()),
                ImageInput::FileToken("file_right".into()),
            ]),
            ..Default::default()
        },
        ..Default::default()
    });
    req.validate().unwrap();
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn text_to_model_texture_version() {
    let req = TaskRequest::TextToModel(TextToModelRequest {
        prompt: "a red robot".into(),
        texture_quality: Some(TextureQuality::Fast),
        texture_version: Some(tripo_api::versions::texture::V3_5.into()),
        delight: Some(true),
        ..Default::default()
    });
    req.validate().unwrap();
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

use tripo_api::{
    EditMultiviewRequest, ImageBackground, ImageOutputFormat, ImageQuality, ImageToImageRequest,
    ImageToImageTemplate, ImageToMultiviewRequest, MultiviewEdit, MultiviewView,
    TextToImageRequest, TextToImageTemplate, versions,
};

#[test]
fn text_to_image_minimal() {
    let req = TaskRequest::TextToImage(TextToImageRequest {
        prompt: "a glass sneaker".into(),
        ..Default::default()
    });
    assert_eq!(req.endpoint(), "generation/text-to-image");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "prompt": "a glass sneaker"
    }
    "###);
}

#[test]
fn text_to_image_seedream_full() {
    let req = TaskRequest::TextToImage(TextToImageRequest {
        prompt: "a knight".into(),
        model: Some(versions::image::SEEDREAM_V5.into()),
        size: Some("2K".into()),
        aspect_ratio: Some("3:2".into()),
        output_format: Some(ImageOutputFormat::Jpeg),
        watermark: Some(false),
        template: Some(TextToImageTemplate::AssetExtraction),
        ..Default::default()
    });
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "aspect_ratio": "3:2",
      "model": "seedream_v5",
      "output_format": "jpeg",
      "prompt": "a knight",
      "size": "2K",
      "template": "asset_extraction",
      "watermark": false
    }
    "###);
}

#[test]
fn text_to_image_chat_image_2_5_quality_background() {
    let req = TaskRequest::TextToImage(TextToImageRequest {
        prompt: "a game icon".into(),
        model: Some(versions::image::CHAT_IMAGE_2_5_FLARE.into()),
        size: Some("1536x1024".into()),
        quality: Some(ImageQuality::Xhigh),
        background: Some(ImageBackground::Transparent),
        output_format: Some(ImageOutputFormat::Png),
        ..Default::default()
    });
    req.validate().unwrap();
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "background": "transparent",
      "model": "chat_image_2.5_flare",
      "output_format": "png",
      "prompt": "a game icon",
      "quality": "xhigh",
      "size": "1536x1024"
    }
    "###);
}

#[test]
fn image_to_image_single_input() {
    let req = TaskRequest::ImageToImage(ImageToImageRequest {
        input: Some(ImageInput::Url(
            "https://example.com/reference.png".parse().unwrap(),
        )),
        prompt: Some("make the outfit glass".into()),
        model: Some(versions::image::CHAT_IMAGE_2_5_SUNBURST.into()),
        quality: Some(ImageQuality::Max),
        background: Some(ImageBackground::Opaque),
        output_format: Some(ImageOutputFormat::Jpeg),
        ..Default::default()
    });
    assert_eq!(req.endpoint(), "generation/image-to-image");
    req.validate().unwrap();
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "background": "opaque",
      "input": "https://example.com/reference.png",
      "model": "chat_image_2.5_sunburst",
      "output_format": "jpeg",
      "prompt": "make the outfit glass",
      "quality": "max"
    }
    "###);
}

#[test]
fn image_to_image_multi_input_template() {
    let req = TaskRequest::ImageToImage(ImageToImageRequest {
        inputs: Some(vec![
            ImageInput::FileToken("file_a".into()),
            ImageInput::FileToken("task_b".into()),
        ]),
        template: Some(ImageToImageTemplate::Enhance3d),
        size: Some("2048x2048".into()),
        aspect_ratio: Some("1:1".into()),
        ..Default::default()
    });
    req.validate().unwrap();
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "aspect_ratio": "1:1",
      "inputs": [
        "file_a",
        "task_b"
      ],
      "size": "2048x2048",
      "template": "3d_enhance"
    }
    "###);
}

#[test]
fn image_to_multiview_body_and_endpoint() {
    let req = TaskRequest::ImageToMultiview(ImageToMultiviewRequest {
        input: ImageInput::Url("https://example.com/character.png".parse().unwrap()),
    });
    assert_eq!(req.endpoint(), "generation/image-to-multiview");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "input": "https://example.com/character.png"
    }
    "###);
}

#[test]
fn edit_multiview_body_and_endpoint() {
    let req = TaskRequest::EditMultiview(EditMultiviewRequest {
        input: ImageInput::FileToken("task_abc123".into()),
        prompts: vec![
            MultiviewEdit {
                prompt: "change the shirt color to red".into(),
                view: MultiviewView::Front,
            },
            MultiviewEdit {
                prompt: "add a logo on the back".into(),
                view: MultiviewView::Back,
            },
        ],
    });
    assert_eq!(req.endpoint(), "generation/edit-multiview");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "input": "task_abc123",
      "prompts": [
        {
          "prompt": "change the shirt color to red",
          "view": "front"
        },
        {
          "prompt": "add a logo on the back",
          "view": "back"
        }
      ]
    }
    "###);
}

use tripo_api::{
    ImageToSplatRequest, ImportModelRequest, MeshSmartSegmentRequest, SegGranularity, SegType,
};

#[test]
fn import_model_url() {
    let req = TaskRequest::ImportModel(ImportModelRequest {
        input: ImageInput::parse("https://example.com/my-model.glb"),
    });
    assert_eq!(req.endpoint(), "models/import");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "input": "https://example.com/my-model.glb"
    }
    "###);
}

#[test]
fn image_to_splat_with_seed() {
    let req = TaskRequest::ImageToSplat(ImageToSplatRequest {
        input: ImageInput::FileToken("file_abc123".into()),
        model_seed: Some(42),
    });
    assert_eq!(req.endpoint(), "generation/image-to-splat");
    insta::assert_json_snapshot!(json_of(&req), @r###"
    {
      "input": "file_abc123",
      "model_seed": 42
    }
    "###);
}

#[test]
fn mesh_smart_segment_image() {
    let req = TaskRequest::MeshSmartSegment(MeshSmartSegmentRequest {
        seg_type: SegType::Image,
        input: ImageInput::FileToken("file_a1b2c3d4".into()),
        granularity: Some(SegGranularity::Medium),
        hint: Some("game character with sword and armor".into()),
        transform: None,
    });
    assert_eq!(req.endpoint(), "mesh/smartsegment");
    insta::assert_json_snapshot!(json_of(&req));
}

#[test]
fn mesh_smart_segment_model_with_transform() {
    let req = TaskRequest::MeshSmartSegment(MeshSmartSegmentRequest {
        seg_type: SegType::Model,
        input: ImageInput::parse("https://example.com/character.glb"),
        granularity: Some(SegGranularity::Fine),
        hint: Some("character body parts".into()),
        transform: Some([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]),
    });
    insta::assert_json_snapshot!(json_of(&req));
}
