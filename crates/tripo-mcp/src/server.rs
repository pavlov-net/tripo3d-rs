//! `TripoServer` — the MCP handler.
//!
//! Tool methods are attached to this struct via `#[tool]` and aggregated by
//! `#[tool_router]`. `#[tool_handler]` then fills in the `list_tools` and
//! `call_tool` hooks on `impl ServerHandler`.

use std::sync::Arc;

use rmcp::{
    ErrorData, ServerHandler,
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool_handler, tool_router,
};
use tripo_api::Client;

#[derive(Clone)]
pub struct TripoServer {
    pub client: Arc<Client>,
}

impl TripoServer {
    /// Build a server around an already-configured [`Client`].
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

// Tool methods are added in subsequent tasks.
#[tool_router]
impl TripoServer {}

#[tool_handler]
impl ServerHandler for TripoServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_06_18)
            .with_server_info(Implementation::from_build_env())
            .with_instructions(
                "Tools for submitting, polling, downloading, and managing Tripo 3D generation tasks."
                    .to_string(),
            )
    }
}

/// Map a [`tripo_api::Error`] into a JSON-RPC [`ErrorData`] suitable for
/// returning from a tool method.
#[allow(
    dead_code,
    reason = "wired up as tool methods land in subsequent tasks"
)]
pub(crate) fn to_error_data(err: tripo_api::Error) -> ErrorData {
    match err {
        tripo_api::Error::Api {
            code,
            message,
            suggestion,
        } => {
            let text = suggestion.map_or_else(
                || format!("[{code}] {message}"),
                |s| format!("[{code}] {message} — {s}"),
            );
            ErrorData::internal_error(text, None)
        }
        other => ErrorData::internal_error(other.to_string(), None),
    }
}
