//! Image/model inputs: URL, pre-uploaded file token, or local path.
//!
//! v3 unifies all remote references into a single bare string — the server
//! infers whether it is a URL, a `file_token` from a prior upload, or a
//! `task_id` of a previous task. Local paths must be uploaded before
//! serialization — the client's `upload_images` helper handles this.

use std::path::PathBuf;

use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};
use url::Url;

/// A reference to an image (or model file), accepted by all input-consuming
/// variants. Serializes as a bare string; the server infers the input type.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum ImageInput {
    /// A publicly fetchable URL.
    Url(Url),
    /// A token returned by a prior upload, or a `task_id` of a previous task
    /// (the server extracts that task's output).
    FileToken(String),
    /// A local file path — must be uploaded before the request is sent.
    Path(PathBuf),
}

impl ImageInput {
    /// Classify a string into a variant.
    ///
    /// `http://` / `https://` → [`ImageInput::Url`]. A canonical UUID or a
    /// `file_` / `task_` prefixed token → [`ImageInput::FileToken`].
    /// Anything else → [`ImageInput::Path`].
    #[must_use]
    pub fn parse(s: &str) -> Self {
        if let Ok(url) = Url::parse(s)
            && matches!(url.scheme(), "http" | "https")
        {
            return Self::Url(url);
        }
        if uuid::Uuid::parse_str(s).is_ok() || s.starts_with("file_") || s.starts_with("task_") {
            return Self::FileToken(s.to_string());
        }
        Self::Path(PathBuf::from(s))
    }

    /// The wire string for this input. Errors for [`ImageInput::Path`], which
    /// must be uploaded first.
    pub(crate) fn as_wire_str(&self) -> Result<&str, String> {
        match self {
            Self::Url(u) => Ok(u.as_str()),
            Self::FileToken(t) => Ok(t.as_str()),
            Self::Path(p) => Err(format!(
                "ImageInput::Path({}) must be uploaded before serialization — call Client::upload_images on the request first",
                p.display()
            )),
        }
    }
}

impl Serialize for ImageInput {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self.as_wire_str() {
            Ok(s) => ser.serialize_str(s),
            Err(msg) => Err(serde::ser::Error::custom(msg)),
        }
    }
}

impl<'de> Deserialize<'de> for ImageInput {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        match v {
            serde_json::Value::String(s) => Ok(Self::parse(&s)),
            // Accept the legacy v2 object shapes ({"url": ...} / {"file_token": ...})
            // so stored request bodies keep deserializing.
            serde_json::Value::Object(mut m) => {
                m.remove("type");
                if let Some(url) = m.remove("url").and_then(|v| v.as_str().map(str::to_string)) {
                    Url::parse(&url).map(Self::Url).map_err(DeError::custom)
                } else if let Some(tok) = m
                    .remove("file_token")
                    .and_then(|v| v.as_str().map(str::to_string))
                {
                    Ok(Self::FileToken(tok))
                } else {
                    Err(DeError::custom(
                        "expected `url` or `file_token` in ImageInput object",
                    ))
                }
            }
            other => Err(DeError::custom(format!(
                "unexpected ImageInput shape: {other}"
            ))),
        }
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
    fn parse_uuid_file_token() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(matches!(ImageInput::parse(uuid), ImageInput::FileToken(_)));
    }

    #[test]
    fn parse_prefixed_tokens() {
        assert!(matches!(
            ImageInput::parse("file_abc123"),
            ImageInput::FileToken(_)
        ));
        assert!(matches!(
            ImageInput::parse("task_abc123"),
            ImageInput::FileToken(_)
        ));
    }

    #[test]
    fn parse_path() {
        let i = ImageInput::parse("./photo.png");
        assert!(matches!(i, ImageInput::Path(_)));
    }

    #[test]
    fn serialize_url_as_bare_string() {
        let i = ImageInput::Url("https://example.com/x.jpg".parse().unwrap());
        let got: serde_json::Value = serde_json::to_value(&i).unwrap();
        assert_eq!(got, serde_json::json!("https://example.com/x.jpg"));
    }

    #[test]
    fn serialize_file_token_as_bare_string() {
        let got: serde_json::Value =
            serde_json::to_value(ImageInput::FileToken("file_abc123".into())).unwrap();
        assert_eq!(got, serde_json::json!("file_abc123"));
    }

    #[test]
    fn serialize_path_errors() {
        let err = serde_json::to_value(ImageInput::Path("./x.png".into())).unwrap_err();
        assert!(err.to_string().contains("must be uploaded"));
    }

    #[test]
    fn deserialize_legacy_object_shape() {
        let i: ImageInput =
            serde_json::from_str(r#"{"type":"jpg","file_token":"file_abc"}"#).unwrap();
        assert_eq!(i, ImageInput::FileToken("file_abc".into()));
        let i: ImageInput = serde_json::from_str(r#"{"url":"https://cdn/x.png"}"#).unwrap();
        assert!(matches!(i, ImageInput::Url(_)));
    }
}
