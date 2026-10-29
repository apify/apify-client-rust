//! Shared internal machinery for resource clients.
//!
//! [`ResourceContext`] holds the resolved URL and the [`HttpClient`] for a single
//! resource (or sub-resource). The free functions in this module implement the CRUD
//! and wait-for-finish primitives once, so that every resource client stays small and
//! consistent (the DRY principle).

use std::collections::HashMap;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::common::{
    catch_not_found, parse_data_envelope, to_path_segment, to_safe_id, PaginationList, QueryParams,
};
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, HttpMethod, HttpRequest, HttpResponse};
use crate::timeouts::Timeout;

/// How long to wait between polls while waiting for a run/build to finish.
const WAIT_FOR_FINISH_POLL_INTERVAL: Duration = Duration::from_millis(250);
/// The `waitForFinish` value (in seconds) sent on each poll; the API caps server-side
/// waiting at 60 seconds, so we poll in chunks of this size.
const WAIT_FOR_FINISH_REQUEST_SECS: i64 = 60;
/// How long a run/build may keep answering `404` before [`wait_for_finish`] gives up and returns
/// a not-found error, instead of polling forever. A brand-new run/build can briefly 404 while the
/// write that created it has not yet reached the replica a poll happens to land on; a success
/// within this window resets it. Matches the reference clients' `DEFAULT_WAIT_WHEN_JOB_NOT_EXIST`.
/// Without this bound, a run/build ID that was simply never valid (e.g. a typo, or the resource
/// was deleted) would poll indefinitely when `wait_secs` is `None`.
const NOT_FOUND_GRACE_PERIOD: Duration = Duration::from_secs(3);
/// Path segment standing in for an ID that failed validation. The URL is never requested:
/// the [`HttpClient`] of such a context is poisoned and fails every call first.
const INVALID_ID_PLACEHOLDER: &str = "invalid-id";

/// The resolved context for a resource client: its base URL and the shared HTTP client.
#[derive(Debug, Clone)]
pub(crate) struct ResourceContext {
    pub(crate) http: HttpClient,
    /// The fully-qualified base URL of the resource, e.g. `https://api.apify.com/v2/actors/ID`.
    pub(crate) url: String,
    /// Parameters inherited from a parent resource (e.g. `status` on `actor.last_run()`).
    pub(crate) base_params: QueryParams,
    /// Origin (scheme + host) the API is reached through, e.g. `https://api.apify.com`.
    pub(crate) api_origin: String,
    /// Origin used to build public, shareable URLs (defaults to `api_origin`).
    pub(crate) public_origin: String,
    /// Whether the client names its resource by ID. A client chained from a parent, such as
    /// `run.dataset()`, has none, so a `404` from its path cannot tell a missing parent from a
    /// missing sub-resource (see [`catch_not_found_for`]).
    pub(crate) has_id: bool,
    /// Timeout set with the client's `with_timeout`, replacing the tier a method defaults to.
    pub(crate) timeout: Option<Timeout>,
}

impl ResourceContext {
    /// Creates a context for a collection endpoint: `{base}/{resource_path}`.
    pub(crate) fn collection(http: HttpClient, base_url: &str, resource_path: &str) -> Self {
        Self::new(http, format!("{base_url}/{resource_path}"), base_url, false)
    }

    /// Creates a context for a single resource: `{base}/{resource_path}/{safe_id}`.
    ///
    /// An ID that is empty or a dot segment cannot be placed in a URL path without changing which
    /// endpoint it addresses. Such a context keeps working as a handle, but every request through
    /// it, or through a client chained from it, fails with
    /// [`ApifyClientError::InvalidArgument`](crate::error::ApifyClientError::InvalidArgument).
    pub(crate) fn single(http: HttpClient, base_url: &str, resource_path: &str, id: &str) -> Self {
        let (http, segment) = match to_path_segment(&to_safe_id(id)) {
            Ok(segment) => (http, segment),
            Err(err) => (
                http.poisoned(err.to_string()),
                INVALID_ID_PLACEHOLDER.to_string(),
            ),
        };
        Self::new(
            http,
            format!("{base_url}/{resource_path}/{segment}"),
            base_url,
            true,
        )
    }

