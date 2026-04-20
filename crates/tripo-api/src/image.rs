//! Image inputs: URL, pre-uploaded file token, or local path.
//!
//! On the wire every image is wrapped as `{"type":"jpg", ...}` regardless of the
//! actual file format (this is what the Tripo server expects). The body is one of:
//! `{"url": "..."}`, `{"file_token": "<uuid>"}`. Local paths must be uploaded
//! before serialization — the client's `upload_images` helper handles this.

use std::path::PathBuf;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use url::Url;
use uuid::Uuid;

/// A reference to an image, accepted by all image-consuming variants.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum ImageInput {
    /// A publicly fetchable URL.
    Url(Url),
    /// A token returned by a prior upload.
    FileToken(Uuid),
    /// A local file path — must be uploaded before the request is sent.
    Path(PathBuf),
}

impl ImageInput {
    /// Classify a string into a variant.
    ///
    /// `http://` / `https://` → [`ImageInput::Url`]. A canonical UUID → [`ImageInput::FileToken`].
    /// Anything else → [`ImageInput::Path`].
    #[must_use]
    pub fn parse(s: &str) -> Self {
        if s.starts_with("http://") || s.starts_with("https://") {
            if let Ok(url) = Url::parse(s) {
                return Self::Url(url);
            }
        }
        if let Ok(uuid) = Uuid::parse_str(s) {
            return Self::FileToken(uuid);
        }
        Self::Path(PathBuf::from(s))
    }
}

impl Serialize for ImageInput {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut st = ser.serialize_struct("FileContent", 2)?;
        st.serialize_field("type", "jpg")?;
        match self {
            Self::Url(u) => st.serialize_field("url", u.as_str())?,
            Self::FileToken(t) => st.serialize_field("file_token", &t.to_string())?,
            Self::Path(p) => {
                return Err(serde::ser::Error::custom(format!(
                    "ImageInput::Path({}) must be uploaded before serialization — call Client::upload_images on the request first",
                    p.display()
                )))
            }
        }
        st.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url() {
        let i = ImageInput::parse("https://example.com/x.jpg");
        assert!(matches!(i, ImageInput::Url(_)));
    }

    #[test]
    fn parse_file_token() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(matches!(ImageInput::parse(uuid), ImageInput::FileToken(_)));
    }

    #[test]
    fn parse_file_token_case_insensitive() {
        let uuid = "550E8400-E29B-41D4-A716-446655440000";
        assert!(matches!(ImageInput::parse(uuid), ImageInput::FileToken(_)));
    }

    #[test]
    fn parse_path() {
        let i = ImageInput::parse("./photo.png");
        assert!(matches!(i, ImageInput::Path(_)));
    }

    #[test]
    fn serialize_url() {
        let i = ImageInput::Url("https://example.com/x.jpg".parse().unwrap());
        let got: serde_json::Value = serde_json::to_value(&i).unwrap();
        assert_eq!(got, serde_json::json!({"type":"jpg","url":"https://example.com/x.jpg"}));
    }

    #[test]
    fn serialize_file_token() {
        let t = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let got: serde_json::Value = serde_json::to_value(ImageInput::FileToken(t)).unwrap();
        assert_eq!(
            got,
            serde_json::json!({"type":"jpg","file_token":"550e8400-e29b-41d4-a716-446655440000"})
        );
    }

    #[test]
    fn serialize_path_errors() {
        let err = serde_json::to_value(ImageInput::Path("./x.png".into())).unwrap_err();
        assert!(err.to_string().contains("must be uploaded"));
    }
}
