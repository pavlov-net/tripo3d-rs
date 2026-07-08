//! `POST /files` multipart upload helper.

use std::path::Path;

use serde::Deserialize;

use crate::client::Client;
use crate::error::{Error, Result};
use crate::types::UploadedFile;

#[derive(Deserialize)]
struct FileTokenBody {
    file_token: String,
}

impl Client {
    /// Upload a local file and return a token usable as `ImageInput::FileToken`
    /// or the `input` field of other API calls.
    #[tracing::instrument(skip(self), fields(path = %path.as_ref().display()))]
    pub async fn upload_file(&self, path: impl AsRef<Path>) -> Result<UploadedFile> {
        let path = path.as_ref();
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
        let status = resp.status();
        let body = resp.bytes().await?;
        if !status.is_success() {
            return Err(crate::envelope::map_http_error(status, &body));
        }
        let env: crate::envelope::Envelope<FileTokenBody> = serde_json::from_slice(&body)?;
        let data = env.into_result()?;
        Ok(UploadedFile {
            file_token: data.file_token,
        })
    }
}
