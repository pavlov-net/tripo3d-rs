//! `Client`: entry point for the library. Builds a configured `reqwest::Client`
//! and carries the API key + base URL + retry policy.

use std::time::Duration;

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use url::Url;

use crate::error::{Error, Result};
use crate::retry::RetryPolicy;

/// Env var name for the API key.
pub const API_KEY_ENV: &str = "TRIPO_API_KEY";

/// Env var name for the region selector (`global` | `cn`).
pub const REGION_ENV: &str = "TRIPO_REGION";

/// Maximum number of ids [`Client::list_tasks`] accepts per request.
pub const MAX_LIST_TASK_IDS: usize = 100;

/// Global v3 base URL.
pub const BASE_URL_GLOBAL: &str = "https://openapi.tripo3d.ai/v3";
/// China mainland v3 base URL.
pub const BASE_URL_CN: &str = "https://openapi.tripo3d.com/v3";

/// Region selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Region {
    /// Global endpoint (default).
    #[default]
    Global,
    /// China mainland endpoint.
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
    pub(crate) retry: RetryPolicy,
    /// Unauthenticated client for signed storage URLs (uploads and downloads).
    pub(crate) storage: reqwest::Client,
    /// Files larger than this many bytes go through presigned upload.
    pub(crate) presign_threshold: u64,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url.as_str())
            .field("region", &self.region)
            .finish_non_exhaustive()
    }
}

/// Rejects an empty key. A key without the usual `tsk_` prefix only logs a
/// warning: credential-injecting proxies put a placeholder in the key and
/// swap in the real one in transit, and the server rejects bad keys itself.
fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(Error::MissingApiKey);
    }
    if !key.starts_with("tsk_") {
        tracing::warn!("API key does not start with `tsk_`; sending it anyway");
    }
    Ok(())
}

const USER_AGENT_VALUE: &str = concat!(
    "tripo-rs/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/pavlov-net/tripo3d-cli)"
);

fn build_http(api_key: &str) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    let mut auth =
        HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|_| Error::InvalidApiKey)?;
    auth.set_sensitive(true);
    headers.insert(AUTHORIZATION, auth);
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
    reqwest::Client::builder()
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_mins(1))
        .build()
        .map_err(Error::from)
}

/// Client for signed storage URLs: the signature in the URL is the only
/// credential, so it carries no `Authorization` header. It has no overall
/// timeout because large transfers can legitimately take many minutes.
fn build_storage_http() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT_VALUE)
        .connect_timeout(Duration::from_secs(10))
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

    /// `GET /account/balance` — current account balance.
    #[tracing::instrument(skip(self))]
    pub async fn get_balance(&self) -> Result<crate::types::Balance> {
        let url = self.url(&["account", "balance"]);
        self.send_json(|| self.http.get(url.clone())).await
    }

    /// `GET /account/usage` — per-task credit consumption history.
    #[tracing::instrument(skip(self))]
    pub async fn get_usage(
        &self,
        query: crate::types::UsageQuery,
    ) -> Result<Vec<crate::types::UsageRecord>> {
        let mut url = self.url(&["account", "usage"]);
        for (key, value) in [("limit", query.limit), ("offset", query.offset)] {
            if let Some(v) = value {
                url.query_pairs_mut().append_pair(key, &v.to_string());
            }
        }
        self.send_json(|| self.http.get(url.clone())).await
    }

    /// `GET /tasks/{id}` — current state of an existing task.
    #[tracing::instrument(skip(self), fields(task_id = %id))]
    pub async fn get_task(&self, id: &crate::types::TaskId) -> Result<crate::types::Task> {
        let url = self.url(&["tasks", id.as_str()]);
        self.send_json(|| self.http.get(url.clone())).await
    }

    /// `POST /tasks/list` — fetch up to [`MAX_LIST_TASK_IDS`] tasks in one
    /// request.
    #[tracing::instrument(skip(self), fields(count = ids.len()))]
    pub async fn list_tasks(&self, ids: &[crate::types::TaskId]) -> Result<crate::types::TaskList> {
        if ids.is_empty() || ids.len() > MAX_LIST_TASK_IDS {
            return Err(Error::InvalidRequest(format!(
                "list_tasks takes 1..={MAX_LIST_TASK_IDS} task ids, got {}",
                ids.len()
            )));
        }
        let url = self.url(&["tasks", "list"]);
        let body = serde_json::json!({ "task_ids": ids });
        let data = self
            .send_json(|| self.http.post(url.clone()).json(&body))
            .await?;
        Ok(crate::types::TaskList::from_data(data, ids)?)
    }

    /// Submit a task to its v3 capability endpoint (e.g. `POST
    /// /generation/text-to-model`). Any `ImageInput::Path` in the request is
    /// uploaded first and replaced with a `FileToken`.
    #[tracing::instrument(skip(self, req))]
    pub async fn create_task(
        &self,
        mut req: crate::tasks::TaskRequest,
    ) -> Result<crate::types::TaskId> {
        req.validate()?;
        req.upload_images(self).await?;
        self.create_task_raw(req.endpoint(), &serde_json::to_value(&req)?)
            .await
    }

    /// Submit an already-built JSON body to a task-creation endpoint given as
    /// a path relative to the base URL (e.g. `generation/text-to-model`).
    /// Used by `create_task` and the CLI's `task create --body <FILE>` escape
    /// hatch.
    pub async fn create_task_raw(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> Result<crate::types::TaskId> {
        #[derive(serde::Deserialize)]
        struct TaskIdBody {
            task_id: String,
        }
        let segments: Vec<&str> = endpoint.split('/').filter(|s| !s.is_empty()).collect();
        let url = self.url(&segments);
        let created: TaskIdBody = self
            .send_json(|| self.http.post(url.clone()).json(body))
            .await?;
        Ok(crate::types::TaskId(created.task_id))
    }

    /// Send with retry, then decode the success envelope's `data` as `T`.
    async fn send_json<T, F>(&self, build: F) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        F: Fn() -> reqwest::RequestBuilder,
    {
        let resp = self.send_with_retry(build).await?;
        crate::envelope::read_envelope(resp).await
    }

    pub(crate) async fn send_with_retry<F>(&self, build: F) -> Result<reqwest::Response>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        self.send_with_retry_async(|| std::future::ready(Ok(build())))
            .await
    }

    /// [`Self::send_with_retry`] for requests whose construction is async and
    /// fallible, such as a streamed file body that must be reopened for each
    /// attempt.
    pub(crate) async fn send_with_retry_async<F, Fut>(&self, build: F) -> Result<reqwest::Response>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<reqwest::RequestBuilder>>,
    {
        use crate::retry::{RetryDecision, parse_retry_after};

        let mut attempt: u32 = 0;
        loop {
            let req = build().await?;
            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() || (status.is_client_error() && status.as_u16() != 429) {
                        return Ok(resp);
                    }
                    let retry_after = resp
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(parse_retry_after);
                    match self.retry.decide_status(attempt, status, retry_after) {
                        RetryDecision::Stop => return Ok(resp),
                        RetryDecision::Retry(d) => {
                            tracing::debug!(?status, ?d, attempt, "retrying after status");
                            tokio::time::sleep(d).await;
                        }
                    }
                }
                Err(err) => match self.retry.decide_transport(attempt, &err) {
                    RetryDecision::Stop => return Err(Error::from(err)),
                    RetryDecision::Retry(d) => {
                        tracing::debug!(error = %err, ?d, attempt, "retrying after transport error");
                        tokio::time::sleep(d).await;
                    }
                },
            }
            attempt += 1;
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
    presign_threshold: Option<u64>,
}

