//! The HTTP layer of the client.
//!
//! The [`HttpBackend`] trait sends a single HTTP request and returns its response. The default
//! implementation, [`ReqwestBackend`], uses [`reqwest`]. Plug in another one with
//! [`ApifyClientBuilder::http_backend`](crate::ApifyClientBuilder::http_backend).
//!
//! The client wraps a backend in an internal `HttpClient` that adds what every endpoint shares:
//! authentication, the `User-Agent` header, query parameters, timeouts, and retries with
//! exponential backoff.

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
/// The header that carries the API token to the Apify API.
const AUTHORIZATION_HEADER: &str = "Authorization";
/// The header that carries the API token to an Actor's server. The platform removes it before the
/// request reaches the Actor, which keeps `Authorization` free for the Actor's own authentication.
const X_APIFY_AUTHORIZATION_HEADER: &str = "X-Apify-Authorization";
/// Multiplier applied to the delay between retries after each attempt.
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
    /// HTTP `PATCH`.
    Patch,
    /// HTTP `OPTIONS`.
    Options,
    /// HTTP `TRACE`.
    Trace,
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
            HttpMethod::Patch => "PATCH",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
        }
    }
}

/// An HTTP request ready to be sent by an [`HttpBackend`]. The client has already applied the auth
/// header, the user agent and the compression.
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
    /// Whether the backend follows redirects. When `false`, the backend must return a redirect
    /// response as it is: the request carries a token that must not follow a redirect to a host
    /// the server picked.
    pub follow_redirects: bool,
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

    /// Returns `true` for a success status, below `300`.
    pub fn is_success(&self) -> bool {
        self.status < MAX_SUCCESS_STATUS_CODE
    }

    /// Returns the body as text, replacing any invalid UTF-8 sequences.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// Parses the body as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> ApifyClientResult<T> {
        Ok(serde_json::from_slice(&self.body)?)
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

/// The transport that sends requests. An implementation performs one network round-trip per
/// request: the client handles retries, authentication and serialization.
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
    /// The default implementation buffers the whole response with [`send`](Self::send) and yields
    /// it as one chunk. [`ReqwestBackend`] overrides it to stream the body in constant memory.
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
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Options => reqwest::Method::OPTIONS,
        HttpMethod::Trace => reqwest::Method::TRACE,
    }
}

/// The default [`HttpBackend`] implementation, backed by [`reqwest`].
///
/// A `reqwest::Client` fixes its redirect policy when it is built, so the backend keeps a second
/// client that never follows redirects, for the requests with
/// [`follow_redirects`](HttpRequest::follow_redirects) set to `false`.
#[derive(Debug, Clone)]
pub struct ReqwestBackend {
    client: reqwest::Client,
    /// Used for requests that must not follow redirects.
    no_redirect_client: reqwest::Client,
}

impl ReqwestBackend {
    /// Creates a new backend with a default `reqwest::Client`.
    pub fn new() -> Self {
        Self::with_client(reqwest::Client::new())
    }

    /// Creates a backend wrapping a caller-provided `reqwest::Client`.
    ///
    /// Useful for sharing a connection pool or customizing proxy/TLS settings. The requests that
    /// must not follow redirects, such as those to an Actor's server, are sent by a default
    /// client that follows none: replace it with
    /// [`with_no_redirect_client`](Self::with_no_redirect_client) to configure it too.
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            no_redirect_client: default_no_redirect_client(),
        }
    }

    /// Replaces the client that sends the requests that must not follow redirects. It has to be
    /// built with `redirect(reqwest::redirect::Policy::none())`, or those requests would follow
    /// redirects.
    #[must_use]
    pub fn with_no_redirect_client(mut self, client: reqwest::Client) -> Self {
        self.no_redirect_client = client;
        self
    }

    /// The `reqwest` client that sends `request`, following its redirect setting.
    fn client_for(&self, request: &HttpRequest) -> &reqwest::Client {
        if request.follow_redirects {
            &self.client
        } else {
            &self.no_redirect_client
        }
    }
}

