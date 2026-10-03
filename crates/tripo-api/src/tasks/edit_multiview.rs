//! `edit_multiview` task variant. Endpoint: `POST /generation/edit-multiview`.

use serde::{Deserialize, Serialize};

use crate::enums::MultiviewView;
use crate::error::{Error, Result};
use crate::image::ImageInput;

/// One edit instruction applied to a single view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MultiviewEdit {
    /// The change to make, e.g. `change the shirt color to red`.
    pub prompt: String,
    /// The view to edit.
    pub view: MultiviewView,
}

/// Request body for `POST /generation/edit-multiview`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct EditMultiviewRequest {
    /// Multiview image to edit: `task_id` of an image-to-multiview or
    /// edit-multiview task, `file_token`, URL, or local path. The API docs
    /// list all of these; the official SDKs report that the service currently
    /// accepts only a `task_id`.
    pub input: ImageInput,
    /// Edit instructions, at most one per view, so 1 to 4 entries.
    pub prompts: Vec<MultiviewEdit>,
}

impl EditMultiviewRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.prompts.is_empty() {
            return Err(Error::InvalidRequest(
                "edit-multiview needs at least one prompt".into(),
            ));
        }
        // The official SDKs cap this at one prompt per view (4); the docs give
        // no count, so only that bound is enforced.
        if self.prompts.len() > 4 {
            return Err(Error::InvalidRequest(format!(
                "edit-multiview accepts at most 4 prompts (one per view), got {}",
                self.prompts.len()
            )));
        }
        Ok(())
    }
}