    /// Internal constructor that derives the API origin from `base_url`. The public origin
    /// defaults to the API origin and can be overridden with [`with_public_origin`].
    fn new(http: HttpClient, url: String, base_url: &str, has_id: bool) -> Self {
        let api_origin = origin_of(base_url);
        Self {
            http,
            url,
            base_params: QueryParams::new(),
            public_origin: api_origin.clone(),
            api_origin,
            has_id,
            timeout: None,
        }
    }

    /// Overrides the origin used when building public URLs.
    pub(crate) fn with_public_origin(mut self, public_origin: &str) -> Self {
        self.public_origin = origin_of(public_origin);
        self
    }

    /// Returns this resource's URL with an optional extra path segment appended.
    pub(crate) fn url(&self, sub_path: Option<&str>) -> String {
        match sub_path {
            Some(path) => format!("{}/{path}", self.url),
            None => self.url.clone(),
        }
    }

    /// Builds the public (shareable) form of this resource's URL with an optional extra
    /// path segment, swapping the API origin for the configured public origin.
    pub(crate) fn public_url(&self, sub_path: Option<&str>) -> String {
        let api_url = self.url(sub_path);
        if self.public_origin == self.api_origin {
            return api_url;
        }
        match api_url.strip_prefix(&self.api_origin) {
            Some(rest) => format!("{}{rest}", self.public_origin),
            None => api_url,
        }
    }

    /// Merges the inherited base params with per-call params into a final query string.
    pub(crate) fn merged_params(&self, params: &QueryParams) -> QueryParams {
        let mut merged = self.base_params.clone();
        merged.extend(params);
        merged
    }

    /// The timeout of a request that defaults to `tier`: the client's own override if it has one,
    /// resolved to the duration of the first attempt.
    pub(crate) fn timeout(&self, tier: Timeout) -> Duration {
        self.http.resolve_timeout(self.timeout.unwrap_or(tier))
    }

    /// Like [`timeout`](Self::timeout), for a request that asks the API to hold its response for
    /// `wait_for_finish_secs`: the default tier gets the hold added, while a timeout set with
    /// `with_timeout` is used as given.
    pub(crate) fn timeout_with_wait_hold(
        &self,
        tier: Timeout,
        wait_for_finish_secs: Option<i64>,
    ) -> Duration {
        match self.timeout {
            Some(timeout) => self.http.resolve_timeout(timeout),
            None => self
                .http
                .resolve_timeout_with_wait_hold(tier, wait_for_finish_secs),
        }
    }

    /// Sends a request with no body or extra headers.
    pub(crate) async fn send(
        &self,
        method: HttpMethod,
        url: String,
        timeout: Duration,
    ) -> ApifyClientResult<HttpResponse> {
        self.send_with(method, url, HashMap::new(), None, timeout)
            .await
    }

    /// Sends a request through the shared [`HttpClient`].
    pub(crate) async fn send_with(
        &self,
        method: HttpMethod,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        timeout: Duration,
    ) -> ApifyClientResult<HttpResponse> {
        self.http
            .call(HttpRequest {
                method,
                url,
                headers,
                body,
                timeout,
            })
            .await
    }

    /// The URL of `sub_path` with the inherited and per-call query parameters applied.
    fn url_with_params(&self, sub_path: Option<&str>, params: &QueryParams) -> String {
        self.merged_params(params).apply_to_url(&self.url(sub_path))
    }
}

/// Generates the public `with_timeout` method of a resource client holding a [`ResourceContext`]
/// in its `ctx` field.
macro_rules! impl_with_timeout {
    ($client:ident) => {
        impl $client {
            /// Sets the timeout of every request this client sends, replacing the tier each
            /// method defaults to (see [`Timeout`](crate::Timeout)).
            ///
            /// The timeout applies to each attempt of a request separately and doubles with every
            /// retry, up to the client's `timeout_max`. A longer timeout is lowered to that cap.
            /// Clients obtained from this one, such as `run.dataset()`, do not inherit it.
            #[must_use]
            pub fn with_timeout(mut self, timeout: $crate::Timeout) -> Self {
                self.ctx.timeout = Some(timeout);
                self
            }
        }
    };
}
pub(crate) use impl_with_timeout;

