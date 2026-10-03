//! `mesh-smart-segment` subcommand.

use anyhow::Result;
use clap::Args;
use tripo_api::enums::{SegGranularity, SegType};
use tripo_api::{ImageInput, MeshSmartSegmentRequest, TaskRequest};

use crate::commands::variants::{VariantArgs, VariantRunOpts};

/// Segment an image or GLB into parts, including auto modeling.
#[derive(Debug, Args)]
pub struct MeshSmartSegmentArgs {
    /// Pipeline (image|model). `model` takes GLB input and requires `--transform`.
    #[arg(long, value_parser = seg_type)]
    pub seg_type: SegType,
    /// URL, `file_token`, or local path.
    #[arg(long)]
    pub input: String,
    /// Segmentation granularity (coarse|medium|fine).
    #[arg(long, value_parser = granularity)]
    pub granularity: Option<SegGranularity>,
    /// Hint text describing the parts to segment.
    #[arg(long)]
    pub hint: Option<String>,
    /// Column-major 4x4 transform as 16 comma-separated numbers.
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    pub transform: Option<Vec<f64>>,
    #[command(flatten)]
    pub run: VariantRunOpts,
}

fn seg_type(s: &str) -> Result<SegType, String> {
    match s {
        "image" => Ok(SegType::Image),
        "model" => Ok(SegType::Model),
        o => Err(format!("invalid seg_type `{o}` — use image|model")),
    }
}

fn granularity(s: &str) -> Result<SegGranularity, String> {
    match s {
        "coarse" => Ok(SegGranularity::Coarse),
        "medium" => Ok(SegGranularity::Medium),
        "fine" => Ok(SegGranularity::Fine),
        o => Err(format!(
            "invalid granularity `{o}` — use coarse|medium|fine"
        )),
    }
}

impl VariantArgs for MeshSmartSegmentArgs {
    fn take_run_opts(&mut self) -> VariantRunOpts {
        std::mem::take(&mut self.run)
    }
    fn into_request(self) -> Result<TaskRequest> {
        let transform = self
            .transform
            .map(|t| {
                <[f64; 16]>::try_from(t).map_err(|t| {
                    tripo_api::Error::InvalidRequest(format!(
                        "--transform takes 16 numbers (column-major 4x4), got {}",
                        t.len()
                    ))
                })
            })
            .transpose()?;
        Ok(TaskRequest::MeshSmartSegment(MeshSmartSegmentRequest {
            seg_type: self.seg_type,
            input: ImageInput::parse(&self.input),
            granularity: self.granularity,
            hint: self.hint,
            transform,
        }))
    }
}
