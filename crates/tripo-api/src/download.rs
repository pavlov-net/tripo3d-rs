//! Download output URLs from a `Task` into a directory.

use std::path::{Path, PathBuf};

use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::client::Client;
use crate::error::{Error, Result};
use crate::types::{Task, TaskId};

/// Which outputs to consider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    /// `output.model_url` — main mesh.
    Model,
    /// `output.rendered_image_url` — preview render.
    RenderedImage,
    /// `output.generated_image_url` — intermediate text-to-image result.
    GeneratedImage,
}

impl OutputKind {
    fn filename(self, id: &TaskId, ext: &str) -> String {
        match self {
            Self::Model => format!("{id}.{ext}"),
            Self::RenderedImage => format!("{id}_rendered.{ext}"),
            Self::GeneratedImage => format!("{id}_generated.{ext}"),
        }
    }
}

/// Knobs for `download_task_models`.
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    /// Max concurrent downloads (default 4).
    pub max_concurrency: usize,
    /// If true, overwrite existing files at target paths. If false, return `Error::FileExists`.
    pub overwrite: bool,
    /// Output kinds to include (default: all).
    pub kinds: Vec<OutputKind>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            max_concurrency: 4,
            overwrite: false,
            kinds: vec![
                OutputKind::Model,
                OutputKind::RenderedImage,
                OutputKind::GeneratedImage,
            ],
        }
    }
}

/// Paths of all successfully downloaded files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct DownloadedFiles {
    /// Main model path.
    pub model: Option<PathBuf>,
    /// Rendered preview image path.
    pub rendered_image: Option<PathBuf>,
    /// Intermediate generated image path.
    pub generated_image: Option<PathBuf>,
}

impl DownloadedFiles {
    /// Iterate over the paths that were actually written, in a stable order.
    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        [&self.model, &self.rendered_image, &self.generated_image]
            .into_iter()
            .flatten()
            .map(PathBuf::as_path)
    }
}

fn extension_of(url: &str, default_ext: &str) -> String {
    let path = url.split('?').next().unwrap_or(url);
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or(default_ext)
        .to_string()
}

impl Client {
    /// Download all available outputs of a task into `dir`. Creates `dir` if
    /// it does not exist. Atomic writes via `.partial` + rename.
    #[tracing::instrument(skip(self, task, opts), fields(task_id = %task.task_id))]
    pub async fn download_task_models(
        &self,
        task: &Task,
        dir: &Path,
        opts: DownloadOptions,
    ) -> Result<DownloadedFiles> {
        tokio::fs::create_dir_all(dir).await?;

        let mut jobs: Vec<(OutputKind, String, PathBuf)> = Vec::new();
        for kind in &opts.kinds {
            let (url, default_ext) = match kind {
                OutputKind::Model => (&task.output.model_url, "glb"),
                OutputKind::RenderedImage => (&task.output.rendered_image_url, "jpg"),
                OutputKind::GeneratedImage => (&task.output.generated_image_url, "jpg"),
            };
            let Some(url) = url.clone() else { continue };
            let ext = extension_of(&url, default_ext);
            let target = dir.join(kind.filename(&task.task_id, &ext));
            if !opts.overwrite && tokio::fs::try_exists(&target).await? {
                return Err(Error::FileExists(target));
            }
            jobs.push((*kind, url, target));
        }

        let max = opts.max_concurrency.max(1);
        let mut in_flight = FuturesUnordered::new();
        let mut pending = jobs.into_iter();

        let mut out = DownloadedFiles::default();
        for _ in 0..max {
            if let Some(job) = pending.next() {
                in_flight.push(download_one(self, job));
            }
        }
        while let Some(done) = in_flight.next().await {
            let (kind, path) = done?;
            match kind {
                OutputKind::Model => out.model = Some(path),
                OutputKind::RenderedImage => out.rendered_image = Some(path),
                OutputKind::GeneratedImage => out.generated_image = Some(path),
            }
            if let Some(job) = pending.next() {
                in_flight.push(download_one(self, job));
            }
        }
        Ok(out)
    }
}

async fn download_one(
    client: &Client,
    (kind, url, target): (OutputKind, String, PathBuf),
) -> Result<(OutputKind, PathBuf)> {
    let mut partial = target.clone();
    partial.as_mut_os_string().push(".partial");
    let mut resp = client.http.get(&url).send().await?.error_for_status()?;
    let mut f = tokio::fs::File::create(&partial).await?;
    while let Some(chunk) = resp.chunk().await? {
        f.write_all(&chunk).await?;
    }
    f.flush().await?;
    drop(f);
    tokio::fs::rename(&partial, &target).await?;
    Ok((kind, target))
}