/// Extracts the origin (`scheme://host[:port]`) from a URL, dropping any path.
fn origin_of(url: &str) -> String {
    match url.split_once("://") {
        Some((scheme, rest)) => {
            let host = rest.split('/').next().unwrap_or(rest);
            format!("{scheme}://{host}")
        }
        None => url.split('/').next().unwrap_or(url).to_string(),
    }
}

impl QueryParams {
    /// Appends all pairs from `other` to `self`.
    pub(crate) fn extend(&mut self, other: &QueryParams) {
        for (k, v) in other.pairs_ref() {
            self.push_raw(k.clone(), v.clone());
        }
    }
}

/// Header map for a request with a JSON body.
fn json_headers() -> HashMap<String, String> {
    HashMap::from([("Content-Type".to_string(), "application/json".to_string())])
}

/// Swallows a `404` and returns `Ok(None)` only when the client names its resource by ID. A
/// chained client without one, such as `run.dataset()`, requests a path where a `404` can mean
/// either the parent or the default sub-resource is missing, and the response cannot tell the two
/// apart, so the error propagates.
pub(crate) fn catch_not_found_for<T>(
    ctx: &ResourceContext,
    result: ApifyClientResult<T>,
) -> ApifyClientResult<Option<T>> {
    if ctx.has_id {
        catch_not_found(result)
    } else {
        result.map(Some)
    }
}

/// A `GET` that unwraps the `data` envelope and maps `404` to `None` (see
/// [`catch_not_found_for`]).
pub(crate) async fn get_resource<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    tier: Timeout,
) -> ApifyClientResult<Option<T>> {
    let result = get_resource_required(ctx, sub_path, params, tier).await;
    catch_not_found_for(ctx, result)
}

/// A `GET` that unwraps the `data` envelope and propagates errors (including `404`).
pub(crate) async fn get_resource_required<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    tier: Timeout,
) -> ApifyClientResult<T> {
    let url = ctx.url_with_params(sub_path, params);
    let response = ctx.send(HttpMethod::Get, url, ctx.timeout(tier)).await?;
    parse_data_envelope(&response.body)
}

/// A `PUT` with a JSON body that unwraps the `data` envelope from the response.
pub(crate) async fn update_resource<B: Serialize, T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    body: &B,
    tier: Timeout,
) -> ApifyClientResult<T> {
    let url = ctx.url_with_params(sub_path, &QueryParams::new());
    let response = ctx
        .send_with(
            HttpMethod::Put,
            url,
            json_headers(),
            Some(serde_json::to_vec(body)?),
            ctx.timeout(tier),
        )
        .await?;
    parse_data_envelope(&response.body)
}

/// A `DELETE` that maps `404` to a successful no-op (see [`catch_not_found_for`]).
pub(crate) async fn delete_resource(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    tier: Timeout,
) -> ApifyClientResult<()> {
    let url = ctx.url_with_params(sub_path, &QueryParams::new());
    let result = ctx.send(HttpMethod::Delete, url, ctx.timeout(tier)).await;
    catch_not_found_for(ctx, result)?;
    Ok(())
}

/// A `GET` that returns a paginated list (`data` envelope wrapping `{ items, total, ... }`).
pub(crate) async fn list_resource<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    tier: Timeout,
) -> ApifyClientResult<PaginationList<T>> {
    get_resource_required(ctx, sub_path, params, tier).await
}

/// A `POST` with a JSON body that unwraps the `data` envelope (used for `create`).
pub(crate) async fn create_resource<B: Serialize, T: DeserializeOwned>(
    ctx: &ResourceContext,
    params: &QueryParams,
    body: &B,
    tier: Timeout,
) -> ApifyClientResult<T> {
    let url = ctx.url_with_params(None, params);
    let response = ctx
        .send_with(
            HttpMethod::Post,
            url,
            json_headers(),
            Some(serde_json::to_vec(body)?),
            ctx.timeout(tier),
        )
        .await?;
    parse_data_envelope(&response.body)
}

