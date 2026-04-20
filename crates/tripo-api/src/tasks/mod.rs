//! Per-variant task request structs and the top-level `TaskRequest` dispatch enum.

use std::future::Future;
use std::pin::Pin;

use serde::Serialize;

use crate::client::Client;
use crate::error::Result;
use crate::image::ImageInput;

pub mod image_to_model;
pub mod text_to_model;

pub use image_to_model::ImageToModelRequest;
pub use text_to_model::TextToModelRequest;

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
}

impl TaskRequest {
    /// Walk the request, uploading any `ImageInput::Path` entries to `file_token`s.
    /// Call this before serializing & sending.
    pub fn upload_images<'a>(
        &'a mut self,
        client: &'a Client,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                Self::TextToModel(_) => {
                    let _ = client;
                    Ok(())
                }
                Self::ImageToModel(r) => upload_image_if_path(client, &mut r.image).await,
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
pub(crate) async fn upload_images_if_paths(
    client: &Client,
    imgs: &mut [ImageInput],
) -> Result<()> {
    for img in imgs.iter_mut() {
        upload_image_if_path(client, img).await?;
    }
    Ok(())
}
