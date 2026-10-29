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

use crate::error::{
    ApiError, ApiErrorBody, ApifyClientError, ApifyClientResult, MIN_SERVER_ERROR_STATUS_CODE,
    RATE_LIMIT_EXCEEDED_STATUS_CODE,
};
use crate::http_compressors::{maybe_compress, HttpCompressor};
use crate::statistics::Statistics;
use crate::timeouts::{Timeout, TimeoutTiers, NO_TIMEOUT};

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
    /// Per-request timeout. The backend should abort the request after this duration, unless it
    /// is [`NO_TIMEOUT`]: such a request may run for as long as it
    /// takes and the backend must not impose a timeout of its own.
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

/// A streaming counterpart of [`HttpResponse`]: the status and headers, available as soon as
/// the response starts, and the body delivered incrementally instead of as one buffered `Vec<u8>`.
pub struct StreamingHttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// The body, as a stream of chunks.
    pub body:
        std::pin::Pin<Box<dyn futures_util::Stream<Item = ApifyClientResult<Vec<u8>>> + Send>>,
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

    /// Like [`send`](Self::send), but for a request whose body the caller wants to process
    /// incrementally (a live log or a large export) instead of fully buffered.
    ///
    /// The default implementation just buffers the whole response via [`send`](Self::send) and
    /// yields it as a single chunk: correct for a custom backend that does not override this
    /// (streaming still works end-to-end, just without a memory-usage benefit), while the
    /// default [`ReqwestBackend`] overrides it for real constant-memory streaming through
    /// `reqwest`'s own chunked body reader.
    async fn send_streaming(
        &self,
        request: HttpRequest,
    ) -> ApifyClientResult<StreamingHttpResponse> {
        let response = self.send(request).await?;
        Ok(StreamingHttpResponse {
            status: response.status,
            headers: response.headers,
            body: Box::pin(futures_util::stream::once(async move { Ok(response.body) })),
        })
    }
}

/// Maps this crate's [`HttpMethod`] to `reqwest`'s, shared by [`ReqwestBackend`]'s buffered and
/// streaming sends.
fn to_reqwest_method(method: HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
    }
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
        let method = to_reqwest_method(request.method);

        let mut builder = self.client.request(method, &request.url);
        if request.timeout != NO_TIMEOUT {
            builder = builder.timeout(request.timeout);
        }

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

    async fn send_streaming(
        &self,
        request: HttpRequest,
    ) -> ApifyClientResult<StreamingHttpResponse> {
        let method = to_reqwest_method(request.method);

        // Deliberately no `.timeout(request.timeout)` here: reqwest's per-request timeout bounds
        // the *entire* request including reading the whole body (it wraps the body stream in a
        // `TotalTimeoutBody`), which would cut off a long-but-active stream (e.g. live log
        // redirection). `HttpClient::call_streaming` applies its own response-start and
        // per-chunk-inactivity bounds instead, uniformly across backends.
        let mut builder = self.client.request(method, &request.url);
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

        let byte_stream = response.bytes_stream();
        let body = Box::pin(futures_util::StreamExt::map(byte_stream, |chunk| {
            chunk.map(|b| b.to_vec()).map_err(ApifyClientError::from)
        }));
        Ok(StreamingHttpResponse {
            status,
            headers,
            body,
        })
    }
}

/// Configuration for the retry behaviour of the [`HttpClient`].
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of *retries* (i.e. the request is attempted up to `max_retries + 1` times).
    pub max_retries: u32,
    /// Minimum delay between retries; doubled on each subsequent retry (exponential backoff).
    pub min_delay_between_retries: Duration,
}

/// Everything an [`HttpClient`] is built from.
pub(crate) struct HttpClientConfig {
    pub(crate) backend: Arc<dyn HttpBackend>,
    pub(crate) token: Option<String>,
    pub(crate) user_agent: String,
    pub(crate) retry: RetryConfig,
    pub(crate) tiers: TimeoutTiers,
    pub(crate) compressor: Arc<dyn HttpCompressor>,
    /// Extra headers sent with every request. They win over the built-in `User-Agent` and
    /// `Authorization` headers, and a request's own headers win over them.
    pub(crate) default_headers: Vec<(String, String)>,
    pub(crate) stats: Arc<Statistics>,
}

