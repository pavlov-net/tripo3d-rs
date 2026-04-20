//! The `compress` field on text/image/multiview/texture task variants serializes
//! as the literal string `"geometry"` when enabled and is omitted when disabled.
//!
//! Model as `Option<CompressionMode>` with `skip_serializing_if = "Option::is_none"`.

use serde::{Deserialize, Serialize};

/// Compression options. Currently the server only supports `Geometry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CompressionMode {
    /// Serializes as the string `"geometry"`.
    Geometry,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Wrap {
        #[serde(skip_serializing_if = "Option::is_none")]
        compress: Option<CompressionMode>,
    }

    #[test]
    fn none_is_omitted() {
        let j = serde_json::to_string(&Wrap { compress: None }).unwrap();
        assert_eq!(j, "{}");
    }

    #[test]
    fn geometry_is_string_literal() {
        let j = serde_json::to_string(&Wrap {
            compress: Some(CompressionMode::Geometry),
        })
        .unwrap();
        assert_eq!(j, "{\"compress\":\"geometry\"}");
    }
}
