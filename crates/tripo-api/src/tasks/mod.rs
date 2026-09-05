//! Per-variant task request structs and the top-level `TaskRequest` dispatch enum.

use serde::Serialize;

use crate::client::Client;
use crate::enums::GeometryQuality;
use crate::error::{Error, Result};
use crate::image::ImageInput;
use crate::versions;

pub mod check_riggable;
pub mod convert_model;
pub mod image_to_model;
pub mod mesh_completion;
pub mod mesh_decimate;
pub mod mesh_segmentation;
pub mod multiview_to_model;
pub mod refine_model;
pub mod retarget_animation;
pub mod rig_model;
pub mod stylize_model;
pub mod text_to_model;
pub mod texture_model;

pub use check_riggable::CheckRiggableRequest;
pub use convert_model::ConvertModelRequest;
pub use image_to_model::ImageToModelRequest;
pub use mesh_completion::MeshCompletionRequest;
pub use mesh_decimate::MeshDecimateRequest;
pub use mesh_segmentation::MeshSegmentationRequest;
pub use multiview_to_model::MultiviewToModelRequest;
pub use refine_model::RefineModelRequest;
pub use retarget_animation::{AnimationInput, RetargetAnimationRequest};
pub use rig_model::RigModelRequest;
pub use stylize_model::StylizeModelRequest;
pub use text_to_model::TextToModelRequest;
pub use texture_model::{TextureModelRequest, TexturePrompt};

/// Task creation request body. Each variant maps to a dedicated v3 endpoint
/// (see [`TaskRequest::endpoint`]); the body is the inner struct serialized
/// as-is — v3 has no `type` discriminator field.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum TaskRequest {
    /// `POST /generation/text-to-model` — generate a 3D model from a text prompt.
    TextToModel(TextToModelRequest),
    /// `POST /generation/image-to-model` — generate a 3D model from a single image.
    ImageToModel(ImageToModelRequest),
    /// `POST /generation/multiview-to-model` — generate from multiple images (front/back/left/right).
    MultiviewToModel(MultiviewToModelRequest),
    /// `POST /models/convert` — convert a completed model to another file format.
    ConvertModel(ConvertModelRequest),
    /// `POST /models/stylize` — apply a stylization preset (lego/voxel/etc).
    Stylize(StylizeModelRequest),
    /// `POST /models/texture` — (re)texture an existing model.
    TextureModel(TextureModelRequest),
    /// `POST /models/refine` — turn a draft model into a finished one.
    Refine(RefineModelRequest),
    /// `POST /animations/rig-check` — precheck whether a model can be rigged.
    CheckRiggable(CheckRiggableRequest),
    /// `POST /animations/rig` — generate a skeletal rig for an existing model.
    Rig(RigModelRequest),
    /// `POST /animations/retarget` — retarget animations onto a rigged model.
    Retarget(RetargetAnimationRequest),
    /// `POST /mesh/segment` — decompose a model into semantic parts.
    MeshSegmentation(MeshSegmentationRequest),
    /// `POST /mesh/complete` — fill holes in an existing mesh.
    MeshCompletion(MeshCompletionRequest),
    /// `POST /mesh/decimate` — retopology: reduce polycount (smart v2.0 or basic v1.0).
    MeshDecimate(MeshDecimateRequest),
}

impl TaskRequest {
    /// The v3 endpoint path for this variant, relative to the base URL.
    #[must_use]
    pub fn endpoint(&self) -> &'static str {
        match self {
            Self::TextToModel(_) => "generation/text-to-model",
            Self::ImageToModel(_) => "generation/image-to-model",
            Self::MultiviewToModel(_) => "generation/multiview-to-model",
            Self::ConvertModel(_) => "models/convert",
            Self::Stylize(_) => "models/stylize",
            Self::TextureModel(_) => "models/texture",
            Self::Refine(_) => "models/refine",
            Self::CheckRiggable(_) => "animations/rig-check",
            Self::Rig(_) => "animations/rig",
            Self::Retarget(_) => "animations/retarget",
            Self::MeshSegmentation(_) => "mesh/segment",
            Self::MeshCompletion(_) => "mesh/complete",
            Self::MeshDecimate(_) => "mesh/decimate",
        }
    }

    /// Client-side request validation. Dispatches to per-variant `validate()`.
    /// Called from `Client::create_task` before the POST so bad requests cost
    /// nothing and produce a usable error message.
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Rig(r) => r.validate(),
            Self::TextToModel(r) => r.validate(),
            Self::ImageToModel(r) => r.validate(),
            Self::MultiviewToModel(r) => r.validate(),
            _ => Ok(()),
        }
    }

    /// Walk the request, uploading any `ImageInput::Path` entries to `file_token`s.
    /// Call this before serializing & sending.
    pub async fn upload_images(&mut self, client: &Client) -> Result<()> {
        match self {
            Self::ImageToModel(r) => upload_image_if_path(client, &mut r.input).await,
            Self::MultiviewToModel(r) => {
                let futs = r
                    .inputs
                    .iter_mut()
                    .flatten()
                    .map(|img| upload_image_if_path(client, img));
                futures::future::try_join_all(futs).await?;
                Ok(())
            }
            Self::TextureModel(r) => {
                let image = &mut r.texture_prompt.image;
                let style = &mut r.texture_prompt.style_image;
                match (image.as_mut(), style.as_mut()) {
                    (Some(a), Some(b)) => {
                        tokio::try_join!(
                            upload_image_if_path(client, a),
                            upload_image_if_path(client, b)
                        )?;
                    }
                    (Some(a), None) => upload_image_if_path(client, a).await?,
                    (None, Some(b)) => upload_image_if_path(client, b).await?,
                    (None, None) => {}
                }
                Ok(())
            }
            Self::TextToModel(_)
            | Self::ConvertModel(_)
            | Self::Stylize(_)
            | Self::Refine(_)
            | Self::CheckRiggable(_)
            | Self::Rig(_)
            | Self::Retarget(_)
            | Self::MeshSegmentation(_)
            | Self::MeshCompletion(_)
            | Self::MeshDecimate(_) => Ok(()),
        }
    }
}