/// A `POST` that gets-or-creates a named resource (`POST {collection}?name=...`),
/// unwrapping the `data` envelope. Shared by the storage collection clients.
pub(crate) async fn get_or_create_named<T: DeserializeOwned>(
    ctx: &ResourceContext,
    name: Option<&str>,
    tier: Timeout,
) -> ApifyClientResult<T> {
    let mut params = QueryParams::new();
    params.add_str("name", name.map(|s| s.to_string()));
    let url = params.apply_to_url(&ctx.url(None));
    let response = ctx.send(HttpMethod::Post, url, ctx.timeout(tier)).await?;
    parse_data_envelope(&response.body)
}

/// A `POST` that unwraps the `data` envelope from the response. Used by the common
/// `{ "data": ... }`-enveloped endpoints; see [`post_action_raw`] for endpoints that return a
/// bare (un-enveloped) body.
pub(crate) async fn post_action<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: Option<Vec<u8>>,
    content_type: Option<&str>,
    timeout: Duration,
) -> ApifyClientResult<T> {
    let response = post_send(ctx, sub_path, params, body, content_type, timeout).await?;
    parse_data_envelope(&response.body)
}

/// Sends a `POST` and returns the raw response body, deserialized directly **without**
/// unwrapping a `data` envelope. A few endpoints (e.g. `validate-input`) return a bare JSON
/// object rather than the usual `{ "data": ... }` envelope, so they must not go through
/// [`parse_data_envelope`].
pub(crate) async fn post_action_raw<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: Option<Vec<u8>>,
    content_type: Option<&str>,
    timeout: Duration,
) -> ApifyClientResult<T> {
    let response = post_send(ctx, sub_path, params, body, content_type, timeout).await?;
    Ok(serde_json::from_slice(&response.body)?)
}

/// Shared `POST` sender used by [`post_action`] and [`post_action_raw`]. Builds the URL with
/// merged query params, sets the optional `Content-Type`, and returns the raw response.
async fn post_send(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: Option<Vec<u8>>,
    content_type: Option<&str>,
    timeout: Duration,
) -> ApifyClientResult<HttpResponse> {
    let url = ctx.url_with_params(sub_path, params);
    let mut headers = HashMap::new();
    if let Some(ct) = content_type {
        headers.insert("Content-Type".to_string(), ct.to_string());
    }
    ctx.send_with(HttpMethod::Post, url, headers, body, timeout)
        .await
}

/// A `GET` returning the raw response body bytes (no `data` envelope), with a `404` mapped to
/// `None` whatever the client's path. Used for key-value-store record values.
pub(crate) async fn get_raw(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    tier: Timeout,
) -> ApifyClientResult<Option<HttpResponse>> {
    catch_not_found(get_raw_required(ctx, sub_path, params, tier).await)
}

/// Like [`get_raw`], but a `404` maps to `None` only when the client names its resource by ID
/// (see [`catch_not_found_for`]). Used for logs.
pub(crate) async fn get_raw_for_resource(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    tier: Timeout,
) -> ApifyClientResult<Option<HttpResponse>> {
    catch_not_found_for(ctx, get_raw_required(ctx, sub_path, params, tier).await)
}

/// A `GET` returning the raw response body bytes (no `data` envelope), propagating errors
/// (including `404`).
pub(crate) async fn get_raw_required(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    tier: Timeout,
) -> ApifyClientResult<HttpResponse> {
    let url = ctx.url_with_params(sub_path, params);
    ctx.send(HttpMethod::Get, url, ctx.timeout(tier)).await
}

/// A `HEAD` request returning whether the resource exists (`true` on 2xx, `false` on 404).
pub(crate) async fn head_exists(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    tier: Timeout,
) -> ApifyClientResult<bool> {
    let url = ctx.url_with_params(sub_path, params);
    let result = ctx.send(HttpMethod::Head, url, ctx.timeout(tier)).await;
    Ok(catch_not_found(result)?.is_some())
}

/// A `PUT` with raw bytes and a content type (used for KVS record uploads).
pub(crate) async fn put_raw(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: Vec<u8>,
    content_type: &str,
    tier: Timeout,
) -> ApifyClientResult<()> {
    put_raw_with_content_encoding(ctx, sub_path, params, body, content_type, None, tier).await
}