/// A default `reqwest::Client` that follows no redirects.
fn default_no_redirect_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("a default reqwest client without redirects builds, like `reqwest::Client::new`")
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

        let mut builder = self.client_for(&request).request(method, &request.url);
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

        // No `.timeout(request.timeout)`: reqwest's timeout covers reading the whole body, which
        // would cut off a long but active stream such as a live log. `HttpClient` bounds the wait
        // for the response and, unless the call opts out, the silence between chunks instead.
        let mut builder = self.client_for(&request).request(method, &request.url);
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
pub(crate) struct RetryConfig {
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

/// The header that carries the client's API token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum AuthorizationHeader {
    /// `Authorization`, which the Apify API reads.
    #[default]
    Authorization,
    /// `X-Apify-Authorization`, which Actor servers accept.
    XApifyAuthorization,
}

/// How the body of a request is compressed.
#[derive(Debug, Clone, Default)]
pub(crate) enum BodyCompression {
    /// With the client's compressor.
    #[default]
    ClientDefault,
    /// Not at all.
    Disabled,
    /// With this compressor, for this request only.
    With(Arc<dyn HttpCompressor>),
}

/// Options of a single [`HttpClient::call_with_options`].
#[derive(Debug, Clone, Default)]
pub(crate) struct CallOptions {
    /// Give up on the first transport timeout instead of retrying it. For a call that is not
    /// idempotent, such as pushing dataset items, whose request may have reached the API before
    /// the client gave up on it: a retry could then store the same data twice. Other errors are
    /// retried as usual.
    pub(crate) do_not_retry_timeouts: bool,
    /// How to compress the request body.
    pub(crate) compression: BodyCompression,
    /// The header that carries the API token.
    pub(crate) authorization_header: AuthorizationHeader,
    /// Return an error response as it is once no retry follows, instead of failing with an
    /// [`ApiError`]. A rate limit (`429`) then stays out of the [`Statistics`], which track the
    /// API's rate limits: only calls to the API fail on an error status.
    pub(crate) return_error_responses: bool,
    /// The statuses to retry, instead of the rate limit and the server errors.
    pub(crate) retry_status_codes: Option<&'static [u16]>,
    /// Give up on the first transport error, timeouts included, since the request may have
    /// reached the server before it failed.
    pub(crate) do_not_retry_transport_errors: bool,
    /// Return a redirect response as it is, since the request carries a token that must not
    /// follow a redirect to a host the server picked.
    pub(crate) do_not_follow_redirects: bool,
    /// For a streaming call: bound only the wait for the response to start, not the silence
    /// between two chunks of its body. For a server that can stay silent while it keeps the
    /// connection open.
    pub(crate) no_stream_inactivity_timeout: bool,
}

/// The orchestrating HTTP client shared by every resource client.
///
/// It owns the [`HttpBackend`], the optional API token, and the retry/timeout policy.
/// It is cheap to clone (everything is reference-counted) so each resource client can
/// hold its own handle.
#[derive(Debug, Clone)]
pub(crate) struct HttpClient {
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

