//! The HTTP layer of the client.
//!
//! The [`HttpBackend`] trait defines the minimal contract for sending a single HTTP
//! request and receiving a response. It is the *replaceable component* of the client:
//! the default implementation [`ReqwestBackend`] uses [`reqwest`], but a custom
//! backend (e.g. for testing or for a different runtime) can be plugged in via
//! [`ApifyClientBuilder::http_backend`](crate::ApifyClientBuilder::http_backend).
//!
//! [`HttpClient`] wraps a backend and adds the cross-cutting concerns shared by every
//! endpoint: authentication, the `User-Agent` header, query-parameter serialization,
//! timeouts and retries with exponential backoff (mirroring the JavaScript and Python
//! reference clients).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::error::{ApiError, ApiErrorBody, ApifyClientError, ApifyClientResult};

/// HTTP status code returned by the API when the per-resource rate limit is exceeded.
const RATE_LIMIT_EXCEEDED_STATUS_CODE: u16 = 429;
/// Statuses `>= 500` are considered retryable internal server errors.
const MIN_SERVER_ERROR_STATUS_CODE: u16 = 500;
/// Responses with status `< 300` are treated as success.
const MAX_SUCCESS_STATUS_CODE: u16 = 300;
/// Multiplier applied to the inter-retry delay after each attempt (exponential backoff).
/// Matches the reference client's `async-retry` default factor of 2.
const BACKOFF_FACTOR: u32 = 2;

/// HTTP method of a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// HTTP `GET`.
    Get,
    /// HTTP `POST`.
    Post,
    /// HTTP `PUT`.
    Put,
    /// HTTP `DELETE`.
    Delete,
    /// HTTP `HEAD`.
    Head,
}

impl HttpMethod {
    /// Returns the uppercase string representation, e.g. `"GET"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
        }
    }
}

/// A fully-resolved HTTP request, ready to be sent by an [`HttpBackend`].
///
/// All cross-cutting concerns (auth header, user agent, retry policy) are applied by
/// [`HttpClient`] before the request reaches the backend.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// The HTTP method.
    pub method: HttpMethod,
    /// The fully-qualified request URL (including query string).
    pub url: String,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// Raw request body bytes (already serialized).
    pub body: Option<Vec<u8>>,
    /// Per-request timeout. The backend should abort the request after this duration.
    pub timeout: Duration,
}

/// An HTTP response returned by an [`HttpBackend`].
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Raw response body bytes.
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Returns the value of a response header (case-insensitive lookup).
    pub fn header(&self, name: &str) -> Option<&str> {
        let lower = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == lower)
            .map(|(_, v)| v.as_str())
    }
}

/// The replaceable transport contract.
///
/// Implementors are responsible only for sending a single request and returning the
/// raw response. Retries, authentication and serialization are handled by
/// [`HttpClient`], so a backend only needs to perform one network round-trip.
#[async_trait]
pub trait HttpBackend: Send + Sync + std::fmt::Debug {
    /// Sends a single HTTP request and returns the response.
    ///
    /// Network-level failures (connection refused, DNS, timeout) should be returned as
    /// [`ApifyClientError::Http`] or [`ApifyClientError::Timeout`]. A non-2xx HTTP
    /// status is *not* an error at this layer — return it as a normal [`HttpResponse`].
    async fn send(&self, request: HttpRequest) -> ApifyClientResult<HttpResponse>;
}

/// The default [`HttpBackend`] implementation, backed by [`reqwest`].
#[derive(Debug, Clone)]
pub struct ReqwestBackend {
    client: reqwest::Client,
}

impl ReqwestBackend {
    /// Creates a new backend with a default `reqwest::Client`.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Creates a backend wrapping a caller-provided `reqwest::Client`.
    ///
    /// Useful for sharing a connection pool or customizing proxy/TLS settings.
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpBackend for ReqwestBackend {
    async fn send(&self, request: HttpRequest) -> ApifyClientResult<HttpResponse> {
        let method = match request.method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Head => reqwest::Method::HEAD,
        };

        let mut builder = self
            .client
            .request(method, &request.url)
            .timeout(request.timeout);

        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }
        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        let response = builder.send().await?;
        let status = response.status().as_u16();

