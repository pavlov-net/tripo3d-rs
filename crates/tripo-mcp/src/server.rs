//! `TripoServer` — the MCP handler.
//!
//! Tool methods are attached to this struct via `#[tool]` and aggregated by
//! `#[tool_router]`. `#[tool_handler]` then fills in the `list_tools` and
//! `call_tool` hooks on `impl ServerHandler`.

use std::sync::Arc;

use rmcp::{
    ErrorData, Json, ServerHandler,
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
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

#[tool_router]
impl TripoServer {
    /// Get the account balance.
    #[tool(
        name = "get_balance",
        description = "Get the current Tripo account balance.",
        annotations(
            title = "Account Balance",
            read_only_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    async fn get_balance(&self) -> Result<Json<tripo_api::Balance>, ErrorData> {
        let bal = self.client.get_balance().await.map_err(to_error_data)?;
        Ok(Json(bal))
    }
}

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