impl ClientBuilder {
    /// Set the API key.
    #[must_use]
    pub fn api_key(mut self, k: impl Into<String>) -> Self {
        self.api_key = Some(k.into());
        self
    }
    /// Set the region (determines the default base URL).
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
    /// Size in bytes above which [`Client::upload_file`] uses presigned
    /// upload instead of multipart `POST /files`. Defaults to
    /// [`DEFAULT_PRESIGN_THRESHOLD`](crate::DEFAULT_PRESIGN_THRESHOLD);
    /// `u64::MAX` disables the switch.
    #[must_use]
    pub fn presign_threshold(mut self, bytes: u64) -> Self {
        self.presign_threshold = Some(bytes);
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
            retry: self.retry.unwrap_or_default(),
            storage: build_storage_http()?,
            presign_threshold: self
                .presign_threshold
                .unwrap_or(crate::upload::DEFAULT_PRESIGN_THRESHOLD),
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
    fn accepts_key_without_tsk_prefix() {
        Client::builder()
            .api_key("sbx-placeholder")
            .build()
            .unwrap();
    }

    #[test]
    fn rejects_empty_key() {
        let err = Client::builder().api_key("").build().unwrap_err();
        assert!(matches!(err, Error::MissingApiKey));
    }

    #[test]
    fn rejects_key_invalid_in_header() {
        let err = Client::builder().api_key("tsk_a\nb").build().unwrap_err();
        assert!(matches!(err, Error::InvalidApiKey));
    }

    #[test]
    fn region_defaults_global() {
        let c = Client::builder().api_key("tsk_abc").build().unwrap();
        assert_eq!(c.region(), Region::Global);
        assert_eq!(c.base_url().as_str(), "https://openapi.tripo3d.ai/v3");
    }

    #[test]
    fn region_cn_switches_base_url() {
        let c = Client::builder()
            .api_key("tsk_abc")
            .region(Region::Cn)
            .build()
            .unwrap();
        assert_eq!(c.base_url().as_str(), "https://openapi.tripo3d.com/v3");
    }

    #[test]
    fn url_joins_segments() {
        let c = Client::builder().api_key("tsk_abc").build().unwrap();
        let u = c.url(&["tasks", "abc123"]);
        assert_eq!(u.as_str(), "https://openapi.tripo3d.ai/v3/tasks/abc123");
    }
}