/// Options of a single [`HttpClient::call_with_options`].
#[derive(Debug, Clone, Copy, Default)]
pub struct CallOptions {
    /// Give up on the first transport timeout instead of retrying it. For a call that is not
    /// idempotent, such as pushing dataset items, whose request may have reached the API before
    /// the client gave up on it: a retry could then store the same data twice. Other errors are
    /// retried as usual.
    pub do_not_retry_timeouts: bool,
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
    tiers: TimeoutTiers,
    compressor: Arc<dyn HttpCompressor>,
    default_headers: Vec<(String, String)>,
    stats: Arc<Statistics>,
    /// Set when the resource client holding this handle was addressed with an invalid ID. Every
    /// call then fails with [`ApifyClientError::InvalidArgument`] before reaching the backend.
    /// The handle is cloned down to every client chained from that resource, so the failure
    /// follows them.
    poison: Option<Arc<str>>,
}

impl HttpClient {
    pub(crate) fn new(config: HttpClientConfig) -> Self {
        Self {
            backend: config.backend,
            token: config.token,
            user_agent: config.user_agent,
            retry: config.retry,
            tiers: config.tiers,
            compressor: config.compressor,
            default_headers: config.default_headers,
            stats: config.stats,
            poison: None,
        }
    }

    /// Returns a handle that fails every call with [`ApifyClientError::InvalidArgument`] carrying
    /// `message`.
    pub(crate) fn poisoned(mut self, message: String) -> Self {
        self.poison = Some(Arc::from(message));
        self
    }

    fn fail_if_poisoned(&self) -> ApifyClientResult<()> {
        match &self.poison {
            Some(message) => Err(ApifyClientError::InvalidArgument(message.to_string())),
            None => Ok(()),
        }
    }

    /// Resolves a [`Timeout`] to the duration of the first attempt of a request, capped at the
    /// client's `timeout_max`.
    pub(crate) fn resolve_timeout(&self, timeout: Timeout) -> Duration {
        self.tiers.resolve(timeout)
    }

    /// Resolves the timeout of a request that asks the API to hold its response for
    /// `wait_for_finish_secs` (see [`TimeoutTiers::resolve_with_wait_hold`]).
    pub(crate) fn resolve_timeout_with_wait_hold(
        &self,
        tier: Timeout,
        wait_for_finish_secs: Option<i64>,
    ) -> Duration {
        self.tiers.resolve(
            self.tiers
                .resolve_with_wait_hold(tier, wait_for_finish_secs),
        )
    }

    /// Adds the headers every request carries: the built-in `User-Agent` and `Authorization`,
    /// overridden by the client's default headers, overridden by the request's own headers.
    /// Header names are compared case-insensitively.
    fn apply_headers(&self, request: &mut HttpRequest) {
        let mut merged: HashMap<String, String> = HashMap::new();
        merged.insert("User-Agent".to_string(), self.user_agent.clone());
        if let Some(token) = &self.token {
            merged.insert("Authorization".to_string(), format!("Bearer {token}"));
        }
        for (name, value) in &self.default_headers {
            set_header(&mut merged, name, value);
        }
        for (name, value) in std::mem::take(&mut request.headers) {
            set_header(&mut merged, &name, &value);
        }
        request.headers = merged;
    }

    /// The statistics of the calls made through this client.
    pub(crate) fn stats(&self) -> &Statistics {
        &self.stats
    }

    /// Sends `request` with authentication, the user-agent header, and the retry policy
    /// applied. Returns the first successful response, or the final error.
    pub async fn call(&self, request: HttpRequest) -> ApifyClientResult<HttpResponse> {
        self.call_with_options(request, CallOptions::default())
            .await
    }

