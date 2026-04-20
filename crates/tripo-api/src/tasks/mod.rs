//! Per-variant task request structs and the top-level `TaskRequest` dispatch enum.

use std::future::Future;
use std::pin::Pin;

use serde::Serialize;

use crate::client::Client;
use crate::error::Result;
use crate::image::ImageInput;

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
pub use retarget_animation::RetargetAnimationRequest;
pub use rig_model::RigModelRequest;
pub use smart_lowpoly::SmartLowpolyRequest;
pub use stylize_model::StylizeModelRequest;
pub use text_to_model::TextToModelRequest;
pub use texture_model::{TextureModelRequest, TexturePrompt};

/// Task creation request body. `type` tag is set by serde.
///
/// Note: four variants have wire-level `type` strings that differ from the
/// Rust variant name — `#[serde(rename = "...")]` per variant handles this.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "type")]
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
    /// Walk the request, uploading any `ImageInput::Path` entries to `file_token`s.
    /// Call this before serializing & sending.
    pub fn upload_images<'a>(
        &'a mut self,
        client: &'a Client,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            #[allow(clippy::match_same_arms)]
            match self {
                Self::TextToModel(_) => {
                    let _ = client;
                    Ok(())
                }
                Self::ImageToModel(r) => upload_image_if_path(client, &mut r.image).await,
                Self::MultiviewToModel(r) => {
                    for slot in r.images.iter_mut().flatten() {
                        upload_image_if_path(client, slot).await?;
                    }
                    Ok(())
                }
                Self::ConvertModel(_) => Ok(()),
                Self::Stylize(_) => Ok(()),
                Self::TextureModel(r) => {
                    if let Some(img) = r.texture_prompt.image.as_mut() {
                        upload_image_if_path(client, img).await?;
                    }
                    if let Some(img) = r.texture_prompt.style_image.as_mut() {
                        upload_image_if_path(client, img).await?;
                    }
                    Ok(())
                }
                Self::Refine(_)
                | Self::CheckRiggable(_)
                | Self::Rig(_)
                | Self::Retarget(_)
                | Self::MeshSegmentation(_)
                | Self::MeshCompletion(_)
                | Self::SmartLowpoly(_) => Ok(()),
            }
        })
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

/// Helper for multi-image variants.
// Used by multi-image variants added in Tasks 16+.
#[allow(dead_code)]
pub(crate) async fn upload_images_if_paths(client: &Client, imgs: &mut [ImageInput]) -> Result<()> {
    for img in imgs.iter_mut() {
        upload_image_if_path(client, img).await?;
    }
    Ok(())
}
