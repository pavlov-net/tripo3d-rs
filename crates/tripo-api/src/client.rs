//! `Client`: entry point for the library. Builds a configured `reqwest::Client`
//! and carries the API key + base URL + retry policy.

use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, USER_AGENT};
use url::Url;

use crate::error::{Error, Result};
use crate::retry::RetryPolicy;

/// Env var name for the API key.
pub const API_KEY_ENV: &str = "TRIPO_API_KEY";

/// Env var name for the region selector (`global` | `cn`).
pub const REGION_ENV: &str = "TRIPO_REGION";

/// Global v2 openapi base URL.
pub const BASE_URL_GLOBAL: &str = "https://api.tripo3d.ai/v2/openapi";
/// China mainland v2 openapi base URL.
pub const BASE_URL_CN: &str = "https://api.tripo3d.com/v2/openapi";

/// Region selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Region {
    /// Global endpoint (default).
    #[default]
    Global,
    /// China mainland endpoint. Adds `X-Tripo-Region: rg2` on GETs.
    Cn,
}

impl Region {
    /// Parse the `TRIPO_REGION` env form: `global` | `cn`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "global" | "" => Some(Self::Global),
            "cn" | "china" | "mainland" => Some(Self::Cn),
            _ => None,
        }
    }

    /// Default base URL for this region.
    #[must_use]
    pub fn default_base_url(self) -> Url {
        match self {
            Self::Global => BASE_URL_GLOBAL.parse().expect("valid const URL"),
            Self::Cn => BASE_URL_CN.parse().expect("valid const URL"),
        }
    }
}

/// Async client for the Tripo 3D Generation API.
#[derive(Clone)]
pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: Url,
    pub(crate) region: Region,
    #[allow(dead_code)]
    pub(crate) api_key: Arc<str>,
    #[allow(dead_code)]
    pub(crate) retry: RetryPolicy,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url.as_str())
            .field("region", &self.region)
            .field("api_key", &"<redacted>")
            .finish_non_exhaustive()
    }
}

fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(Error::MissingApiKey);
    }
    if !key.starts_with("tsk_") {
        return Err(Error::InvalidApiKey);
    }
    Ok(())
}

fn build_http(api_key: &str) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    let mut auth = HeaderValue::from_str(&format!("Bearer {api_key}"))
        .map_err(|_| Error::InvalidApiKey)?;
    auth.set_sensitive(true);
    headers.insert(AUTHORIZATION, auth);
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(concat!(
            "tripo-rs/",
            env!("CARGO_PKG_VERSION"),
            " (+https://github.com/stuartparmenter/tripo3d-cli)"
        )),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .http2_prior_knowledge()
        .build()
        .map_err(Error::from)
}

impl Client {
    /// Read `TRIPO_API_KEY` (and optionally `TRIPO_REGION`) from the environment.
    pub fn new() -> Result<Self> {
        let key = std::env::var(API_KEY_ENV).map_err(|_| Error::MissingApiKey)?;
        let region = std::env::var(REGION_ENV)
            .ok()
            .and_then(|r| Region::parse(&r))
            .unwrap_or_default();
        Self::builder().api_key(key).region(region).build()
    }

    /// Start a [`ClientBuilder`].
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Construct with an explicit key, using defaults for everything else.
    pub fn with_api_key(key: impl Into<String>) -> Result<Self> {
        Self::builder().api_key(key).build()
    }

    /// Override the base URL (testing or staging).
    #[must_use]
    pub fn with_base_url(mut self, url: Url) -> Self {
        self.base_url = url;
        self
    }

    /// Current base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Current region.
    #[must_use]
    pub fn region(&self) -> Region {
        self.region
    }

    /// Join `segments` onto the base URL, preserving the existing path.
    pub(crate) fn url(&self, segments: &[&str]) -> Url {
        let mut u = self.base_url.clone();
        {
            let mut seg = u.path_segments_mut().expect("http(s) base");
            for s in segments {
                seg.push(s);
            }
        }
        u
    }

