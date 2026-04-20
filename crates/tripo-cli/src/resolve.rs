//! Build a `tripo_api::Client` from CLI global args, honoring env vars.

use anyhow::Context;

use crate::cli::GlobalArgs;

/// Construct the client. Exits with usage error if the API key is missing/invalid.
#[allow(dead_code)] // First caller wires up in Task 4.
pub fn build_client(g: &GlobalArgs) -> anyhow::Result<tripo_api::Client> {
    let mut b = tripo_api::Client::builder();
    if let Some(key) = &g.api_key {
        b = b.api_key(key);
    } else if let Ok(env_key) = std::env::var(tripo_api::API_KEY_ENV) {
        b = b.api_key(env_key);
    }
    b = b.region(g.region.into());
    if let Some(url) = &g.base_url {
        b = b.base_url(url.clone());
    }
    b.build().context("failed to build Tripo client")
}
