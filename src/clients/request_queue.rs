//! Client for a single request queue (`/v2/request-queues/{queueId}` and variants).

use std::collections::HashSet;
use std::time::Duration;

use futures_util::stream::{FuturesUnordered, StreamExt};
use serde::Serialize;

use crate::clients::base::{
    delete_item, delete_resource_with_timeout, delete_with_body,
    get_resource_required_with_timeout, get_resource_with_timeout, post_action_with_timeout,
    put_action, update_resource_with_params, ResourceContext, MEDIUM_REQUEST_TIMEOUT,
    SMALL_REQUEST_TIMEOUT,
};
use crate::common::{encode_path_segment, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, CONTENT_TYPE_JSON};
use crate::models::{
    RequestQueue, RequestQueueHead, RequestQueueOperationInfo, RequestQueueRequest,
};

/// Maximum number of requests the API accepts in a single `requests/batch` call. Larger
/// inputs are split into chunks of this size (matching the reference client's
/// `REQUEST_QUEUE_MAX_REQUESTS_PER_BATCH_OPERATION`).
const MAX_REQUESTS_PER_BATCH_OPERATION: usize = 25;

/// Maximum accepted request-body size (bytes) for a single `requests/batch` call, matching the
/// reference client's `@apify/consts` `MAX_PAYLOAD_SIZE_BYTES` (9 MiB). A chunk that would
/// serialize larger than this (even after the [`MAX_REQUESTS_PER_BATCH_OPERATION`] count cap) is
/// sliced further by [`slice_by_byte_length`].
const MAX_PAYLOAD_SIZE_BYTES: usize = 9_437_184;
/// Fraction of [`MAX_PAYLOAD_SIZE_BYTES`] reserved as a safety buffer, so the byte-size slicing
/// targets a limit slightly under the API's actual cap. Matches the reference client's
/// `SAFETY_BUFFER_PERCENT` (0.01%).
const SAFETY_BUFFER_PERCENT: f64 = 0.0001;

/// Default maximum number of parallel `requests/batch` calls in flight at once, matching the
/// reference client's `DEFAULT_PARALLEL_BATCH_ADD_REQUESTS`.
const DEFAULT_MAX_PARALLEL_BATCH_ADD_REQUESTS: usize = 5;
/// Default number of retry attempts for a chunk's `unprocessedRequests`, matching the reference
/// client's `DEFAULT_UNPROCESSED_RETRIES_BATCH_ADD_REQUESTS`.
const DEFAULT_MAX_UNPROCESSED_REQUESTS_RETRIES: u32 = 3;
/// Default minimum delay before the first unprocessed-request retry (doubled, with jitter, on
/// each subsequent retry), matching the reference client's
/// `DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES_MILLIS`.
const DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES: Duration = Duration::from_millis(500);

/// Options for [`RequestQueueClient::batch_add_requests`].
#[derive(Debug, Default, Clone)]
pub struct BatchAddRequestsOptions {
    /// If `true`, adds all requests to the beginning of the queue. Default `false`.
    pub forefront: Option<bool>,
    /// Maximum number of retry attempts for a chunk's rate-limited (`unprocessedRequests`)
    /// requests. Default `DEFAULT_MAX_UNPROCESSED_REQUESTS_RETRIES` (3).
    pub max_unprocessed_requests_retries: Option<u32>,
    /// Maximum number of `requests/batch` API calls in flight at once. Default
    /// `DEFAULT_MAX_PARALLEL_BATCH_ADD_REQUESTS` (5).
    pub max_parallel: Option<usize>,
    /// Minimum delay before the first unprocessed-request retry; doubles (with jitter) on each
    /// subsequent retry. Default `DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES` (500ms).
    pub min_delay_between_unprocessed_requests_retries: Option<Duration>,
}

/// Appends the array under `key` in `chunk_result` (if present) onto `acc`. Used to merge the
/// per-chunk `processedRequests` / `unprocessedRequests` arrays of a chunked batch-add.
fn merge_request_array(
    acc: &mut Vec<serde_json::Value>,
    chunk_result: &serde_json::Value,
    key: &str,
) {
    if let Some(items) = chunk_result.get(key).and_then(|v| v.as_array()) {
        acc.extend(items.iter().cloned());
    }
}

/// The key the API uses to deduplicate/match a request: its `unique_key`, defaulting to `url`
/// (the same default the server applies when `unique_key` is omitted).
fn request_unique_key(request: &RequestQueueRequest) -> String {
    request
        .unique_key
        .clone()
        .unwrap_or_else(|| request.url.clone())
}

