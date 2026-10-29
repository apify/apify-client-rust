//! The top-level [`ApifyClient`] and its [`ApifyClientBuilder`].

use std::sync::Arc;
use std::time::Duration;

use crate::clients::actor::ActorClient;
use crate::clients::actor_collection::ActorCollectionClient;
use crate::clients::build::BuildClient;
use crate::clients::build_collection::BuildCollectionClient;
use crate::clients::dataset::DatasetClient;
use crate::clients::dataset_collection::DatasetCollectionClient;
use crate::clients::key_value_store::KeyValueStoreClient;
use crate::clients::key_value_store_collection::KeyValueStoreCollectionClient;
use crate::clients::log::LogClient;
use crate::clients::request_queue::RequestQueueClient;
use crate::clients::request_queue_collection::RequestQueueCollectionClient;
use crate::clients::run::RunClient;
use crate::clients::run_collection::RunCollectionClient;
use crate::clients::schedule::ScheduleClient;
use crate::clients::schedule_collection::ScheduleCollectionClient;
use crate::clients::store_collection::StoreCollectionClient;
use crate::clients::task::TaskClient;
use crate::clients::task_collection::TaskCollectionClient;
use crate::clients::user::UserClient;
use crate::clients::webhook::WebhookClient;
use crate::clients::webhook_collection::WebhookCollectionClient;
use crate::clients::webhook_dispatch::WebhookDispatchClient;
use crate::clients::webhook_dispatch_collection::WebhookDispatchCollectionClient;
use crate::common::build_user_agent;
use crate::http_client::{HttpBackend, HttpClient, HttpClientConfig, ReqwestBackend, RetryConfig};
use crate::http_compressors::{HttpCompressor, RequestCompression};
use crate::timeouts::TimeoutTiers;

/// Default base URL of the Apify API (without the `/v2` suffix).
const DEFAULT_BASE_URL: &str = "https://api.apify.com";
/// Apify's API version path segment, appended to a configured base URL unless it is already
/// present there (see [`to_api_base_url`]).
const API_VERSION_PATH: &str = "/v2";
/// Default maximum number of retries, matching the reference clients.
const DEFAULT_MAX_RETRIES: u32 = 8;
/// Default minimum delay between retries.
const DEFAULT_MIN_DELAY_BETWEEN_RETRIES: Duration = Duration::from_millis(500);
/// Placeholder used to address the current user (`/users/me`).
pub(crate) const ME_USER_PLACEHOLDER: &str = "me";

/// Builder for [`ApifyClient`].
///
/// # Example
/// ```no_run
/// use apify_client::ApifyClient;
///
/// let client = ApifyClient::builder()
///     .token("my-api-token")
///     .max_retries(5)
///     .build();
/// ```
pub struct ApifyClientBuilder {
    token: Option<String>,
    base_url: String,
    public_base_url: Option<String>,
    max_retries: u32,
    min_delay_between_retries: Duration,
    tiers: TimeoutTiers,
    user_agent_suffix: Option<String>,
    http_backend: Option<Arc<dyn HttpBackend>>,
    compressor: Arc<dyn HttpCompressor>,
    default_headers: Vec<(String, String)>,
}

impl Default for ApifyClientBuilder {
    fn default() -> Self {
        Self {
            token: None,
            base_url: DEFAULT_BASE_URL.to_string(),
            public_base_url: None,
            max_retries: DEFAULT_MAX_RETRIES,
            min_delay_between_retries: DEFAULT_MIN_DELAY_BETWEEN_RETRIES,
            tiers: TimeoutTiers::default(),
            user_agent_suffix: None,
            http_backend: None,
            compressor: RequestCompression::default().into_compressor(),
            default_headers: Vec::new(),
        }
    }
}

impl ApifyClientBuilder {
    /// Sets the API token used for authentication (sent as a `Bearer` token).
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Overrides the base URL of the API, with or without the `/v2` version path — it is
    /// appended automatically when not already the URL's final path segment, so passing a URL
    /// that already ends in `/v2` (e.g. one read back from [`ApifyClient::api_base_url`]) does
    /// not double it up.
    ///
    /// Defaults to `https://api.apify.com`.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Overrides the base URL used when building public, shareable resource URLs (e.g. a
    /// signed dataset-items URL). Defaults to the API base URL. Like [`base_url`](Self::base_url),
    /// the `/v2` version path is appended automatically only when not already present.
    pub fn public_base_url(mut self, public_base_url: impl Into<String>) -> Self {
        self.public_base_url = Some(public_base_url.into());
        self
    }

