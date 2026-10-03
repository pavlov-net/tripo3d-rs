//! `edit-multiview` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::MultiviewView;
use tripo_api::{EditMultiviewRequest, ImageInput, MultiviewEdit, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Edit views of a multiview image. Give at least one of `--front`, `--left`,
/// `--back`, `--right`.
#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("edits").required(true).multiple(true))]
pub struct EditMultiviewArgs {
    /// Multiview source: `task_id` of an image-to-multiview or edit-multiview
    /// task, `file_token`, URL, or local path. The service currently accepts
    /// only a `task_id`.
    #[arg(long)]
    pub input: String,
    /// Edit instruction for the front view.
    #[arg(long, group = "edits")]
    pub front: Option<String>,
    /// Edit instruction for the left view.
    #[arg(long, group = "edits")]
    pub left: Option<String>,
    /// Edit instruction for the back view.
    #[arg(long, group = "edits")]
    pub back: Option<String>,
    /// Edit instruction for the right view.
    #[arg(long, group = "edits")]
    pub right: Option<String>,

    #[command(flatten)]
    pub run: VariantRunOpts,
}

impl VariantArgs for EditMultiviewArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        let prompts = [
            (MultiviewView::Front, self.front),
            (MultiviewView::Left, self.left),
            (MultiviewView::Back, self.back),
            (MultiviewView::Right, self.right),
        ]
        .into_iter()
        .filter_map(|(view, prompt)| {
            Some(MultiviewEdit {
                prompt: prompt?,
                view,
            })
        })
        .collect();
        Ok(TaskRequest::EditMultiview(EditMultiviewRequest {
            input: ImageInput::parse(&self.input),
            prompts,
        }))
    }
}