/// Like [`put_raw`], but can also set a `Content-Encoding` header (used for KVS record uploads
/// of an already-compressed value). Setting it makes [`HttpClient::call`]'s automatic request
/// compression skip the body (it already refuses to double-encode a request that carries a
/// `Content-Encoding`), so the bytes reach the API exactly as given.
pub(crate) async fn put_raw_with_content_encoding(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: Vec<u8>,
    content_type: &str,
    content_encoding: Option<&str>,
    tier: Timeout,
) -> ApifyClientResult<()> {
    let url = ctx.url_with_params(sub_path, params);
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), content_type.to_string());
    if let Some(encoding) = content_encoding {
        headers.insert("Content-Encoding".to_string(), encoding.to_string());
    }
    ctx.send_with(HttpMethod::Put, url, headers, Some(body), ctx.timeout(tier))
        .await?;
    Ok(())
}

/// Opens a streaming `GET` connection to `url` (already fully built, including query string),
/// for a caller that wants to process a large body (a log or a dataset export) incrementally
/// instead of waiting for [`HttpClient::call`] to buffer the whole response. Delegates to
/// [`HttpClient::call_streaming`], which — unlike a one-off raw `reqwest` call — goes through
/// the configured [`HttpBackend`](crate::http_client::HttpBackend), so a custom backend (or a
/// `ReqwestBackend` wrapping a caller-provided, proxy/TLS-configured `reqwest::Client`) is
/// honoured for streaming requests exactly as it is for buffered ones.
pub(crate) async fn get_byte_stream(
    ctx: &ResourceContext,
    url: String,
    tier: Timeout,
) -> ApifyClientResult<impl futures_util::Stream<Item = ApifyClientResult<Vec<u8>>>> {
    ctx.http
        .call_streaming(HttpRequest {
            method: HttpMethod::Get,
            url,
            headers: Default::default(),
            body: None,
            timeout: ctx.timeout(tier),
        })
        .await
}

/// A `POST` with a raw body that returns an enveloped resource. Used by `actor.start`
/// and `run.metamorph` where the input is arbitrary user JSON.
pub(crate) async fn post_with_body<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: Option<Vec<u8>>,
    content_type: &str,
    timeout: Duration,
) -> ApifyClientResult<T> {
    post_action(ctx, sub_path, params, body, Some(content_type), timeout).await
}

/// A `DELETE` with a JSON body (used for batch request deletion).
pub(crate) async fn delete_with_body<B: Serialize, T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: &B,
    tier: Timeout,
) -> ApifyClientResult<T> {
    let url = ctx.url_with_params(sub_path, params);
    let response = ctx
        .send_with(
            HttpMethod::Delete,
            url,
            json_headers(),
            Some(serde_json::to_vec(body)?),
            ctx.timeout(tier),
        )
        .await?;
    parse_data_envelope(&response.body)
}