    /// Sets the maximum number of retries for failed requests (default `8`).
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Sets the minimum delay between retries (default `500ms`).
    pub fn min_delay_between_retries(mut self, delay: Duration) -> Self {
        self.min_delay_between_retries = delay;
        self
    }

    /// Sets the duration of the [`Timeout::Short`](crate::Timeout::Short) tier, used by simple
    /// metadata reads and writes (default `5s`).
    pub fn timeout_short(mut self, timeout: Duration) -> Self {
        self.tiers.short = timeout;
        self
    }

    /// Sets the duration of the [`Timeout::Medium`](crate::Timeout::Medium) tier, used by
    /// listing, batch and trigger operations (default `30s`).
    pub fn timeout_medium(mut self, timeout: Duration) -> Self {
        self.tiers.medium = timeout;
        self
    }

    /// Sets the duration of the [`Timeout::Long`](crate::Timeout::Long) tier, used by downloads,
    /// uploads and streaming (default `360s`).
    pub fn timeout_long(mut self, timeout: Duration) -> Self {
        self.tiers.long = timeout;
        self
    }

    /// Caps the timeout of a single request attempt (default `360s`). A tier or an explicit
    /// timeout above the cap is lowered to it. The timeout of each attempt doubles with every
    /// retry, up to this cap.
    pub fn timeout_max(mut self, timeout: Duration) -> Self {
        self.tiers.max = timeout;
        self
    }

    /// Caps the timeout of a single request attempt (default `360s`). Equivalent to
    /// [`timeout_max`](Self::timeout_max), kept under its original name.
    pub fn timeout(self, timeout: Duration) -> Self {
        self.timeout_max(timeout)
    }

    /// Appends a custom suffix to the `User-Agent` header.
    pub fn user_agent_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.user_agent_suffix = Some(suffix.into());
        self
    }

    /// Selects the built-in algorithm used to compress large request bodies (default
    /// [`RequestCompression::Brotli`]), at its default quality.
    ///
    /// The client compresses request bodies of at least 1024 bytes; this chooses whether that
    /// uses brotli (`Content-Encoding: br`) or gzip (`Content-Encoding: gzip`). Brotli is
    /// preferred (matching the reference client); select [`RequestCompression::Gzip`] for
    /// environments or intermediaries that do not handle brotli. To tune the quality or use
    /// another algorithm, pass an [`HttpCompressor`] to [`http_compressor`](Self::http_compressor).
    pub fn request_compression(mut self, compression: RequestCompression) -> Self {
        self.compressor = compression.into_compressor();
        self
    }

    /// Replaces the request-body compressor with a custom [`HttpCompressor`], for example a
    /// [`BrotliHttpCompressor`](crate::BrotliHttpCompressor) with a non-default quality.
    pub fn http_compressor(mut self, compressor: Arc<dyn HttpCompressor>) -> Self {
        self.compressor = compressor;
        self
    }

    /// Adds a header sent with every request. It wins over the built-in `User-Agent` and
    /// `Authorization` headers, and a header a request sets itself wins over it. Header names are
    /// compared case-insensitively, and adding the same name again replaces the earlier value.
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        self.default_headers
            .retain(|(existing, _)| !existing.eq_ignore_ascii_case(&name));
        self.default_headers.push((name, value.into()));
        self
    }

    /// Replaces the default [`reqwest`]-based HTTP backend with a custom implementation.
    ///
    /// This is the seam that makes the transport a replaceable component.
    pub fn http_backend(mut self, backend: Arc<dyn HttpBackend>) -> Self {
        self.http_backend = Some(backend);
        self
    }

    /// Builds the configured [`ApifyClient`].
    pub fn build(self) -> ApifyClient {
        let backend = self
            .http_backend
            .unwrap_or_else(|| Arc::new(ReqwestBackend::new()));

        let user_agent = build_user_agent(self.user_agent_suffix.as_deref());

        let retry = RetryConfig {
            max_retries: self.max_retries,
            min_delay_between_retries: self.min_delay_between_retries,
        };

        let http = HttpClient::new(HttpClientConfig {
            backend,
            token: self.token,
            user_agent,
            retry,
            tiers: self.tiers,
            compressor: self.compressor,
            default_headers: self.default_headers,
        });

        let base_url = to_api_base_url(&self.base_url);
        let public_base_url =
            to_api_base_url(self.public_base_url.as_deref().unwrap_or(&self.base_url));

        ApifyClient {
            http,
            base_url,
            public_base_url,
        }
    }
}

/// The entry point for interacting with the Apify API.
///
/// Construct it with [`ApifyClient::builder`] (or [`ApifyClient::new`] for a quick
/// token-only setup), then obtain resource clients via the accessor methods, e.g.
/// [`ApifyClient::actor`], [`ApifyClient::dataset`], [`ApifyClient::run`].
///
/// The client is cheap to clone — all internal state is reference-counted.
#[derive(Debug, Clone)]
pub struct ApifyClient {
    http: HttpClient,
    base_url: String,
    public_base_url: String,
}