/// Collects the `uniqueKey` field of every already-`processed` request/response object, so a
/// retry can compute which of the originally-submitted requests still remain.
fn processed_unique_keys(processed: &[serde_json::Value]) -> HashSet<String> {
    processed
        .iter()
        .filter_map(|v| v.get("uniqueKey").and_then(|k| k.as_str()))
        .map(str::to_owned)
        .collect()
}

/// Slices `requests` (already capped to at most [`MAX_REQUESTS_PER_BATCH_OPERATION`] items)
/// further so the slice's serialized JSON size stays under `max_byte_length`, mirroring the
/// reference client's `sliceArrayByByteLength`. Returns the whole input unchanged when it
/// already fits. `start_index` is only used to name the offending request in the error message.
fn slice_by_byte_length(
    requests: &[RequestQueueRequest],
    max_byte_length: usize,
    start_index: usize,
) -> ApifyClientResult<Vec<RequestQueueRequest>> {
    if requests.is_empty() {
        return Ok(Vec::new());
    }
    let whole_len = serde_json::to_vec(requests)?.len();
    if whole_len < max_byte_length {
        return Ok(requests.to_vec());
    }

    let mut sliced = Vec::new();
    let mut byte_length = 2usize; // 2 bytes for the empty array `[]`.
    for (i, item) in requests.iter().enumerate() {
        let item_byte_length = serde_json::to_vec(item)?.len();
        if item_byte_length > max_byte_length {
            return Err(crate::error::ApifyClientError::InvalidArgument(format!(
                "RequestQueueClient::batch_add_requests: the size of the request at index {} \
                 exceeds the maximum allowed size ({max_byte_length} bytes)",
                start_index + i
            )));
        }
        if byte_length + item_byte_length >= max_byte_length {
            break;
        }
        byte_length += item_byte_length;
        sliced.push(item.clone());
    }
    // A non-empty input always fits at least one item under `max_byte_length` (the per-item
    // check above already rejects an item that alone exceeds it); the only way `sliced` could
    // still be empty is the razor-thin case where a single item fits alone but not alongside the
    // 2-byte array overhead. Force it through rather than stalling the caller's `while` loop
    // (which advances by `sliced.len()`) on a zero-length chunk.
    if sliced.is_empty() {
        sliced.push(requests[0].clone());
    }
    Ok(sliced)
}

/// Returns the reference client's exponential-backoff-with-jitter delay for the `attempt`-th
/// retry (0-indexed): `(1 + random) * 2^attempt * min_delay`, `random` in `[0, 1)`. Matches
/// `_batchAddRequestsWithRetries`'s backoff formula.
fn unprocessed_retry_backoff(min_delay: Duration, attempt: u32) -> Duration {
    let factor = 2u32.saturating_pow(attempt);
    let base_millis = (min_delay.as_millis() as u64).saturating_mul(u64::from(factor));
    let extra_millis = (base_millis as f64 * random_fraction()) as u64;
    Duration::from_millis(base_millis.saturating_add(extra_millis))
}

/// Modulus applied to the current sub-second nanoseconds to derive [`random_fraction`]'s
/// numerator; also its denominator, so the result lands in `[0, 1)`. `1_000_000` (one
/// microsecond's worth of nanoseconds) is an arbitrary but sufficiently fine-grained choice for
/// jitter — not a value with external meaning to name after anything more specific.
const RANDOM_FRACTION_MODULUS: u32 = 1_000_000;

/// A cheap, non-crypto random fraction in `[0, 1)` for backoff jitter (mirrors JS `Math.random()`
/// in spirit, not in distribution quality — this is jitter, not a security-sensitive value).
fn random_fraction() -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    f64::from(nanos % RANDOM_FRACTION_MODULUS) / f64::from(RANDOM_FRACTION_MODULUS)
}

/// Options for [`RequestQueueClient::list_requests`].
///
/// Covers the spec query parameters of `GET /v2/request-queues/{queueId}/requests`.
#[derive(Debug, Default, Clone)]
pub struct ListRequestsOptions {
    /// Maximum number of requests to return.
    pub limit: Option<i64>,
    /// Start listing after this request ID (exclusive).
    pub exclusive_start_id: Option<String>,
    /// Opaque pagination cursor returned by a previous call.
    pub cursor: Option<String>,
    /// Restrict the returned requests to the given states. The spec defines this as an array of
    /// the enum values `"locked"` and `"pending"`; multiple values are sent comma-joined (matching
    /// the JS reference, which serializes `filter: Array<'locked' | 'pending'>` via `join(',')`).
    pub filter: Option<Vec<String>>,
}