    pub(crate) fn fail_if_poisoned(&self) -> ApifyClientResult<()> {
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
    /// Header names are compared case-insensitively. With
    /// [`AuthorizationHeader::XApifyAuthorization`], the `Authorization` header of the client,
    /// default headers included, is sent as `X-Apify-Authorization`; a request header of either
    /// name still wins.
    fn apply_headers(&self, request: &mut HttpRequest, authorization_header: AuthorizationHeader) {
        let mut merged: HashMap<String, String> = HashMap::new();
        merged.insert("User-Agent".to_string(), self.user_agent.clone());
        if let Some(token) = &self.token {
            merged.insert(AUTHORIZATION_HEADER.to_string(), format!("Bearer {token}"));
        }
        for (name, value) in &self.default_headers {
            set_header(&mut merged, name, value);
        }
        if authorization_header == AuthorizationHeader::XApifyAuthorization {
            move_header(
                &mut merged,
                AUTHORIZATION_HEADER,
                X_APIFY_AUTHORIZATION_HEADER,
            );
        }
        for (name, value) in std::mem::take(&mut request.headers) {
            set_header(&mut merged, &name, &value);
        }
        request.headers = merged;
    }

    /// Compresses the body of `request` once (not per attempt) with the compressor `compression`
    /// picks, when the body is large enough.
    fn compress_body(
        &self,
        request: &mut HttpRequest,
        compression: &BodyCompression,
    ) -> ApifyClientResult<()> {
        let compressor = match compression {
            BodyCompression::ClientDefault => self.compressor.as_ref(),
            BodyCompression::Disabled => return Ok(()),
            BodyCompression::With(compressor) => compressor.as_ref(),
        };
        if let Some(body) = request.body.take() {
            request.body = Some(maybe_compress(compressor, &mut request.headers, body)?);
        }
        Ok(())
    }

    /// The statistics of the calls made through this client.
    pub(crate) fn stats(&self) -> &Statistics {
        &self.stats
    }

    /// Sends `request` with authentication, the user-agent header, and the retry policy
    /// applied, as `options` adjust it. Returns the first successful response, or the final
    /// error. With [`CallOptions::return_error_responses`], the final error response is returned
    /// instead of the error.
    pub(crate) async fn call_with_options(
        &self,
        request: HttpRequest,
        options: CallOptions,
    ) -> ApifyClientResult<HttpResponse> {
        self.call_with_retries(request, &options).await
    }

    /// Opens a streaming connection through the configured [`HttpBackend`]'s
    /// [`HttpBackend::send_streaming`], with authentication, the user-agent header and the retry
    /// policy applied like [`call_with_options`](Self::call_with_options), but with no request
    /// compression. It retries until the response starts (an error status or a transport error
    /// before it opens), and never once the body is being read, since an open stream can't be
    /// replayed from the start.
    ///
    /// On a non-success status, reads the body (bounded by the same inactivity timeout as a
    /// successful body below) and returns the same structured [`ApiError`] [`call_with_options`](Self::call_with_options)
    /// would. On success, returns the body as a stream whose `.next()` must resolve within the
    /// request's resolved timeout each time — bounding silence (a stalled connection) without
    /// capping the stream's total duration, so a long-but-active stream (e.g. live log
    /// redirection) is never cut off. The same timeout also bounds the initial wait for the
    /// response to start.
    pub(crate) async fn call_streaming(
        &self,
        request: HttpRequest,
    ) -> ApifyClientResult<impl futures_util::Stream<Item = ApifyClientResult<Vec<u8>>>> {
        let options = CallOptions {
            compression: BodyCompression::Disabled,
            ..CallOptions::default()
        };
        Ok(self.call_streaming_response(request, &options).await?.body)
    }

    /// Opens a streaming connection like [`call_streaming`](Self::call_streaming), but honors
    /// `options` like [`call_with_options`](Self::call_with_options) does: with
    /// [`CallOptions::return_error_responses`] the final error response is returned, with its
    /// body unread, instead of the error. The body of the response is wrapped in the per-chunk
    /// inactivity timeout that the request's timeout sets, unless
    /// [`CallOptions::no_stream_inactivity_timeout`] says otherwise.
    pub(crate) async fn call_streaming_response(
        &self,
        request: HttpRequest,
        options: &CallOptions,
    ) -> ApifyClientResult<StreamingHttpResponse> {
        self.call_with_retries(request, options).await
    }

    /// The retry loop shared by the buffered and the streaming calls: sends `request` until an
    /// attempt succeeds, no retry follows its failure, or the attempts run out.
    async fn call_with_retries<R: AttemptResponse>(
        &self,
        mut request: HttpRequest,
        options: &CallOptions,
    ) -> ApifyClientResult<R> {
        self.fail_if_poisoned()?;
        self.stats.add_call();
        self.apply_headers(&mut request, options.authorization_header);

        // Compress the request body once, not per attempt, when it is large enough.
        self.compress_body(&mut request, &options.compression)?;

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
            let attempt_timeout = self.attempt_timeout(base_timeout, attempt);
            let mut attempt_request = request.clone();
            attempt_request.timeout = attempt_timeout;

            self.stats.add_request();
            // The error that ends the call, or `None` when a retry follows. A retry that follows
            // drops the failed response unread: reading the body of an error that is thrown away
            // would only delay the retry, or never end for a body that keeps streaming.
            let final_error = match R::send(self.backend.as_ref(), attempt_request, options).await {
                Ok(response) => {
                    let status = response.status();
                    if status < MAX_SUCCESS_STATUS_CODE {
                        return Ok(response);
                    }
                    if status == RATE_LIMIT_EXCEEDED_STATUS_CODE && !options.return_error_responses
                    {
                        self.stats.add_rate_limit_error(attempt);
                    }
                    // Give up immediately on a status a retry cannot fix, or after the last attempt.
                    let is_final = !is_status_retryable(status, options.retry_status_codes)
                        || attempt == max_attempts;
                    if !is_final {
                        None
                    } else if options.return_error_responses {
                        return Ok(response);
                    } else {
                        let buffered = response.into_buffered(attempt_timeout).await;
                        let api_error = build_api_error(&buffered, attempt, &method_str, &path);
                        Some(ApifyClientError::from(api_error))
                    }
                }
                Err(err) => {
                    let retryable = is_error_retryable(&err)
                        && !options.do_not_retry_transport_errors
                        && !(options.do_not_retry_timeouts
                            && matches!(err, ApifyClientError::Timeout));
                    // Give up immediately on a non-retryable error or after the last attempt.
                    (!retryable || attempt == max_attempts).then_some(err)
                }
            };
            if let Some(error) = final_error {
                return Err(error);
            }

            // The backoff doubles each retry and is capped at `timeout_max`.
            sleep(randomized_delay(delay)).await;
            delay = delay.saturating_mul(BACKOFF_FACTOR).min(self.tiers.max);
            attempt += 1;
        }
    }

