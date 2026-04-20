//! Tripo response envelope: `{code, data}` on success or `{code, message, suggestion}` on error.

use serde::Deserialize;

use crate::error::Error;

/// Raw wire envelope. `data` is success-only; `message` is error-only.
#[derive(Debug, Deserialize)]
pub(crate) struct Envelope<T> {
    pub code: i32,
    #[serde(default = "Option::default")]
    pub data: Option<T>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub suggestion: Option<String>,
}

impl<T> Envelope<T> {
    /// Convert the envelope to a `Result`: `code == 0` → `Ok(data)`, else `Err(Error::Api)`.
    pub(crate) fn into_result(self) -> Result<T, Error> {
        if self.code == 0 {
            self.data.ok_or(Error::Api {
                code: 0,
                message: "missing `data` field in success envelope".into(),
                suggestion: None,
            })
        } else {
            Err(Error::Api {
                code: self.code,
                message: self.message.unwrap_or_else(|| "no message".into()),
                suggestion: self.suggestion,
            })
        }
    }
}

/// Map an HTTP error response to an `Error`, preferring Tripo's envelope
/// shape when present, falling back to raw HTTP status/body otherwise.
pub(crate) fn map_http_error(status: reqwest::StatusCode, bytes: &[u8]) -> Error {
    if let Ok(env) = serde_json::from_slice::<Envelope<serde_json::Value>>(bytes)
        && env.code != 0
    {
        return Error::Api {
            code: env.code,
            message: env.message.unwrap_or_else(|| status.to_string()),
            suggestion: env.suggestion,
        };
    }
    Error::Http {
        status: status.as_u16(),
        message: String::from_utf8_lossy(bytes).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_success_envelope() {
        let json = r#"{"code":0,"data":{"balance":12.5,"frozen":0.0}}"#;
        let env: Envelope<serde_json::Value> = serde_json::from_str(json).unwrap();
        let data = env.into_result().unwrap();
        assert_eq!(data["balance"], 12.5);
    }

    #[test]
    fn parses_error_envelope() {
        let json = r#"{"code":1001,"message":"bad key","suggestion":"regenerate"}"#;
        let env: Envelope<serde_json::Value> = serde_json::from_str(json).unwrap();
        let err = env.into_result().unwrap_err();
        let crate::Error::Api {
            code,
            message,
            suggestion,
        } = err
        else {
            panic!("wrong variant");
        };
        assert_eq!(code, 1001);
        assert_eq!(message, "bad key");
        assert_eq!(suggestion.as_deref(), Some("regenerate"));
    }

    #[test]
    fn error_envelope_without_suggestion() {
        let json = r#"{"code":500,"message":"boom"}"#;
        let env: Envelope<serde_json::Value> = serde_json::from_str(json).unwrap();
        let err = env.into_result().unwrap_err();
        assert!(matches!(err, crate::Error::Api { code: 500, .. }));
    }
}