/// Reject parameters that aren't supported by `model: P1-20260311`.
/// P1 is a low-poly-optimized pipeline and per the docs rejects `quad`,
/// `smart_low_poly`, `generate_parts`, and `geometry_quality`. Called from
/// text/image/multiview `validate()`.
pub(crate) fn validate_p1_params(
    model: Option<&str>,
    quad: Option<bool>,
    smart_low_poly: Option<bool>,
    generate_parts: Option<bool>,
    geometry_quality: Option<&GeometryQuality>,
) -> Result<()> {
    if !matches!(model, Some(versions::text_image::P1 | "tripo-p1")) {
        return Ok(());
    }
    let mut unsupported: Vec<&str> = Vec::new();
    if quad == Some(true) {
        unsupported.push("quad");
    }
    if smart_low_poly == Some(true) {
        unsupported.push("smart_low_poly");
    }
    if generate_parts == Some(true) {
        unsupported.push("generate_parts");
    }
    if geometry_quality.is_some() {
        unsupported.push("geometry_quality");
    }
    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(Error::InvalidRequest(format!(
            "model {} does not support: {}",
            versions::text_image::P1,
            unsupported.join(", "),
        )))
    }
}

/// Validate the documented P2 polycount ranges, leaving other models to the server.
pub(crate) fn validate_p2_face_limit(
    model: Option<&str>,
    quad: Option<bool>,
    face_limit: Option<i32>,
) -> Result<()> {
    if model != Some(versions::text_image::P2) {
        return Ok(());
    }
    let maximum = if quad == Some(true) { 25_000 } else { 50_000 };
    if let Some(limit) = face_limit
        && !(48..=maximum).contains(&limit)
    {
        return Err(Error::InvalidRequest(format!(
            "model {} requires face_limit between 48 and {maximum}; omit it for adaptive sizing",
            versions::text_image::P2,
        )));
    }
    Ok(())
}

/// Helper used by variants that consume one image: uploads if the variant is
/// `ImageInput::Path`, replacing it with `ImageInput::FileToken`.
pub(crate) async fn upload_image_if_path(client: &Client, img: &mut ImageInput) -> Result<()> {
    if let ImageInput::Path(p) = img {
        let up = client.upload_file(&*p).await?;
        *img = ImageInput::FileToken(up.file_token);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_p1_version_skips_p1_checks() {
        validate_p1_params(
            None,
            Some(true),
            Some(true),
            Some(true),
            Some(&GeometryQuality::Detailed),
        )
        .unwrap();
        validate_p1_params(
            Some(versions::text_image::V3_1),
            Some(true),
            Some(true),
            Some(true),
            Some(&GeometryQuality::Detailed),
        )
        .unwrap();
    }

    #[test]
    fn p1_with_no_unsupported_fields_ok() {
        validate_p1_params(Some(versions::text_image::P1), None, None, None, None).unwrap();
        validate_p1_params(
            Some(versions::text_image::P1),
            Some(false),
            Some(false),
            Some(false),
            None,
        )
        .unwrap();
    }

    #[test]
    fn p1_rejects_quad() {
        let err = validate_p1_params(Some(versions::text_image::P1), Some(true), None, None, None)
            .unwrap_err();
        assert!(matches!(err, Error::InvalidRequest(ref m) if m.contains("quad")));
    }

    #[test]
    fn p1_rejects_all_unsupported_together() {
        let err = validate_p1_params(
            Some(versions::text_image::P1),
            Some(true),
            Some(true),
            Some(true),
            Some(&GeometryQuality::Detailed),
        )
        .unwrap_err();
        let Error::InvalidRequest(msg) = err else {
            panic!("wrong variant");
        };
        for field in [
            "quad",
            "smart_low_poly",
            "generate_parts",
            "geometry_quality",
        ] {
            assert!(msg.contains(field), "missing {field} in {msg}");
        }
    }
}