    /// Per-attempt timeout: `min(timeout_max, base * 2^(attempt-1))`.
    ///
    /// The first attempt uses the `base` timeout of the request; each retry doubles it so a
    /// slow-but-progressing connection gets more time, while never exceeding the client's
    /// `timeout_max`. A request without a timeout ([`NO_TIMEOUT`]) stays without one.
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
}

/// A response that one attempt of a retried request yields: a buffered one, or a streaming one
/// whose body is still unread.
#[async_trait]
trait AttemptResponse: Sized + Send {
    /// Sends one attempt of `request` through `backend`.
    async fn send(
        backend: &dyn HttpBackend,
        request: HttpRequest,
        options: &CallOptions,
    ) -> ApifyClientResult<Self>;

    /// The HTTP status of the response.
    fn status(&self) -> u16;

    /// Reads the whole response within `timeout`, for the error of an error status.
    async fn into_buffered(self, timeout: Duration) -> HttpResponse;
}

#[async_trait]
impl AttemptResponse for HttpResponse {
    async fn send(
        backend: &dyn HttpBackend,
        request: HttpRequest,
        _options: &CallOptions,
    ) -> ApifyClientResult<Self> {
        backend.send(request).await
    }

    fn status(&self) -> u16 {
        self.status
    }

    async fn into_buffered(self, _timeout: Duration) -> HttpResponse {
        self
    }
}

#[async_trait]
impl AttemptResponse for StreamingHttpResponse {
    async fn send(
        backend: &dyn HttpBackend,
        request: HttpRequest,
        options: &CallOptions,
    ) -> ApifyClientResult<Self> {
        // The request timeout bounds the wait for the response to start and, unless the options
        // say otherwise, the silence between two chunks of the body: a stream that stays silent
        // for that long (e.g. a run that logs nothing for the `Long` tier) times out.
        // `Timeout::NoTimeout`, set with a resource client's `with_timeout`, turns that off.
        let timeout = request.timeout;
        let response = tokio::time::timeout(timeout, backend.send_streaming(request))
            .await
            .map_err(|_| ApifyClientError::Timeout)??;
        let body = if options.no_stream_inactivity_timeout {
            response.body
        } else {
            with_inactivity_timeout(response.body, timeout)
        };
        Ok(StreamingHttpResponse {
            status: response.status,
            headers: response.headers,
            body,
        })
    }

