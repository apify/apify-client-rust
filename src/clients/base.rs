//! Shared internal machinery for resource clients.
//!
//! [`ResourceContext`] holds the resolved URL and the [`HttpClient`] for a single
//! resource (or sub-resource). The free functions in this module implement the CRUD
//! and wait-for-finish primitives once, so that every resource client stays small and
//! consistent (the DRY principle).

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::common::{
    catch_not_found, parse_data_envelope, to_safe_id, PaginationList, QueryParams,
};
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, HttpMethod, HttpRequest};

/// How long to wait between polls while waiting for a run/build to finish.
const WAIT_FOR_FINISH_POLL_INTERVAL: Duration = Duration::from_millis(250);
/// The `waitForFinish` value (in seconds) sent on each poll; the API caps server-side
/// waiting at 60 seconds, so we poll in chunks of this size.
const WAIT_FOR_FINISH_REQUEST_SECS: i64 = 60;

/// Default per-request timeout used when an endpoint does not specify its own (6 minutes).
/// This is the single source of truth for the base request timeout; clients that issue raw
/// requests reuse it instead of repeating the literal.
pub(crate) const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(360);
/// Short timeout for fast, idempotent metadata operations (mirrors the JS `SMALL_TIMEOUT`).
pub(crate) const SMALL_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
/// Medium timeout for operations that may take a little longer (mirrors JS `MEDIUM_TIMEOUT`).
pub(crate) const MEDIUM_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

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
}

impl ResourceContext {
    /// Creates a context for a collection endpoint: `{base}/{resource_path}`.
    pub(crate) fn collection(http: HttpClient, base_url: &str, resource_path: &str) -> Self {
        Self::new(http, format!("{base_url}/{resource_path}"), base_url)
    }

    /// Creates a context for a single resource: `{base}/{resource_path}/{safe_id}`.
    pub(crate) fn single(http: HttpClient, base_url: &str, resource_path: &str, id: &str) -> Self {
        Self::new(
            http,
            format!("{base_url}/{resource_path}/{}", to_safe_id(id)),
            base_url,
        )
    }