impl ApifyClient {
    /// Returns a [`ApifyClientBuilder`] for configuring a client.
    pub fn builder() -> ApifyClientBuilder {
        ApifyClientBuilder::default()
    }

    /// Creates a client authenticated with the given API token and default settings.
    pub fn new(token: impl Into<String>) -> Self {
        ApifyClientBuilder::default().token(token).build()
    }

    pub(crate) fn http(&self) -> HttpClient {
        self.http.clone()
    }

    /// The `User-Agent` header value this client sends.
    pub fn user_agent(&self) -> &str {
        self.http.user_agent()
    }

    /// The fully-qualified API base URL this client targets (including the `/v2` suffix),
    /// e.g. `https://api.apify.com/v2`. Reflects any `base_url` override.
    pub fn api_base_url(&self) -> &str {
        &self.base_url
    }

    // ----- Actor accessors -------------------------------------------------

    /// Returns a client for the Actor collection (list & create Actors).
    pub fn actors(&self) -> ActorCollectionClient {
        ActorCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific Actor, addressed by ID or `username~name`.
    pub fn actor(&self, id: impl Into<String>) -> ActorClient {
        ActorClient::new(self.clone(), self.http(), &self.base_url, &id.into())
    }

    // ----- Build accessors -------------------------------------------------

    /// Returns a client for the Actor build collection (list builds).
    pub fn builds(&self) -> BuildCollectionClient {
        BuildCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific Actor build.
    pub fn build(&self, id: impl Into<String>) -> BuildClient {
        BuildClient::new(self.http(), &self.base_url, &id.into())
    }

    // ----- Run accessors ---------------------------------------------------

    /// Returns a client for the Actor run collection (list runs).
    pub fn runs(&self) -> RunCollectionClient {
        RunCollectionClient::new(self.http(), &self.base_url, "actor-runs")
    }

    /// Returns a client for a specific Actor run.
    pub fn run(&self, id: impl Into<String>) -> RunClient {
        RunClient::new(
            self.clone(),
            self.http(),
            &self.base_url,
            "actor-runs",
            &id.into(),
        )
    }

    // ----- Dataset accessors ----------------------------------------------

    /// Returns a client for the dataset collection (list & get-or-create datasets).
    pub fn datasets(&self) -> DatasetCollectionClient {
        DatasetCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific dataset, addressed by ID or name.
    pub fn dataset(&self, id: impl Into<String>) -> DatasetClient {
        DatasetClient::new(self.http(), &self.base_url, "datasets", &id.into())
            .with_public_base(&self.public_base_url)
    }

    // ----- Key-value store accessors --------------------------------------

    /// Returns a client for the key-value store collection.
    pub fn key_value_stores(&self) -> KeyValueStoreCollectionClient {
        KeyValueStoreCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific key-value store, addressed by ID or name.
    pub fn key_value_store(&self, id: impl Into<String>) -> KeyValueStoreClient {
        KeyValueStoreClient::new(self.http(), &self.base_url, "key-value-stores", &id.into())
            .with_public_base(&self.public_base_url)
    }

    // ----- Request queue accessors ----------------------------------------

    /// Returns a client for the request queue collection.
    pub fn request_queues(&self) -> RequestQueueCollectionClient {
        RequestQueueCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific request queue, addressed by ID or name.
    pub fn request_queue(&self, id: impl Into<String>) -> RequestQueueClient {
        RequestQueueClient::new(self.http(), &self.base_url, "request-queues", &id.into())
    }

    // ----- Task accessors --------------------------------------------------

    /// Returns a client for the Actor task collection (list & create tasks).
    pub fn tasks(&self) -> TaskCollectionClient {
        TaskCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific Actor task.
    pub fn task(&self, id: impl Into<String>) -> TaskClient {
        TaskClient::new(self.clone(), self.http(), &self.base_url, &id.into())
    }

    // ----- Schedule accessors ---------------------------------------------

    /// Returns a client for the schedule collection (list & create schedules).
    pub fn schedules(&self) -> ScheduleCollectionClient {
        ScheduleCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific schedule.
    pub fn schedule(&self, id: impl Into<String>) -> ScheduleClient {
        ScheduleClient::new(self.http(), &self.base_url, &id.into())
    }

    // ----- Webhook accessors ----------------------------------------------

    /// Returns a client for the webhook collection (list & create webhooks).
    pub fn webhooks(&self) -> WebhookCollectionClient {
        WebhookCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific webhook.
    pub fn webhook(&self, id: impl Into<String>) -> WebhookClient {
        WebhookClient::new(self.http(), &self.base_url, &id.into())
    }

    /// Returns a client for the webhook dispatch collection.
    pub fn webhook_dispatches(&self) -> WebhookDispatchCollectionClient {
        WebhookDispatchCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for a specific webhook dispatch.
    pub fn webhook_dispatch(&self, id: impl Into<String>) -> WebhookDispatchClient {
        WebhookDispatchClient::new(self.http(), &self.base_url, &id.into())
    }

    // ----- Misc accessors --------------------------------------------------

    /// Returns a client for browsing the Apify Store.
    pub fn store(&self) -> StoreCollectionClient {
        StoreCollectionClient::new(self.http(), &self.base_url)
    }

    /// Returns a client for accessing a build's or run's log.
    pub fn log(&self, build_or_run_id: impl Into<String>) -> LogClient {
        LogClient::new(self.http(), &self.base_url, "logs", &build_or_run_id.into())
    }

    /// Returns a client for the current user (`/users/me`).
    pub fn me(&self) -> UserClient {
        UserClient::new(self.http(), &self.base_url, ME_USER_PLACEHOLDER)
    }

    /// Returns a client for a specific user by ID or username.
    pub fn user(&self, id: impl Into<String>) -> UserClient {
        UserClient::new(self.http(), &self.base_url, &id.into())
    }

    /// Sets the status message of the current Actor run.
    ///
    /// This convenience method updates the run identified by the `ACTOR_RUN_ID` environment
    /// variable, so it only works when called from inside an Actor run. If
    /// `is_terminal` is `true`, the message becomes final and won't be overwritten.
    ///
    /// Returns [`ApifyClientError::InvalidArgument`](crate::ApifyClientError::InvalidArgument)
    /// if `ACTOR_RUN_ID` is not set.
    pub async fn set_status_message(
        &self,
        message: &str,
        is_terminal: bool,
    ) -> crate::error::ApifyClientResult<crate::models::ActorRun> {
        let run_id = std::env::var("ACTOR_RUN_ID").map_err(|_| {
            crate::error::ApifyClientError::InvalidArgument(
                "ACTOR_RUN_ID environment variable is not set".to_string(),
            )
        })?;
        let body = serde_json::json!({
            "statusMessage": message,
            "isStatusMessageTerminal": is_terminal,
        });
        self.run(run_id).update(&body).await
    }
}

/// Appends [`API_VERSION_PATH`] to `url` unless it is already the final path segment, mirroring
/// the reference client's `toApiBaseUrl`.
///
/// A trailing slash (or several) is trimmed first, so `https://host/v2/` and `https://host/v2//`
/// both normalize to `https://host/v2` rather than growing a double slash. The check looks only
/// at the URL's *path*, not at the string as a whole: `https://v2` has no path (`v2` is the
/// host), so it still becomes `https://v2/v2` even though the raw string ends in `v2`.
fn to_api_base_url(url: &str) -> String {
    let (authority, path) = split_authority(url);
    let path = path.trim_end_matches('/');
    if path.ends_with(API_VERSION_PATH) {
        format!("{authority}{path}")
    } else {
        format!("{authority}{path}{API_VERSION_PATH}")
    }
}

/// Splits a URL into its authority (`scheme://host[:port]`, or the whole string if there is no
/// path) and its path (starting with `/`, or empty if there is none).
fn split_authority(url: &str) -> (&str, &str) {
    let search_from = url.find("://").map(|i| i + 3).unwrap_or(0);
    match url[search_from..].find('/') {
        Some(rel_idx) => url.split_at(search_from + rel_idx),
        None => (url, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::to_api_base_url;

    /// Mirrors the reference client's `toApiBaseUrl` test matrix exactly, so the two clients
    /// agree on every case, including the `https://v2` host-vs-path edge case.
    #[test]
    fn to_api_base_url_matches_reference_matrix() {
        let cases = [
            ("https://example.com", "https://example.com/v2"),
            ("https://example.com/", "https://example.com/v2"),
            ("https://example.com/v2", "https://example.com/v2"),
            ("https://example.com/v2/", "https://example.com/v2"),
            ("https://example.com/v2//", "https://example.com/v2"),
            (
                "https://example.com/proxy/v2",
                "https://example.com/proxy/v2",
            ),
            ("https://example.com/apiv2", "https://example.com/apiv2/v2"),
            ("https://v2", "https://v2/v2"),
        ];
        for (input, expected) in cases {
            assert_eq!(to_api_base_url(input), expected, "input: {input}");
        }
    }
}