/// Client for a specific request queue.
#[derive(Debug, Clone)]
pub struct RequestQueueClient {
    ctx: ResourceContext,
    client_key: Option<String>,
}

impl RequestQueueClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, resource_path: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, resource_path, id),
            client_key: None,
        }
    }

    /// Creates an RQ client for a run's default queue (nested path, no ID).
    pub(crate) fn nested(http: HttpClient, base_url: &str, sub_path: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, sub_path),
            client_key: None,
        }
    }

    /// Sets the `clientKey` used to identify this client across requests (for locking).
    pub fn with_client_key(mut self, client_key: impl Into<String>) -> Self {
        self.client_key = Some(client_key.into());
        self
    }

    fn base_params(&self) -> QueryParams {
        let mut params = QueryParams::new();
        params.add_str("clientKey", self.client_key.clone());
        params
    }

    /// Fetches the queue metadata, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<RequestQueue>> {
        get_resource_with_timeout(&self.ctx, None, &QueryParams::new(), SMALL_REQUEST_TIMEOUT).await
    }

    /// Updates the queue metadata (e.g. `name`, `title`).
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<RequestQueue> {
        update_resource_with_params(
            &self.ctx,
            None,
            &QueryParams::new(),
            new_fields,
            SMALL_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Deletes the queue.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource_with_timeout(&self.ctx, None, SMALL_REQUEST_TIMEOUT).await
    }

    /// Lists requests from the head of the queue (without locking them).
    pub async fn list_head(&self, limit: Option<i64>) -> ApifyClientResult<RequestQueueHead> {
        let mut params = self.base_params();
        params.add_int("limit", limit);
        get_resource_required_with_timeout(&self.ctx, Some("head"), &params, SMALL_REQUEST_TIMEOUT)
            .await
    }

    /// Adds a single request to the queue. If `forefront` is true, adds it to the front.
    pub async fn add_request(
        &self,
        request: &RequestQueueRequest,
        forefront: bool,
    ) -> ApifyClientResult<RequestQueueOperationInfo> {
        let mut params = self.base_params();
        params.add_bool("forefront", Some(forefront));
        let body = serde_json::to_vec(request)?;
        post_action_with_timeout(
            &self.ctx,
            Some("requests"),
            &params,
            Some(body),
            Some(CONTENT_TYPE_JSON),
            SMALL_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Gets a request by ID, or `None` if it does not exist.
    pub async fn get_request(&self, id: &str) -> ApifyClientResult<Option<RequestQueueRequest>> {
        get_resource_with_timeout(
            &self.ctx,
            Some(&format!("requests/{}", encode_path_segment(id))),
            &self.base_params(),
            SMALL_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Updates a request (which must include its `id`).
    pub async fn update_request(
        &self,
        request: &RequestQueueRequest,
        forefront: bool,
    ) -> ApifyClientResult<RequestQueueOperationInfo> {
        let id = request.id.clone().ok_or_else(|| {
            crate::error::ApifyClientError::InvalidArgument(
                "request.id is required to update a request".to_string(),
            )
        })?;
        let mut params = self.base_params();
        params.add_bool("forefront", Some(forefront));
        update_resource_with_params(
            &self.ctx,
            Some(&format!("requests/{}", encode_path_segment(&id))),
            &params,
            request,
            MEDIUM_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Deletes a request by ID.
    ///
    /// Unlike [`delete`](Self::delete) (the whole queue), a missing request is **not** treated
    /// as a no-op: this call propagates a `404` as an error, matching the JS reference client's
    /// `deleteRequest`. See `delete_item`'s doc comment (`clients::base`) for the full rationale
    /// behind this whole-resource-vs-sub-resource split, shared by every `delete_item` caller.
    pub async fn delete_request(&self, id: &str) -> ApifyClientResult<()> {
        delete_item(
            &self.ctx,
            Some(&format!("requests/{}", encode_path_segment(id))),
            &self.base_params(),
            SMALL_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Lists and locks requests from the head of the queue for `lock_secs` seconds.
    pub async fn list_and_lock_head(
        &self,
        lock_secs: i64,
        limit: Option<i64>,
    ) -> ApifyClientResult<serde_json::Value> {
        let mut params = self.base_params();
        params
            .add_int("lockSecs", Some(lock_secs))
            .add_int("limit", limit);
        post_action_with_timeout(
            &self.ctx,
            Some("head/lock"),
            &params,
            None,
            None,
            MEDIUM_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Adds multiple requests to the queue, using the default retry/parallelism/slicing behavior
    /// of [`batch_add_requests_with_options`](Self::batch_add_requests_with_options) (reference
    /// parity). Use that method directly to override the retry count, parallelism, or retry
    /// delay.
    pub async fn batch_add_requests(
        &self,
        requests: &[RequestQueueRequest],
        forefront: bool,
    ) -> ApifyClientResult<serde_json::Value> {
        self.batch_add_requests_with_options(
            requests,
            BatchAddRequestsOptions {
                forefront: Some(forefront),
                ..Default::default()
            },
        )
        .await
    }

    /// Adds multiple requests to the queue, matching the reference client's `batchAddRequests`
    /// convenience behavior:
    ///
    /// - Requests are chunked to at most `MAX_REQUESTS_PER_BATCH_OPERATION` (25) per API call,
    ///   and each chunk is further sliced (via `slice_by_byte_length`) so its serialized JSON
    ///   payload stays under the API's byte-size limit (`MAX_PAYLOAD_SIZE_BYTES`, less a small
    ///   safety buffer).
    /// - Up to `options.max_parallel` chunk calls are in flight at once (bounded concurrency,
    ///   default `DEFAULT_MAX_PARALLEL_BATCH_ADD_REQUESTS`).
    /// - Any `unprocessedRequests` in a chunk's response (typically caused by rate limiting) are
    ///   retried up to `options.max_unprocessed_requests_retries` additional times
    ///   (default `DEFAULT_MAX_UNPROCESSED_REQUESTS_RETRIES`), with exponential backoff seeded
    ///   by `options.min_delay_between_unprocessed_requests_retries`
    ///   (default `DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES`).
    ///
    /// A chunk call that fails outright (network/API error, after the transport's own retries are
    /// exhausted) does not fail this call: its still-unsubmitted requests are instead folded into
    /// the returned `unprocessedRequests`, matching the reference client's guarantee that this
    /// method itself does not throw for a partial failure — callers must inspect the returned
    /// `unprocessedRequests` to detect that case.
    pub async fn batch_add_requests_with_options(
        &self,
        requests: &[RequestQueueRequest],
        options: BatchAddRequestsOptions,
    ) -> ApifyClientResult<serde_json::Value> {
        let forefront = options.forefront.unwrap_or(false);
        let max_parallel = options
            .max_parallel
            .unwrap_or(DEFAULT_MAX_PARALLEL_BATCH_ADD_REQUESTS)
            .max(1);
        let max_retries = options
            .max_unprocessed_requests_retries
            .unwrap_or(DEFAULT_MAX_UNPROCESSED_REQUESTS_RETRIES);
        let min_delay = options
            .min_delay_between_unprocessed_requests_retries
            .unwrap_or(DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES);

        // Target a limit slightly under the API's actual cap (the safety buffer).
        let payload_size_limit_bytes = MAX_PAYLOAD_SIZE_BYTES
            - ((MAX_PAYLOAD_SIZE_BYTES as f64) * SAFETY_BUFFER_PERCENT).ceil() as usize;

        let mut processed: Vec<serde_json::Value> = Vec::new();
        let mut unprocessed: Vec<serde_json::Value> = Vec::new();
        let mut in_flight = FuturesUnordered::new();

        // Keep a pool of up to `max_parallel` chunk calls running at once: push the next chunk's
        // future, and once the pool is full, await (and drain) whichever finishes first before
        // producing another. This mirrors the reference client's `Promise.race` pool.
        let mut i = 0usize;
        while i < requests.len() {
            let count_capped_end = (i + MAX_REQUESTS_PER_BATCH_OPERATION).min(requests.len());
            let batch =
                slice_by_byte_length(&requests[i..count_capped_end], payload_size_limit_bytes, i)?;
            let batch_len = batch.len();
            let client = self.clone();
            in_flight.push(async move {
                client
                    .batch_add_chunk_with_retries(batch, forefront, max_retries, min_delay)
                    .await
            });

            if in_flight.len() >= max_parallel {
                if let Some((chunk_processed, chunk_unprocessed)) = in_flight.next().await {
                    processed.extend(chunk_processed);
                    unprocessed.extend(chunk_unprocessed);
                }
            }
            i += batch_len;
        }

        // Drain whatever is still in flight once every chunk has been submitted.
        while let Some((chunk_processed, chunk_unprocessed)) = in_flight.next().await {
            processed.extend(chunk_processed);
            unprocessed.extend(chunk_unprocessed);
        }

        Ok(serde_json::json!({
            "processedRequests": processed,
            "unprocessedRequests": unprocessed,
        }))
    }

    /// Adds one chunk (already count- and byte-size-limited) of requests, retrying any
    /// `unprocessedRequests` up to `max_retries` additional times with exponential backoff
    /// (mirrors the reference client's `_batchAddRequestsWithRetries`). Never returns an `Err`
    /// for a failed chunk POST: the not-yet-processed requests are folded into the returned
    /// `unprocessed` list instead, so the caller's overall call does not fail for a partial
    /// failure.
    async fn batch_add_chunk_with_retries(
        &self,
        requests: Vec<RequestQueueRequest>,
        forefront: bool,
        max_retries: u32,
        min_delay: Duration,
    ) -> (Vec<serde_json::Value>, Vec<serde_json::Value>) {
        let mut remaining = requests;
        let mut processed: Vec<serde_json::Value> = Vec::new();
        let mut unprocessed: Vec<serde_json::Value> = Vec::new();

        for attempt in 0..=max_retries {
            match self.batch_add_chunk(&remaining, forefront).await {
                Ok(chunk_result) => {
                    merge_request_array(&mut processed, &chunk_result, "processedRequests");
                    unprocessed = Vec::new();
                    merge_request_array(&mut unprocessed, &chunk_result, "unprocessedRequests");

                    if unprocessed.is_empty() {
                        break;
                    }

                    // Only requests not yet confirmed processed are worth retrying.
                    let done = processed_unique_keys(&processed);
                    remaining.retain(|r| !done.contains(&request_unique_key(r)));

                    if remaining.is_empty() {
                        break;
                    }
                }
                Err(_) => {
                    // The transport already retried transient failures; a surfaced error means
                    // this chunk truly could not be submitted. Report every not-yet-processed
                    // request as unprocessed rather than failing the whole call.
                    let done = processed_unique_keys(&processed);
                    unprocessed = remaining
                        .iter()
                        .filter(|r| !done.contains(&request_unique_key(r)))
                        .map(|r| {
                            serde_json::json!({
                                "method": r.method,
                                "uniqueKey": request_unique_key(r),
                                "url": r.url,
                            })
                        })
                        .collect();
                    break;
                }
            }

            if attempt < max_retries {
                crate::http_client::sleep_public(unprocessed_retry_backoff(min_delay, attempt))
                    .await;
            }
        }

        (processed, unprocessed)
    }

    /// Posts a single chunk of requests (at most [`MAX_REQUESTS_PER_BATCH_OPERATION`]).
    async fn batch_add_chunk(
        &self,
        requests: &[RequestQueueRequest],
        forefront: bool,
    ) -> ApifyClientResult<serde_json::Value> {
        let mut params = self.base_params();
        params.add_bool("forefront", Some(forefront));
        let body = serde_json::to_vec(requests)?;
        post_action_with_timeout(
            &self.ctx,
            Some("requests/batch"),
            &params,
            Some(body),
            Some(CONTENT_TYPE_JSON),
            MEDIUM_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Deletes multiple requests in a single batch operation.
    pub async fn batch_delete_requests<T: Serialize>(
        &self,
        requests: &[T],
    ) -> ApifyClientResult<serde_json::Value> {
        delete_with_body(
            &self.ctx,
            Some("requests/batch"),
            &self.base_params(),
            &requests,
            SMALL_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Lists requests in the queue.
    ///
    /// Supports pagination via `limit`/`exclusive_start_id` and the spec's `cursor`/`filter`
    /// parameters (see [`ListRequestsOptions`]).
    pub async fn list_requests(
        &self,
        options: ListRequestsOptions,
    ) -> ApifyClientResult<serde_json::Value> {
        let mut params = self.base_params();
        params
            .add_int("limit", options.limit)
            .add_str("exclusiveStartId", options.exclusive_start_id)
            .add_str("cursor", options.cursor)
            .add_csv("filter", options.filter.as_deref());
        get_resource_required_with_timeout(
            &self.ctx,
            Some("requests"),
            &params,
            MEDIUM_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Prolongs the lock on a request for another `lock_secs` seconds.
    ///
    /// If `forefront` is `true`, the request moves to the front of the queue when its lock
    /// later expires.
    pub async fn prolong_request_lock(
        &self,
        id: &str,
        lock_secs: i64,
        forefront: bool,
    ) -> ApifyClientResult<serde_json::Value> {
        let mut params = self.base_params();
        params
            .add_int("lockSecs", Some(lock_secs))
            .add_bool("forefront", Some(forefront));
        put_action(
            &self.ctx,
            Some(&format!("requests/{}/lock", encode_path_segment(id))),
            &params,
            None,
            None,
            MEDIUM_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Releases the lock on a request so other clients can process it.
    ///
    /// If `forefront` is `true`, the request moves to the front of the queue. Like
    /// [`delete_request`](Self::delete_request), a missing lock is not treated as a no-op
    /// (matching the JS reference client's `deleteRequestLock`, which does not catch not-found).
    pub async fn delete_request_lock(&self, id: &str, forefront: bool) -> ApifyClientResult<()> {
        let mut params = self.base_params();
        params.add_bool("forefront", Some(forefront));
        delete_item(
            &self.ctx,
            Some(&format!("requests/{}/lock", encode_path_segment(id))),
            &params,
            SMALL_REQUEST_TIMEOUT,
        )
        .await
    }

    /// Lazily paginates over all requests in the queue, fetching pages on demand.
    ///
    /// Returns a [`RequestQueueRequestsIterator`]; call its `next()` to get one request at a
    /// time. Pagination uses the API's opaque `nextCursor` token: the first page may be
    /// anchored with `exclusiveStartId`, but every subsequent page is fetched with `cursor`
    /// (matching the JS reference). `cursor` and `exclusiveStartId` are mutually exclusive.
    pub fn paginate_requests(&self, page_limit: Option<i64>) -> RequestQueueRequestsIterator {
        RequestQueueRequestsIterator {
            client: self.clone(),
            page_limit,
            buffer: std::collections::VecDeque::new(),
            next_cursor: None,
            exhausted: false,
        }
    }

    /// Unlocks all requests currently locked by this client (identified by `client_key`).
    pub async fn unlock_requests(&self) -> ApifyClientResult<serde_json::Value> {
        post_action_with_timeout(
            &self.ctx,
            Some("requests/unlock"),
            &self.base_params(),
            None,
            None,
            MEDIUM_REQUEST_TIMEOUT,
        )
        .await
    }
}

/// A lazy, page-fetching iterator over the requests in a queue.
///
/// Created by [`RequestQueueClient::paginate_requests`]. Each call to [`next`](Self::next)
/// returns the next request, fetching another page from the API when the local buffer is
/// exhausted, until all requests have been yielded.
pub struct RequestQueueRequestsIterator {
    client: RequestQueueClient,
    page_limit: Option<i64>,
    buffer: std::collections::VecDeque<RequestQueueRequest>,
    /// Opaque pagination token returned by the previous page, fed back as `cursor`.
    next_cursor: Option<String>,
    exhausted: bool,
}

impl RequestQueueRequestsIterator {
    /// Returns the next request, or `None` when all requests have been yielded.
    pub async fn next(&mut self) -> ApifyClientResult<Option<RequestQueueRequest>> {
        if let Some(item) = self.buffer.pop_front() {
            return Ok(Some(item));
        }
        if self.exhausted {
            return Ok(None);
        }

        // The first page may be anchored by exclusiveStartId; every later page is fetched
        // with the opaque `cursor` token (mutually exclusive with exclusiveStartId), matching
        // the JS reference. Here we only ever paginate from the queue head, so the first page
        // uses neither and subsequent pages use `cursor`.
        let page = self
            .client
            .list_requests(ListRequestsOptions {
                limit: self.page_limit,
                cursor: self.next_cursor.clone(),
                ..Default::default()
            })
            .await?;

        // Parse the items and the next cursor from the (untyped) page.
        let items: Vec<RequestQueueRequest> = page
            .get("items")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_default();

        if items.is_empty() {
            self.exhausted = true;
            return Ok(None);
        }

        // Advance the cursor; stop when the API stops returning one.
        match page.get("nextCursor").and_then(|v| v.as_str()) {
            Some(cursor) if !cursor.is_empty() => self.next_cursor = Some(cursor.to_string()),
            _ => self.exhausted = true,
        }

        self.buffer.extend(items);
        Ok(self.buffer.pop_front())
    }
}