    /// Internal constructor that derives the API origin from `base_url`. The public origin
    /// defaults to the API origin and can be overridden with [`with_public_origin`].
    fn new(http: HttpClient, url: String, base_url: &str) -> Self {
        let api_origin = origin_of(base_url);
        Self {
            http,
            url,
            base_params: QueryParams::new(),
            public_origin: api_origin.clone(),
            api_origin,
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
}

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

/// A `GET` that unwraps the `data` envelope and maps `404` to `None`.
pub(crate) async fn get_resource<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
) -> ApifyClientResult<Option<T>> {
    let result = get_resource_required(ctx, sub_path, params).await;
    catch_not_found(result)
}

/// A `GET` that unwraps the `data` envelope and propagates errors (including `404`).
pub(crate) async fn get_resource_required<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
) -> ApifyClientResult<T> {
    let url = ctx.merged_params(params).apply_to_url(&ctx.url(sub_path));
    let response = ctx
        .http
        .call(HttpRequest {
            method: HttpMethod::Get,
            url,
            headers: Default::default(),
            body: None,
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await?;
    parse_data_envelope(&response.body)
}

/// A `PUT` with a JSON body that unwraps the `data` envelope from the response.
pub(crate) async fn update_resource<B: Serialize, T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    body: &B,
) -> ApifyClientResult<T> {
    let url = ctx
        .merged_params(&QueryParams::new())
        .apply_to_url(&ctx.url(sub_path));
    let body_bytes = serde_json::to_vec(body)?;
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let response = ctx
        .http
        .call(HttpRequest {
            method: HttpMethod::Put,
            url,
            headers,
            body: Some(body_bytes),
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await?;
    parse_data_envelope(&response.body)
}

/// A `DELETE` that maps `404` to a successful no-op.
pub(crate) async fn delete_resource(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
) -> ApifyClientResult<()> {
    let url = ctx
        .merged_params(&QueryParams::new())
        .apply_to_url(&ctx.url(sub_path));
    let result = ctx
        .http
        .call(HttpRequest {
            method: HttpMethod::Delete,
            url,
            headers: Default::default(),
            body: None,
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await;
    catch_not_found(result.map(|_| ()))?;
    Ok(())
}

/// A `GET` that returns a paginated list (`data` envelope wrapping `{ items, total, ... }`).
pub(crate) async fn list_resource<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
) -> ApifyClientResult<PaginationList<T>> {
    get_resource_required(ctx, sub_path, params).await
}

/// A `POST` with a JSON body that unwraps the `data` envelope (used for `create`).
pub(crate) async fn create_resource<B: Serialize, T: DeserializeOwned>(
    ctx: &ResourceContext,
    params: &QueryParams,
    body: &B,
) -> ApifyClientResult<T> {
    let url = ctx.merged_params(params).apply_to_url(&ctx.url(None));
    let body_bytes = serde_json::to_vec(body)?;
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let response = ctx
        .http
        .call(HttpRequest {
            method: HttpMethod::Post,
            url,
            headers,
            body: Some(body_bytes),
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await?;
    parse_data_envelope(&response.body)
}

/// A `POST` that gets-or-creates a named resource (`POST {collection}?name=...`),
/// unwrapping the `data` envelope. Shared by the storage collection clients.
pub(crate) async fn get_or_create_named<T: DeserializeOwned>(
    ctx: &ResourceContext,
    name: Option<&str>,
) -> ApifyClientResult<T> {
    let mut params = QueryParams::new();
    params.add_str("name", name.map(|s| s.to_string()));
    let url = params.apply_to_url(&ctx.url(None));
    let response = ctx
        .http
        .call(HttpRequest {
            method: HttpMethod::Post,
            url,
            headers: Default::default(),
            body: None,
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await?;
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
) -> ApifyClientResult<T> {
    let response = post_send(ctx, sub_path, params, body, content_type).await?;
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
) -> ApifyClientResult<T> {
    let response = post_send(ctx, sub_path, params, body, content_type).await?;
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
) -> ApifyClientResult<crate::http_client::HttpResponse> {
    let url = ctx.merged_params(params).apply_to_url(&ctx.url(sub_path));
    let mut headers = std::collections::HashMap::new();
    if let Some(ct) = content_type {
        headers.insert("Content-Type".to_string(), ct.to_string());
    }
    ctx.http
        .call(HttpRequest {
            method: HttpMethod::Post,
            url,
            headers,
            body,
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await
}

/// A `GET` returning the raw response body bytes (no `data` envelope). Maps `404`/`HEAD`
/// not-found to `None`. Used for logs and key-value-store record values.
pub(crate) async fn get_raw(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
) -> ApifyClientResult<Option<crate::http_client::HttpResponse>> {
    let url = ctx.merged_params(params).apply_to_url(&ctx.url(sub_path));
    let result = ctx
        .http
        .call(HttpRequest {
            method: HttpMethod::Get,
            url,
            headers: Default::default(),
            body: None,
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await;
    catch_not_found(result)
}

/// A `HEAD` request returning whether the resource exists (`true` on 2xx, `false` on 404).
pub(crate) async fn head_exists(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
) -> ApifyClientResult<bool> {
    let url = ctx.merged_params(params).apply_to_url(&ctx.url(sub_path));
    let result = ctx
        .http
        .call(HttpRequest {
            method: HttpMethod::Head,
            url,
            headers: Default::default(),
            body: None,
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await;
    Ok(catch_not_found(result)?.is_some())
}

/// A `PUT` with raw bytes and a content type (used for KVS record uploads).
pub(crate) async fn put_raw(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: Vec<u8>,
    content_type: &str,
) -> ApifyClientResult<()> {
    let url = ctx.merged_params(params).apply_to_url(&ctx.url(sub_path));
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), content_type.to_string());
    ctx.http
        .call(HttpRequest {
            method: HttpMethod::Put,
            url,
            headers,
            body: Some(body),
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await?;
    Ok(())
}

/// A `POST` with a raw body that returns an enveloped resource. Used by `actor.start`
/// and `run.metamorph` where the input is arbitrary user JSON.
pub(crate) async fn post_with_body<T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: Option<Vec<u8>>,
    content_type: &str,
) -> ApifyClientResult<T> {
    post_action(ctx, sub_path, params, body, Some(content_type)).await
}

/// A `DELETE` with a JSON body (used for batch request deletion).
pub(crate) async fn delete_with_body<B: Serialize, T: DeserializeOwned>(
    ctx: &ResourceContext,
    sub_path: Option<&str>,
    params: &QueryParams,
    body: &B,
) -> ApifyClientResult<T> {
    let url = ctx.merged_params(params).apply_to_url(&ctx.url(sub_path));
    let body_bytes = serde_json::to_vec(body)?;
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let response = ctx
        .http
        .call(HttpRequest {
            method: HttpMethod::Delete,
            url,
            headers,
            body: Some(body_bytes),
            timeout: DEFAULT_REQUEST_TIMEOUT,
        })
        .await?;
    parse_data_envelope(&response.body)
}

/// Polls a `GET` endpoint with `waitForFinish` until the resource reaches a terminal
/// state or `wait_secs` elapses. Used by `RunClient` and `BuildClient`.
///
/// `is_terminal` extracts whether the fetched resource is finished. Returns the latest
/// resource (finished, or still running if the wait budget was exhausted).
pub(crate) async fn wait_for_finish<T, F>(
    ctx: &ResourceContext,
    wait_secs: Option<i64>,
    is_terminal: F,
) -> ApifyClientResult<T>
where
    T: DeserializeOwned,
    F: Fn(&T) -> bool,
{
    let start = std::time::Instant::now();
    let budget = wait_secs.map(|s| Duration::from_secs(s.max(0) as u64));

    loop {
        let remaining_request_secs = match budget {
            Some(budget) => {
                let elapsed = start.elapsed();
                if elapsed >= budget {
                    // Budget exhausted: do one final immediate fetch and return it.
                    let mut params = QueryParams::new();
                    params.add_int("waitForFinish", Some(0));
                    return get_resource_required(ctx, None, &params).await;
                }
                let remaining = (budget - elapsed).as_secs() as i64;
                remaining.min(WAIT_FOR_FINISH_REQUEST_SECS)
            }
            None => WAIT_FOR_FINISH_REQUEST_SECS,
        };

        let mut params = QueryParams::new();
        params.add_int("waitForFinish", Some(remaining_request_secs));

        let resource: Option<T> = get_resource(ctx, None, &params).await?;
        if let Some(resource) = resource {
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

        // Brief pause to let replicas catch up, then poll again.
        crate::http_client::sleep_public(WAIT_FOR_FINISH_POLL_INTERVAL).await;
    }
}
