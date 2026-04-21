//! `tripo-mcp` entry point.

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tripo_mcp::server;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    tracing::info!("tripo-mcp starting");

    let client = tripo_api::Client::new()?;
    let server = server::TripoServer::new(client);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("tripo_mcp=info,tripo_api=info"));
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_ansi(false),
        )
        .with(filter)
        .init();
}
