//! Per-variant task request structs and the top-level `TaskRequest` dispatch enum.

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::enums::Quality;
use crate::error::{Error, Result};
use crate::image::ImageInput;
use crate::versions;

pub mod check_riggable;
pub mod convert_model;
pub mod image_to_model;
pub mod mesh_completion;
pub mod mesh_segmentation;
pub mod multiview_to_model;
pub mod refine_model;
pub mod retarget_animation;
pub mod rig_model;
pub mod smart_lowpoly;
pub mod stylize_model;
pub mod text_to_model;
pub mod texture_model;

pub use check_riggable::CheckRiggableRequest;
pub use convert_model::ConvertModelRequest;
pub use image_to_model::ImageToModelRequest;
pub use mesh_completion::MeshCompletionRequest;
pub use mesh_segmentation::MeshSegmentationRequest;
pub use multiview_to_model::MultiviewToModelRequest;
pub use refine_model::RefineModelRequest;
pub use retarget_animation::{AnimationInput, RetargetAnimationRequest};
pub use rig_model::RigModelRequest;
pub use smart_lowpoly::SmartLowpolyRequest;
pub use stylize_model::StylizeModelRequest;
pub use text_to_model::TextToModelRequest;
pub use texture_model::{TextureModelRequest, TexturePrompt};

/// Task creation request body. `type` tag is set by serde.
///
/// Note: four variants have wire-level `type` strings that differ from the
/// Rust variant name — `#[serde(rename = "...")]` per variant handles this.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "type")]
#[allow(
    clippy::unsafe_derive_deserialize,
    reason = "transitive lint via nested types; this enum itself has no unsafe methods"
)]
pub enum TaskRequest {
    /// `text_to_model` — generate a 3D model from a text prompt.
    #[serde(rename = "text_to_model")]
    TextToModel(TextToModelRequest),
    /// `image_to_model` — generate a 3D model from a single image.
    #[serde(rename = "image_to_model")]
    ImageToModel(ImageToModelRequest),
    /// `multiview_to_model` — generate from multiple images (front/back/left/right).
    #[serde(rename = "multiview_to_model")]
    MultiviewToModel(MultiviewToModelRequest),
    /// `convert_model` — convert a completed model to another file format.
    #[serde(rename = "convert_model")]
    ConvertModel(ConvertModelRequest),
    /// `stylize_model` — apply a stylization preset (lego/voxel/etc).
    #[serde(rename = "stylize_model")]
    Stylize(StylizeModelRequest),
    /// `texture_model` — (re)texture an existing model.
    #[serde(rename = "texture_model")]
    TextureModel(TextureModelRequest),
    /// `refine_model` — turn a draft model into a finished one.
    #[serde(rename = "refine_model")]
    Refine(RefineModelRequest),
    /// `check_riggable` — precheck whether a model can be rigged.
    #[serde(rename = "animate_prerigcheck")]
    CheckRiggable(CheckRiggableRequest),
    /// `rig_model` — generate a skeletal rig for an existing model.
    #[serde(rename = "animate_rig")]
    Rig(RigModelRequest),
    /// `retarget_animation` — retarget animations onto a rigged model.
    #[serde(rename = "animate_retarget")]
    Retarget(RetargetAnimationRequest),
    /// `mesh_segmentation` — decompose a model into semantic parts.
    #[serde(rename = "mesh_segmentation")]
    MeshSegmentation(MeshSegmentationRequest),
    /// `mesh_completion` — fill holes in an existing mesh.
    #[serde(rename = "mesh_completion")]
    MeshCompletion(MeshCompletionRequest),
    /// `smart_lowpoly` — reduce a high-poly model to a lowpoly one. Wire: `highpoly_to_lowpoly`.
    #[serde(rename = "highpoly_to_lowpoly")]
    SmartLowpoly(SmartLowpolyRequest),
}

impl TaskRequest {
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
            Self::ImageToModel(r) => upload_image_if_path(client, &mut r.image).await,
            Self::MultiviewToModel(r) => {
                let futs = r
                    .images
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
            | Self::SmartLowpoly(_) => Ok(()),
        }
    }
}

/// Reject parameters that aren't supported by `model_version: P1-20260311`.
/// P1 is a low-poly-optimized pipeline and per the docs rejects `quad`,
/// `smart_low_poly`, `generate_parts`, and `geometry_quality`. Called from
/// text/image/multiview `validate()`.
pub(crate) fn validate_p1_params(
    model_version: Option<&str>,
    quad: Option<bool>,
    smart_low_poly: Option<bool>,
    generate_parts: Option<bool>,
    geometry_quality: Option<&Quality>,
) -> Result<()> {
    if model_version != Some(versions::text_image::P1) {
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
            "model_version {} does not support: {}",
            versions::text_image::P1,
            unsupported.join(", "),
        )))
    }
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
            Some(&Quality::Detailed),
        )
        .unwrap();
        validate_p1_params(
            Some(versions::text_image::V3_1),
            Some(true),
            Some(true),
            Some(true),
            Some(&Quality::Detailed),
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
            Some(&Quality::Detailed),
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