        let mut headers = HashMap::new();
        for (name, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.as_str().to_string(), v.to_string());
            }
        }

        let body = response.bytes().await?.to_vec();
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

/// Configuration for the retry/timeout behaviour of the [`HttpClient`].
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of *retries* (i.e. the request is attempted up to `max_retries + 1` times).
    pub max_retries: u32,
    /// Minimum delay between retries; doubled on each subsequent retry (exponential backoff).
    pub min_delay_between_retries: Duration,
    /// Overall per-request timeout budget. Each attempt's timeout grows but is capped here.
    pub timeout: Duration,
}

/// The orchestrating HTTP client shared by every resource client.
///
/// It owns the [`HttpBackend`], the optional API token, and the retry/timeout policy.
/// It is cheap to clone (everything is reference-counted) so each resource client can
/// hold its own handle.
#[derive(Debug, Clone)]
pub struct HttpClient {
    backend: Arc<dyn HttpBackend>,
    token: Option<String>,
    user_agent: String,
    retry: RetryConfig,
}

impl HttpClient {
    pub(crate) fn new(
        backend: Arc<dyn HttpBackend>,
        token: Option<String>,
        user_agent: String,
        retry: RetryConfig,
    ) -> Self {
        Self {
            backend,
            token,
            user_agent,
            retry,
        }
    }

    /// Sends `request` with authentication, the user-agent header, and the retry policy
    /// applied. Returns the first successful response, or the final error.
    pub async fn call(&self, mut request: HttpRequest) -> ApifyClientResult<HttpResponse> {
        // Inject auth + user-agent headers shared by every endpoint.
        request
            .headers
            .insert("User-Agent".to_string(), self.user_agent.clone());
        if let Some(token) = &self.token {
            request
                .headers
                .insert("Authorization".to_string(), format!("Bearer {token}"));
        }

        let method_str = request.method.as_str().to_string();
        let path = extract_path(&request.url);

        // The caller-supplied `request.timeout` is the per-endpoint base; it grows with each
        // attempt up to the client's overall timeout budget.
        let base_timeout = request.timeout;
        let mut delay = self.retry.min_delay_between_retries;
        // `saturating_add` so an extreme `max_retries` can't overflow the attempt count.
        let max_attempts = self.retry.max_retries.saturating_add(1);

        let mut attempt = 1;
        loop {
            // Grow per-attempt timeout with each attempt, capped at the overall budget.
            let mut attempt_request = request.clone();
            attempt_request.timeout = self.attempt_timeout(base_timeout, attempt);

            let outcome = match self.backend.send(attempt_request).await {
                Ok(response) => {
                    if response.status < MAX_SUCCESS_STATUS_CODE {
                        return Ok(response);
                    }
                    let api_error = build_api_error(&response, attempt, &method_str, &path);
                    let retryable = is_status_retryable(response.status);
                    (ApifyClientError::from(api_error), retryable)
                }
                Err(err) => {
                    let retryable = is_error_retryable(&err);
                    (err, retryable)
                }
            };

            let (error, retryable) = outcome;
            // Give up immediately on non-retryable errors or after the last attempt.
            if !retryable || attempt == max_attempts {
                return Err(error);
            }

            // Sleep with randomized exponential backoff before the next attempt. The backoff
            // doubles each retry (matching the reference client, which uses `async-retry` with
            // a factor of 2) and is capped at the overall request timeout so a single backoff
            // can never exceed the budget the whole request is allowed.
            sleep(randomized_delay(delay)).await;
            // `saturating_mul` mirrors the saturating arithmetic in `attempt_timeout`.
            delay = delay.saturating_mul(BACKOFF_FACTOR).min(self.retry.timeout);
            attempt += 1;
        }
    }

    /// Per-attempt timeout: `min(overall_timeout, base * 2^(attempt-1))`.
    ///
    /// The first attempt uses the per-endpoint `base` timeout; each retry doubles it so a
    /// slow-but-progressing connection gets more time, while never exceeding the client's
    /// overall timeout budget. Mirrors the reference clients.
    fn attempt_timeout(&self, base: Duration, attempt: u32) -> Duration {
        let scaled = base.saturating_mul(2u32.saturating_pow(attempt.saturating_sub(1)));
        scaled.min(self.retry.timeout)
    }