    /// Like [`call`](Self::call), with the given per-call [`CallOptions`].
    pub async fn call_with_options(
        &self,
        mut request: HttpRequest,
        options: CallOptions,
    ) -> ApifyClientResult<HttpResponse> {
        self.fail_if_poisoned()?;
        self.stats.add_call();
        self.apply_headers(&mut request);

        // Compress the request body once (not per attempt) when it is large enough, mirroring the
        // reference client. The API accepts both brotli- and gzip-encoded request bodies.
        if let Some(body) = request.body.take() {
            request.body = Some(maybe_compress(
                self.compressor.as_ref(),
                &mut request.headers,
                body,
            )?);
        }

        let method_str = request.method.as_str().to_string();
        let path = extract_path(&request.url);

        // The caller-supplied `request.timeout` is the base of the request's tier; it grows with
        // each attempt up to the client's `timeout_max`.
        let base_timeout = request.timeout;
        let mut delay = self.retry.min_delay_between_retries;
        // `saturating_add` so an extreme `max_retries` can't overflow the attempt count.
        let max_attempts = self.retry.max_retries.saturating_add(1);

        let mut attempt = 1;
        loop {
            // Grow the timeout of the attempt with each attempt, up to `timeout_max`.
            let mut attempt_request = request.clone();
            attempt_request.timeout = self.attempt_timeout(base_timeout, attempt);

            self.stats.add_request();
            let outcome = match self.backend.send(attempt_request).await {
                Ok(response) => {
                    if response.status < MAX_SUCCESS_STATUS_CODE {
                        return Ok(response);
                    }
                    if response.status == RATE_LIMIT_EXCEEDED_STATUS_CODE {
                        self.stats.add_rate_limit_error(attempt);
                    }
                    let api_error = build_api_error(&response, attempt, &method_str, &path);
                    let retryable = is_status_retryable(response.status);
                    (ApifyClientError::from(api_error), retryable)
                }
                Err(err) => {
                    let retryable = is_error_retryable(&err)
                        && !(options.do_not_retry_timeouts
                            && matches!(err, ApifyClientError::Timeout));
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
            // a factor of 2) and is capped at `timeout_max`.
            sleep(randomized_delay(delay)).await;
            // `saturating_mul` mirrors the saturating arithmetic in `attempt_timeout`.
            delay = delay.saturating_mul(BACKOFF_FACTOR).min(self.tiers.max);
            attempt += 1;
        }
    }

    /// Per-attempt timeout: `min(timeout_max, base * 2^(attempt-1))`.
    ///
    /// The first attempt uses the `base` timeout of the request; each retry doubles it so a
    /// slow-but-progressing connection gets more time, while never exceeding the client's
    /// `timeout_max`. A request without a timeout ([`NO_TIMEOUT`]) stays without one. Mirrors the
    /// reference clients.
    fn attempt_timeout(&self, base: Duration, attempt: u32) -> Duration {
        if base == NO_TIMEOUT {
            return NO_TIMEOUT;
        }
        let scaled = base.saturating_mul(2u32.saturating_pow(attempt.saturating_sub(1)));
        scaled.min(self.tiers.max)
    }

    pub(crate) fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Sends `request` through the configured [`HttpBackend`]'s [`HttpBackend::send_streaming`],
    /// with authentication and the user-agent header applied like [`call`](Self::call), but with
    /// no retries and no request compression: matches the reference clients, which do not retry
    /// an open stream (a log redirection or a large export already in flight can't be safely
    /// replayed from the start).
    ///
    /// On a non-success status, reads the body (bounded by the same inactivity timeout as a
    /// successful body below) and returns the same structured [`ApiError`] [`call`](Self::call)
    /// would. On success, returns the body as a stream whose `.next()` must resolve within the
    /// request's resolved timeout each time — bounding silence (a stalled connection) without
    /// capping the stream's total duration, so a long-but-active stream (e.g. live log
    /// redirection) is never cut off. The same timeout also bounds the initial wait for the
    /// response to start.
    pub(crate) async fn call_streaming(
        &self,
        mut request: HttpRequest,
    ) -> ApifyClientResult<impl futures_util::Stream<Item = ApifyClientResult<Vec<u8>>>> {
        self.fail_if_poisoned()?;
        self.stats.add_call();
        self.stats.add_request();
        self.apply_headers(&mut request);

        let method_str = request.method.as_str().to_string();
        let path = extract_path(&request.url);
        // The request timeout is both the response-start wait and the per-chunk inactivity
        // window, so a stream that stays silent for that long (e.g. a run that logs nothing for
        // the `Long` tier) times out. `Timeout::NoTimeout`, set with a resource client's
        // `with_timeout`, turns that off.
        let budget = request.timeout;

        let response = tokio::time::timeout(budget, self.backend.send_streaming(request))
            .await
            .map_err(|_| ApifyClientError::Timeout)??;

        if response.status < MAX_SUCCESS_STATUS_CODE {
            return Ok(with_inactivity_timeout(response.body, budget));
        }

        // The error body arrives as a stream here too; read it (under the same inactivity
        // budget) so the failure carries the same structured `ApiError` (type/message/data) as
        // every buffered endpoint, rather than losing the body to a generic transport error. A
        // body that fails to read (e.g. the connection drops mid-response) still surfaces the
        // status code, just without a parsed message.
        let body = collect_with_timeout(response.body, budget).await;
        if response.status == RATE_LIMIT_EXCEEDED_STATUS_CODE {
            self.stats.add_rate_limit_error(1);
        }
        let http_response = HttpResponse {
            status: response.status,
            headers: response.headers,
            body,
        };
        let api_error = build_api_error(&http_response, 1, &method_str, &path);
        Err(ApifyClientError::from(api_error))
    }
}

/// Wraps a streaming body so each chunk must arrive within `timeout` of the previous one (or of
/// the stream opening), erroring with [`ApifyClientError::Timeout`] otherwise — an inactivity
/// timeout, not a cap on the stream's total duration. Shared by
/// [`HttpClient::call_streaming`]'s success and error-body-reading paths.
///
/// A timeout yields exactly one `Err(Timeout)` and then ends the stream (further polls return
/// `None`), rather than retrying the same read forever: the underlying body is dropped once it
/// has stalled, so a caller that logs the error and keeps draining (instead of stopping on the
/// first `Err`, as `?`-style consumption does) still sees the stream terminate.
fn with_inactivity_timeout(
    body: std::pin::Pin<Box<dyn futures_util::Stream<Item = ApifyClientResult<Vec<u8>>> + Send>>,
    timeout: Duration,
) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = ApifyClientResult<Vec<u8>>> + Send>> {
    enum State {
        Active(
            std::pin::Pin<Box<dyn futures_util::Stream<Item = ApifyClientResult<Vec<u8>>> + Send>>,
        ),
        Stalled,
    }

    Box::pin(futures_util::stream::unfold(
        State::Active(body),
        move |state| async move {
            let State::Active(mut body) = state else {
                return None;
            };
            match tokio::time::timeout(timeout, futures_util::StreamExt::next(&mut body)).await {
                Ok(Some(item)) => Some((item, State::Active(body))),
                Ok(None) => None,
                Err(_) => Some((Err(ApifyClientError::Timeout), State::Stalled)),
            }
        },
    ))
}

/// Drains a streaming body into a single buffer, under the same per-chunk inactivity timeout as
/// [`with_inactivity_timeout`]. Used only for a non-success response's (typically small) error
/// body, so buffering it is fine — unlike a successful body, which callers stream incrementally.
async fn collect_with_timeout(
    body: std::pin::Pin<Box<dyn futures_util::Stream<Item = ApifyClientResult<Vec<u8>>> + Send>>,
    timeout: Duration,
) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut timed = with_inactivity_timeout(body, timeout);
    while let Some(chunk) = futures_util::StreamExt::next(&mut timed).await {
        match chunk {
            Ok(bytes) => buf.extend(bytes),
            Err(_) => break,
        }
    }
    buf
}

/// Sets header `name` to `value`, replacing any existing header of the same name regardless of
/// its casing.
fn set_header(headers: &mut HashMap<String, String>, name: &str, value: &str) {
    headers.retain(|existing, _| !existing.eq_ignore_ascii_case(name));
    headers.insert(name.to_string(), value.to_string());
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
///
/// `pub(crate)` so other retrying call sites (e.g. `RequestQueueClient::batch_add_requests`'s
/// unprocessed-request retries) can reuse the same jitter source instead of duplicating it.
pub(crate) fn randomized_delay(delay: Duration) -> Duration {
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    /// After one inactivity timeout, [`with_inactivity_timeout`] must end the stream rather than
    /// retrying the stalled read forever: a chunk that would only arrive after the stall must
    /// never be yielded, and a third poll past the timeout error must return `None`.
    #[tokio::test(start_paused = true)]
    async fn with_inactivity_timeout_ends_after_one_stall() {
        let timeout = Duration::from_millis(10);
        let body: std::pin::Pin<
            Box<dyn futures_util::Stream<Item = ApifyClientResult<Vec<u8>>> + Send>,
        > = Box::pin(futures_util::stream::unfold(0u8, move |n| async move {
            match n {
                0 => Some((Ok(b"chunk".to_vec()), 1)),
                // Stall well past the inactivity timeout before the stream would otherwise
                // continue, simulating a connection that goes quiet.
                1 => {
                    tokio::time::sleep(timeout * 10).await;
                    Some((Ok(b"should never be seen".to_vec()), 2))
                }
                _ => None,
            }
        }));

        let mut timed = with_inactivity_timeout(body, timeout);

        let first = timed.next().await;
        assert!(
            matches!(&first, Some(Ok(b)) if b == b"chunk"),
            "expected the first chunk, got {first:?}"
        );

        let second = timed.next().await;
        assert!(
            matches!(second, Some(Err(ApifyClientError::Timeout))),
            "expected a timeout error after the stall, got {second:?}"
        );

        let third = timed.next().await;
        assert!(
            third.is_none(),
            "the stream must end after the inactivity timeout instead of retrying forever, got {third:?}"
        );
    }
}