    fn status(&self) -> u16 {
        self.status
    }

    /// The error body arrives as a stream here too; reading it lets the failure carry the same
    /// structured `ApiError` (type/message/data) as every buffered endpoint, rather than losing
    /// the body to a generic transport error. A body that fails to read (e.g. the connection
    /// drops mid-response) still surfaces the status code, just without a parsed message.
    async fn into_buffered(self, timeout: Duration) -> HttpResponse {
        HttpResponse {
            status: self.status,
            headers: self.headers,
            body: collect_body(with_inactivity_timeout(self.body, timeout)).await,
        }
    }
}

/// Wraps a streaming body so each chunk must arrive within `timeout` of the previous one (or of
/// the stream opening), erroring with [`ApifyClientError::Timeout`] otherwise — an inactivity
/// timeout, not a cap on the stream's total duration. Used by the streaming attempts of
/// [`HttpClient`], for the body of a response and for the body of an error response they read.
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

/// Drains a streaming body into a single buffer, stopping at the first error. Used only for a
/// non-success response's (typically small) error body, so buffering it is fine — unlike a
/// successful body, which callers stream incrementally.
async fn collect_body(
    mut body: std::pin::Pin<
        Box<dyn futures_util::Stream<Item = ApifyClientResult<Vec<u8>>> + Send>,
    >,
) -> Vec<u8> {
    let mut buf = Vec::new();
    while let Some(chunk) = futures_util::StreamExt::next(&mut body).await {
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

/// Renames header `from` to `to`, whatever the casing of `from`, replacing any `to` header. Does
/// nothing when there is no `from` header.
fn move_header(headers: &mut HashMap<String, String>, from: &str, to: &str) {
    let Some(name) = headers
        .keys()
        .find(|name| name.eq_ignore_ascii_case(from))
        .cloned()
    else {
        return;
    };
    if let Some(value) = headers.remove(&name) {
        set_header(headers, to, &value);
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

/// Retries `429` (rate limit) and `5xx`, unless the request lists its own `retry_status_codes`.
/// Other `4xx` statuses are caller errors and are not retried.
fn is_status_retryable(status: u16, retry_status_codes: Option<&[u16]>) -> bool {
    match retry_status_codes {
        Some(codes) => codes.contains(&status),
        None => status == RATE_LIMIT_EXCEEDED_STATUS_CODE || status >= MIN_SERVER_ERROR_STATUS_CODE,
    }
}

/// Only transport failures (network errors and timeouts) are retried. Serialization errors,
/// invalid arguments and API errors are not.
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

/// Returns a delay chosen randomly from the interval `[delay, 2*delay)`, the jitter of the API's
/// recommended exponential backoff. Shared with the retries of `batch_add_requests`.
pub(crate) fn randomized_delay(delay: Duration) -> Duration {
    let base = delay.as_millis() as u64;
    if base == 0 {
        return delay;
    }
    let extra = next_jitter() % base;
    Duration::from_millis(base + extra)
}

/// A process-wide SplitMix64 source for backoff jitter. It is not cryptographic, but it gives
/// concurrent retries distinct values so they don't run in lockstep, without an RNG dependency.
fn next_jitter() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(0);

    const GOLDEN_GAMMA: u64 = 0x9E3779B97F4A7C15;

    // Seed from the clock on first use. Two racing seeds are both valid starting points.
    if STATE.load(Ordering::Relaxed) == 0 {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(GOLDEN_GAMMA)
            | 1;
        let _ = STATE.compare_exchange(0, seed, Ordering::Relaxed, Ordering::Relaxed);
    }

    // A single atomic `fetch_add` gives each concurrent caller a distinct value.
    let mut z = STATE
        .fetch_add(GOLDEN_GAMMA, Ordering::Relaxed)
        .wrapping_add(GOLDEN_GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Sleeps for the given duration.
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