/// Polls a `GET` endpoint with `waitForFinish` until the resource reaches a terminal
/// state or `wait_secs` elapses. Used by `RunClient` and `BuildClient`.
///
/// `is_terminal` extracts whether the fetched resource is finished. Returns the latest
/// resource (finished, or still running if the wait budget was exhausted).
///
/// Each poll has no client-side timeout unless the client was given one with `with_timeout`: the
/// API holds a poll for up to a minute, and a poll that is aborted and retried loses nothing but
/// time, yet an arbitrary tier would cut the hold short.
pub(crate) async fn wait_for_finish<T, F>(
    ctx: &ResourceContext,
    wait_secs: Option<i64>,
    is_terminal: F,
) -> ApifyClientResult<T>
where
    T: DeserializeOwned,
    F: Fn(&T) -> bool,
{
    let start = tokio::time::Instant::now();
    let budget = wait_secs.map(|s| Duration::from_secs(s.max(0) as u64));
    // When the fetch below starts hitting `404`, the instant the *first* one was seen; reset on
    // any successful fetch. `None` means the resource has not 404'd (yet).
    let mut not_found_since: Option<tokio::time::Instant> = None;

    loop {
        let remaining_request_secs = match budget {
            Some(budget) => {
                let elapsed = start.elapsed();
                if elapsed >= budget {
                    // Budget exhausted: do one final immediate fetch and return it.
                    let mut params = QueryParams::new();
                    params.add_int("waitForFinish", Some(0));
                    return get_resource_required(ctx, None, &params, Timeout::NoTimeout).await;
                }
                let remaining = (budget - elapsed).as_secs() as i64;
                remaining.min(WAIT_FOR_FINISH_REQUEST_SECS)
            }
            None => WAIT_FOR_FINISH_REQUEST_SECS,
        };

        let mut params = QueryParams::new();
        params.add_int("waitForFinish", Some(remaining_request_secs));

        match get_resource_required::<T>(ctx, None, &params, Timeout::NoTimeout).await {
            Ok(resource) => {
                not_found_since = None;
                if is_terminal(&resource) {
                    return Ok(resource);
                }
                // Not finished yet. If we have no budget left, return what we have.
                if let Some(budget) = budget {
                    if start.elapsed() >= budget {
                        return Ok(resource);
                    }
                }
            }
            Err(err) => {
                let is_not_found = err.as_api_error().is_some_and(|e| e.status_code == 404);
                if !is_not_found {
                    return Err(err);
                }
                // A run/build that keeps 404ing past the grace period was never going to appear
                // (bad ID, or it was deleted); without this, `wait_secs: None` would poll it
                // forever instead of surfacing the not-found error.
                let now = tokio::time::Instant::now();
                let first_seen = *not_found_since.get_or_insert(now);
                if now.duration_since(first_seen) >= NOT_FOUND_GRACE_PERIOD {
                    return Err(err);
                }
            }
        }

        // Brief pause to let replicas catch up, then poll again.
        crate::http_client::sleep_public(WAIT_FOR_FINISH_POLL_INTERVAL).await;
    }
}

/// Cooldown between retries of a run start the API rejected for lack of resources, matching the
/// reference clients.
const WAIT_FOR_RESOURCES_COOLDOWN: Duration = Duration::from_secs(10);
/// `ApiError::error_type` values the API rejects a run start with while the account has no free
/// memory or concurrent-run slot for it. Both clear as other runs/builds finish.
const RESOURCE_LIMIT_ERROR_TYPES: [&str; 2] = [
    "actor-memory-limit-exceeded",
    "concurrent-runs-limit-exceeded",
];

/// Retries `attempt` while it fails with a resource-limit [`ApiError`](crate::ApiError), per
/// `wait_for_resources` (see [`WaitForResources`](crate::clients::actor::WaitForResources)).
/// `None` makes exactly one attempt. Any other error is returned immediately, unretried. Shared
/// by [`ActorClient::start`](crate::clients::actor::ActorClient::start) and
/// [`TaskClient::start`](crate::clients::task::TaskClient::start), whose run-start options and
/// retry behavior are otherwise identical (the don't-repeat-yourself goal of this module).
pub(crate) async fn retry_while_resource_limited<F, Fut, T>(
    wait_for_resources: Option<crate::clients::actor::WaitForResources>,
    mut attempt: F,
) -> ApifyClientResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = ApifyClientResult<T>>,
{
    use crate::clients::actor::WaitForResources;

    let Some(wait) = wait_for_resources else {
        return attempt().await;
    };
    let deadline = match wait {
        WaitForResources::Forever => None,
        WaitForResources::ForSecs(secs) => {
            Some(tokio::time::Instant::now() + Duration::from_secs_f64(secs.max(0.0)))
        }
    };

    loop {
        let err = match attempt().await {
            Ok(value) => return Ok(value),
            Err(err) => err,
        };
        let is_resource_limited = err.as_api_error().is_some_and(|api_err| {
            api_err
                .error_type
                .as_deref()
                .is_some_and(|t| RESOURCE_LIMIT_ERROR_TYPES.contains(&t))
        });
        if !is_resource_limited {
            return Err(err);
        }

        let cooldown = match deadline {
            Some(deadline) => {
                let now = tokio::time::Instant::now();
                if now >= deadline {
                    return Err(err);
                }
                WAIT_FOR_RESOURCES_COOLDOWN.min(deadline - now)
            }
            None => WAIT_FOR_RESOURCES_COOLDOWN,
        };
        crate::http_client::sleep_public(cooldown).await;
    }
}
