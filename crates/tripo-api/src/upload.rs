//! File uploads: multipart `POST /files` for small files, and the presigned
//! flow (`POST /files/presign` + `PUT` to storage) for large ones.

use std::path::Path;

use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use serde::Deserialize;

use crate::client::Client;
use crate::envelope::{map_http_error, read_envelope};
use crate::error::{Error, Result};
use crate::types::{PresignedUpload, UploadedFile};

/// Size above which [`Client::upload_file`] switches to presigned upload by
/// default: 60 MiB, the size above which the API docs recommend it.
pub const DEFAULT_PRESIGN_THRESHOLD: u64 = 60 * 1024 * 1024;

/// File extensions accepted by `POST /files/presign`.
pub const PRESIGN_FORMATS: &[&str] = &[
    "jpeg", "jpg", "png", "webp", "bmp", "tiff", "glb", "gltf", "fbx", "obj", "stl", "3mf", "usdz",
];

#[derive(Deserialize)]
struct FileTokenBody {
    file_token: String,
}

/// Presign `format` for `path`: its extension, lowercased, if presign accepts it.
fn presign_format(path: &Path) -> Result<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if PRESIGN_FORMATS.contains(&ext.as_str()) {
        Ok(ext)
    } else {
        Err(Error::InvalidRequest(format!(
            "presigned upload needs a file extension in {{{}}}, got {}",
            PRESIGN_FORMATS.join(", "),
            path.display()
        )))
    }
}

impl Client {
    /// Upload a local file and return a token usable as `ImageInput::FileToken`
    /// or the `input` field of other API calls.
    ///
    /// Files up to the client's presign threshold (default
    /// [`DEFAULT_PRESIGN_THRESHOLD`]) go through multipart `POST /files`.
    /// Larger files go through [`Client::upload_file_presigned`], which
    /// requires an extension listed in [`PRESIGN_FORMATS`].
    #[tracing::instrument(skip(self), fields(path = %path.as_ref().display()))]
    pub async fn upload_file(&self, path: impl AsRef<Path>) -> Result<UploadedFile> {
        let path = path.as_ref();
        let len = tokio::fs::metadata(path).await?.len();
        if len > self.presign_threshold {
            return self.upload_file_presigned(path).await;
        }
        self.upload_file_multipart(path).await
    }

    async fn upload_file_multipart(&self, path: &Path) -> Result<UploadedFile> {
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "non-utf8 filename",
                ))
            })?
            .to_string();
        let bytes = tokio::fs::read(path).await?;
        let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);
        let form = reqwest::multipart::Form::new().part("file", part);

        let url = self.url(&["files"]);
        let resp = self.http.post(url).multipart(form).send().await?;
        let data: FileTokenBody = read_envelope(resp).await?;
        Ok(UploadedFile {
            file_token: data.file_token,
        })
    }

    /// Upload a local file via presigned URL regardless of its size: presign
    /// with the file's extension as `format`, then stream the file to storage.
    #[tracing::instrument(skip(self), fields(path = %path.as_ref().display()))]
    pub async fn upload_file_presigned(&self, path: impl AsRef<Path>) -> Result<UploadedFile> {
        let path = path.as_ref();
        let format = presign_format(path)?;
        let presigned = self.presign_upload(&format).await?;
        self.put_presigned(&presigned, path).await
    }

    /// `POST /files/presign` — get a presigned storage URL and file token for
    /// a file with extension `format` (with or without the leading dot).
    #[tracing::instrument(skip(self))]
    pub async fn presign_upload(&self, format: &str) -> Result<PresignedUpload> {
        let url = self.url(&["files", "presign"]);
        let body = serde_json::json!({ "format": format.trim_start_matches('.') });
        let resp = self
            .send_with_retry(|| self.http.post(url.clone()).json(&body))
            .await?;
        read_envelope(resp).await
    }

    /// `PUT` the file at `path` to `presigned.presigned_url` and return its
    /// file token. The body is streamed from disk as
    /// `application/octet-stream`, with no `Authorization` header. The `PUT`
    /// is idempotent, so it follows the client's retry policy, reopening the
    /// file for each attempt.
    #[tracing::instrument(skip(self, presigned), fields(path = %path.as_ref().display()))]
    pub async fn put_presigned(
        &self,
        presigned: &PresignedUpload,
        path: impl AsRef<Path>,
    ) -> Result<UploadedFile> {
        let path = path.as_ref();
        let url = presigned.presigned_url.as_str();
        let resp = self
            .send_with_retry_async(|| async move {
                let file = tokio::fs::File::open(path).await?;
                // Storage rejects chunked transfer encoding, so the length
                // of the streamed body must be declared up front.
                let len = file.metadata().await?.len();
                Ok(self
                    .storage
                    .put(url)
                    .header(CONTENT_TYPE, "application/octet-stream")
                    .header(CONTENT_LENGTH, len)
                    .body(file))
            })
            .await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(map_http_error(status, &resp.bytes().await?));
        }
        Ok(UploadedFile {
            file_token: presigned.file_token.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presign_format_lowercases_extension() {
        assert_eq!(presign_format(Path::new("a/Model.GLB")).unwrap(), "glb");
        assert_eq!(presign_format(Path::new("scan.3mf")).unwrap(), "3mf");
    }

    #[test]
    fn presign_format_rejects_unsupported_or_missing_extension() {
        for p in ["model.blend", "noext", "archive.zip"] {
            let err = presign_format(Path::new(p)).unwrap_err();
            assert!(matches!(err, Error::InvalidRequest(_)), "{p}: {err}");
        }
    }

    #[test]
    fn presigned_upload_accepts_upload_url_alias() {
        let p: PresignedUpload = serde_json::from_str(
            r#"{"file_token":"file_1","upload_url":"https://s/x","object_key":"k"}"#,
        )
        .unwrap();
        assert_eq!(p.presigned_url, "https://s/x");
        assert_eq!(p.expires_in, None);
    }
}
