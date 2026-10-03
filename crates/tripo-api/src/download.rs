//! Download output URLs from a `Task` into a directory.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::client::Client;
use crate::enums::MultiviewView;
use crate::error::{Error, Result};
use crate::types::Task;

/// Which outputs to consider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    /// `output.model_url` — main mesh (a `.splat` for `image_to_splat`).
    Model,
    /// `output.rendered_image_url` — preview render.
    RenderedImage,
    /// `output.generated_image_url` — text/image-to-image result, or the
    /// intermediate image of text-to-model.
    GeneratedImage,
    /// `output.<view>_view_url` — one image-to-multiview view.
    View(MultiviewView),
    /// `output.seg_model_url` — segmented mesh (`mesh/smartsegment`).
    SegModel,
    /// `output.mask_url` — segmentation mask (`mesh/smartsegment`).
    Mask,
}

impl OutputKind {
    /// Every output kind, in download-result order.
    const ALL: [Self; 9] = [
        Self::Model,
        Self::RenderedImage,
        Self::GeneratedImage,
        Self::View(MultiviewView::Front),
        Self::View(MultiviewView::Left),
        Self::View(MultiviewView::Back),
        Self::View(MultiviewView::Right),
        Self::SegModel,
        Self::Mask,
    ];

    /// The task output URL for this kind, the file extension to use when the
    /// URL has none, and the filename suffix after `<task_id>_` (`None` for
    /// the bare `<task_id>.<ext>`).
    fn source(self, task: &Task) -> (Option<&str>, &'static str, Option<&'static str>) {
        let out = &task.output;
        match self {
            Self::Model if task.task_type == "image_to_splat" => {
                (out.model_url.as_deref(), "splat", None)
            }
            Self::Model => (out.model_url.as_deref(), "glb", None),
            Self::RenderedImage => (out.rendered_image_url.as_deref(), "jpg", Some("rendered")),
            Self::GeneratedImage => (out.generated_image_url.as_deref(), "png", Some("generated")),
            Self::View(view) => (out.views.get(view), "jpg", Some(view.as_str())),
            Self::SegModel => (out.seg_model_url.as_deref(), "glb", Some("seg")),
            Self::Mask => (out.mask_url.as_deref(), "png", Some("mask")),
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
            kinds: OutputKind::ALL.to_vec(),
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
    /// Generated image path (image generation, or text-to-model intermediate).
    pub generated_image: Option<PathBuf>,
    /// Image-to-multiview view paths, keyed by view.
    #[serde(default)]
    pub views: BTreeMap<MultiviewView, PathBuf>,
    /// Segmented model path (`mesh/smartsegment`).
    pub seg_model: Option<PathBuf>,
    /// Segmentation mask image path (`mesh/smartsegment`).
    pub mask: Option<PathBuf>,
}

impl DownloadedFiles {
    /// Iterate over the paths that were actually written, in a stable order.
    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        OutputKind::ALL
            .into_iter()
            .filter_map(|kind| self.get(kind))
    }

    fn get(&self, kind: OutputKind) -> Option<&Path> {
        match kind {
            OutputKind::Model => self.model.as_deref(),
            OutputKind::RenderedImage => self.rendered_image.as_deref(),
            OutputKind::GeneratedImage => self.generated_image.as_deref(),
            OutputKind::View(view) => self.views.get(&view).map(PathBuf::as_path),
            OutputKind::SegModel => self.seg_model.as_deref(),
            OutputKind::Mask => self.mask.as_deref(),
        }
    }

    fn set(&mut self, kind: OutputKind, path: PathBuf) {
        match kind {
            OutputKind::Model => self.model = Some(path),
            OutputKind::RenderedImage => self.rendered_image = Some(path),
            OutputKind::GeneratedImage => self.generated_image = Some(path),
            OutputKind::View(view) => {
                self.views.insert(view, path);
            }
            OutputKind::SegModel => self.seg_model = Some(path),
            OutputKind::Mask => self.mask = Some(path),
        }
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
    /// it does not exist. Each file is written to `<name>.partial` and renamed
    /// into place; the `.partial` file is removed if the download fails or the
    /// returned future is dropped.
    #[tracing::instrument(skip(self, task, opts), fields(task_id = %task.task_id))]
    pub async fn download_task_models(
        &self,
        task: &Task,
        dir: &Path,
        opts: DownloadOptions,
    ) -> Result<DownloadedFiles> {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(Error::file(dir))?;

        let mut jobs: Vec<(OutputKind, String, PathBuf)> = Vec::new();
        for kind in &opts.kinds {
            let (Some(url), default_ext, suffix) = kind.source(task) else {
                continue;
            };
            let ext = extension_of(url, default_ext);
            let id = &task.task_id;
            let target = dir.join(match suffix {
                Some(suffix) => format!("{id}_{suffix}.{ext}"),
                None => format!("{id}.{ext}"),
            });
            if !opts.overwrite
                && tokio::fs::try_exists(&target)
                    .await
                    .map_err(Error::file(&target))?
            {
                return Err(Error::FileExists(target));
            }
            jobs.push((*kind, url.to_owned(), target));
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
            out.set(kind, path);
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
    let mut path = target.clone();
    path.as_mut_os_string().push(".partial");
    // Output URLs are signed storage/CDN URLs: fetch them without the API key
    // and without the API client's overall timeout.
    let mut resp = client.storage.get(&url).send().await?.error_for_status()?;
    // Declared before `f` so the file is closed before the guard removes it.
    let mut partial = PartialFile {
        path,
        renamed: false,
    };
    let mut f = tokio::fs::File::create(&partial.path)
        .await
        .map_err(Error::file(&partial.path))?;
    while let Some(chunk) = resp.chunk().await? {
        f.write_all(&chunk)
            .await
            .map_err(Error::file(&partial.path))?;
    }
    f.flush().await.map_err(Error::file(&partial.path))?;
    drop(f);
    tokio::fs::rename(&partial.path, &target)
        .await
        .map_err(Error::file(&target))?;
    partial.renamed = true;
    Ok((kind, target))
}

/// A `.partial` download file, removed on drop unless it was renamed into
/// place. Dropping covers both errors and cancellation of the download future.
struct PartialFile {
    path: PathBuf,
    renamed: bool,
}

impl Drop for PartialFile {
    fn drop(&mut self) {
        if !self.renamed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(task_type: &str, model_url: &str) -> Task {
        serde_json::from_value(serde_json::json!({
            "task_id": "t", "type": task_type, "status": "success",
            "output": {"model_url": model_url},
        }))
        .unwrap()
    }

    fn model_ext(t: &Task) -> String {
        let (url, default_ext, _) = OutputKind::Model.source(t);
        extension_of(url.unwrap(), default_ext)
    }

    #[test]
    fn extension_from_url_path_ignores_query() {
        assert_eq!(
            extension_of("https://cdn/x/scene.splat?sig=1", "glb"),
            "splat"
        );
        assert_eq!(extension_of("https://cdn/x/model.fbx", "glb"), "fbx");
    }

    #[test]
    fn splat_task_defaults_to_splat_extension() {
        assert_eq!(
            model_ext(&task("image_to_splat", "https://cdn/out")),
            "splat"
        );
        assert_eq!(model_ext(&task("text_to_model", "https://cdn/out")), "glb");
    }
}