    /// Extra headers attached to every request. `X-Tripo-Region: rg2` for CN.
    pub(crate) fn region_headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        if self.region == Region::Cn {
            h.insert(
                HeaderName::from_static("x-tripo-region"),
                HeaderValue::from_static("rg2"),
            );
        }
        h
    }

    /// `GET /user/balance` — current account balance.
    #[tracing::instrument(skip(self))]
    pub async fn get_balance(&self) -> Result<crate::types::Balance> {
        let url = self.url(&["user", "balance"]);
        let resp = self.http.get(url).headers(self.region_headers()).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(self.map_http_error(status, &bytes));
        }
        let env: crate::envelope::Envelope<crate::types::Balance> =
            serde_json::from_slice(&bytes)?;
        env.into_result()
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn map_http_error(&self, status: reqwest::StatusCode, bytes: &[u8]) -> Error {
        if let Ok(env) = serde_json::from_slice::<crate::envelope::Envelope<serde_json::Value>>(bytes) {
            if env.code != 0 {
                return Error::Api {
                    code: env.code,
                    message: env.message.unwrap_or_else(|| status.to_string()),
                    suggestion: env.suggestion,
                };
            }
        }
        Error::Http {
            status: status.as_u16(),
            message: String::from_utf8_lossy(bytes).into_owned(),
        }
    }
}

/// Builder for [`Client`].
#[derive(Default)]
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: Option<Url>,
    region: Option<Region>,
    retry: Option<RetryPolicy>,
}

impl ClientBuilder {
    /// Set the API key.
    #[must_use]
    pub fn api_key(mut self, k: impl Into<String>) -> Self {
        self.api_key = Some(k.into());
        self
    }
    /// Set the region (determines default base URL and `X-Tripo-Region` header).
    #[must_use]
    pub fn region(mut self, r: Region) -> Self {
        self.region = Some(r);
        self
    }
    /// Override the base URL (ignores region's default).
    #[must_use]
    pub fn base_url(mut self, u: Url) -> Self {
        self.base_url = Some(u);
        self
    }
    /// Override the retry policy.
    #[must_use]
    pub fn retry(mut self, r: RetryPolicy) -> Self {
        self.retry = Some(r);
        self
    }
    /// Build, validating the API key.
    pub fn build(self) -> Result<Client> {
        let key = self.api_key.ok_or(Error::MissingApiKey)?;
        validate_key(&key)?;
        let region = self.region.unwrap_or_default();
        let base_url = self.base_url.unwrap_or_else(|| region.default_base_url());
        let http = build_http(&key)?;
        Ok(Client {
            http,
            base_url,
            region,
            api_key: Arc::from(key),
            retry: self.retry.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_key() {
        let err = Client::builder().build().unwrap_err();
        assert!(matches!(err, Error::MissingApiKey));
    }

    #[test]
    fn rejects_bad_prefix() {
        let err = Client::builder().api_key("wrong_prefix").build().unwrap_err();
        assert!(matches!(err, Error::InvalidApiKey));
    }

    #[test]
    fn region_defaults_global() {
        let c = Client::builder().api_key("tsk_abc").build().unwrap();
        assert_eq!(c.region(), Region::Global);
        assert_eq!(c.base_url().as_str(), "https://api.tripo3d.ai/v2/openapi");
    }

    #[test]
    fn region_cn_switches_base_url() {
        let c = Client::builder()
            .api_key("tsk_abc")
            .region(Region::Cn)
            .build()
            .unwrap();
        assert_eq!(c.base_url().as_str(), "https://api.tripo3d.com/v2/openapi");
        assert!(c.region_headers().contains_key("x-tripo-region"));
    }

    #[test]
    fn url_joins_segments() {
        let c = Client::builder().api_key("tsk_abc").build().unwrap();
        let u = c.url(&["task", "abc123"]);
        assert_eq!(u.as_str(), "https://api.tripo3d.ai/v2/openapi/task/abc123");
    }
}