    pub(crate) fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Returns the token and user-agent, for endpoints (like log streaming) that must
    /// open a raw connection outside the buffered backend.
    pub(crate) fn stream_credentials(&self) -> (Option<String>, String) {
        (self.token.clone(), self.user_agent.clone())
    }
}

/// Returns the path + query portion of a URL, for error reporting.
fn extract_path(url: &str) -> Option<String> {
    // Find the start of the path after the scheme+host.
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    after_scheme
        .find('/')
        .map(|idx| after_scheme[idx..].to_string())
}

/// We retry `429` (rate limit) and `5xx` (internal server errors), matching the
/// reference client policy. Other `4xx` statuses are caller errors and are not retried.
fn is_status_retryable(status: u16) -> bool {
    status == RATE_LIMIT_EXCEEDED_STATUS_CODE || status >= MIN_SERVER_ERROR_STATUS_CODE
}

/// Only transport-level failures are retryable. Programming errors (serde, invalid
/// argument) and already-classified API errors are handled elsewhere, so they are not
/// retried here — matching the reference clients, which retry only network/timeout errors.
fn is_error_retryable(err: &ApifyClientError) -> bool {
    matches!(err, ApifyClientError::Http(_) | ApifyClientError::Timeout)
}

/// Parses the API error body (if present) into an [`ApiError`].
fn build_api_error(
    response: &HttpResponse,
    attempt: u32,
    method: &str,
    path: &Option<String>,
) -> ApiError {
    let parsed: Option<ApiErrorBody> = serde_json::from_slice(&response.body).ok();
    let (error_type, message, data) = match parsed {
        Some(body) => (
            body.error.error_type,
            body.error
                .message
                .unwrap_or_else(|| format!("Unexpected error with status {}", response.status)),
            body.error.data,
        ),
        None => {
            let raw = String::from_utf8_lossy(&response.body);
            let message = if raw.trim().is_empty() {
                format!("Unexpected error with status {}", response.status)
            } else {
                format!("Unexpected error: {raw}")
            };
            (None, message, None)
        }
    };

    ApiError {
        status_code: response.status,
        error_type,
        message,
        attempt,
        http_method: Some(method.to_string()),
        path: path.clone(),
        data,
    }
}

/// Returns a delay chosen randomly from the interval `[delay, 2*delay)`, matching the
/// exponential-backoff-with-jitter algorithm described in the API docs.
fn randomized_delay(delay: Duration) -> Duration {
    let base = delay.as_millis() as u64;
    if base == 0 {
        return delay;
    }
    let extra = next_jitter() % base;
    Duration::from_millis(base + extra)
}

/// A process-wide pseudo-random source for backoff jitter.
///
/// Backoff jitter does not need cryptographic quality, but it must be well-distributed and
/// uncorrelated across concurrent retries (otherwise many clients retry in lockstep). A
/// shared atomically-advanced SplitMix64 generator, seeded once from the clock, gives each
/// caller a distinct value without pulling in a heavyweight RNG dependency.
fn next_jitter() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(0);

    const GOLDEN_GAMMA: u64 = 0x9E3779B97F4A7C15;

    // Lazily seed from the clock on first use. A racing double-seed is harmless: both
    // candidate seeds are valid SplitMix64 stream starting points.
    if STATE.load(Ordering::Relaxed) == 0 {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(GOLDEN_GAMMA)
            | 1;
        let _ = STATE.compare_exchange(0, seed, Ordering::Relaxed, Ordering::Relaxed);
    }

    // SplitMix64: advance the shared state by the golden-ratio increment in a single atomic
    // read-modify-write (`fetch_add`) so concurrent callers each observe a distinct value —
    // a plain load-then-store could hand two racing retries the same number. Then scramble.
    let mut z = STATE
        .fetch_add(GOLDEN_GAMMA, Ordering::Relaxed)
        .wrapping_add(GOLDEN_GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Sleeps for the given duration (public crate-internal helper for poll loops).
pub(crate) async fn sleep_public(duration: Duration) {
    sleep(duration).await;
}

/// Sleeps for the given duration using the Tokio timer (the runtime `reqwest` requires).
async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}
