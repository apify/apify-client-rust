//! Client for a single request queue (`/v2/request-queues/{queueId}` and variants).

use std::collections::HashSet;
use std::time::Duration;

use futures_util::stream::{self, StreamExt};
use serde::Serialize;

use crate::clients::base::{
    delete_resource, delete_with_body, get_resource, get_resource_required, post_action,
    post_with_body, update_resource, ResourceContext,
};
use crate::common::{encode_path_segment, QueryParams};
use crate::error::{ApifyClientError, ApifyClientResult};
use crate::http_client::{sleep_public, HttpClient, HttpMethod, HttpRequest};
use crate::models::{
    BatchRequestsOperationResult, LockedRequestQueueHead, RequestLockInfo, RequestQueue,
    RequestQueueHead, RequestQueueOperationInfo, RequestQueueRequest, RequestQueueRequestsPage,
    UnlockRequestsResult, UnprocessedRequest,
};

/// Maximum number of requests the API accepts in a single `requests/batch` call. Larger
/// `batch_add_requests` inputs are split into chunks of at most this size (matching the
/// reference client's `REQUEST_QUEUE_MAX_REQUESTS_PER_BATCH_OPERATION`); `batch_delete_requests`
/// does not auto-chunk (matching the reference client) and instead rejects larger inputs.
const MAX_REQUESTS_PER_BATCH_OPERATION: usize = 25;
/// Maximum accepted size (bytes) of a request body, mirroring the platform-wide
/// `MAX_PAYLOAD_SIZE_BYTES` (9 MiB) that the reference client chunks `batch_add_requests` calls
/// against, on top of the per-call request-count limit.
const MAX_PAYLOAD_SIZE_BYTES: usize = 9_437_184;
/// Fraction of [`MAX_PAYLOAD_SIZE_BYTES`] held back as a safety margin (0.01%), matching the
/// reference client's `SAFETY_BUFFER_PERCENT`.
const SAFETY_BUFFER_PERCENT: f64 = 0.0001;
/// Default number of batch-add API calls [`RequestQueueClient::batch_add_requests`] keeps in
/// flight at once, matching the reference client's `DEFAULT_PARALLEL_BATCH_ADD_REQUESTS`.
const DEFAULT_MAX_PARALLEL_BATCH_ADD_REQUESTS: usize = 5;
/// Default number of retry attempts for requests a batch-add call reports as `unprocessed`
/// (typically rate-limited), matching the reference client's
/// `DEFAULT_UNPROCESSED_RETRIES_BATCH_ADD_REQUESTS`.
const DEFAULT_MAX_UNPROCESSED_REQUESTS_RETRIES: u32 = 3;
/// Default minimum delay before the first unprocessed-request retry; doubles (with jitter) on
/// each subsequent retry, matching the reference client's
/// `DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES_MILLIS`.
const DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES: Duration = Duration::from_millis(500);

/// Options for [`RequestQueueClient::batch_add_requests`].
///
/// Mirrors the reference client's retrying, chunked, parallel `batchAddRequests`: large inputs
/// are split by count (max [`MAX_REQUESTS_PER_BATCH_OPERATION`]) and by JSON byte size (max
/// [`MAX_PAYLOAD_SIZE_BYTES`], minus a safety margin), chunks are sent with up to `max_parallel`
/// requests in flight at once, and any request an API call reports as `unprocessed` (typically
/// due to rate limiting) is retried with exponential backoff.
#[derive(Debug, Default, Clone)]
pub struct BatchAddRequestsOptions {
    /// If `true`, adds all requests to the front of the queue.
    pub forefront: bool,
    /// Maximum retries for requests reported as `unprocessed`. Defaults to
    /// [`DEFAULT_MAX_UNPROCESSED_REQUESTS_RETRIES`] (3) when `None`.
    pub max_unprocessed_requests_retries: Option<u32>,
    /// Maximum number of chunk-add API calls in flight at once. Defaults to
    /// [`DEFAULT_MAX_PARALLEL_BATCH_ADD_REQUESTS`] (5) when `None`.
    pub max_parallel: Option<usize>,
    /// Minimum delay before the first unprocessed-request retry (doubles, with jitter, on each
    /// subsequent retry). Defaults to
    /// [`DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES`] (500ms) when `None`.
    pub min_delay_between_unprocessed_requests_retries: Option<Duration>,
}

/// Returns the key used to correlate a request across the batch-add retry loop: its explicit
/// `unique_key` if set, otherwise its `url` — matching the API's own fallback (a request added
/// without a `unique_key` is deduplicated by its normalized URL).
fn dedup_key(request: &RequestQueueRequest) -> &str {
    request.unique_key.as_deref().unwrap_or(&request.url)
}

/// Returns the JSON-serialized byte length of `value`.
fn json_byte_len<T: Serialize + ?Sized>(value: &T) -> ApifyClientResult<usize> {
    Ok(serde_json::to_vec(value)?.len())
}

/// Slices `requests` down to a byte-limited prefix, mirroring the reference client's
/// `sliceArrayByByteLength`: if the whole slice already fits under `max_bytes` it is returned
/// unchanged; otherwise items are accumulated one at a time until the next one would exceed the
/// budget. `start_index` is only used to name the offending item in the error message, so it
/// should be the slice's absolute position within the caller's full input.
///
/// The first item is always included regardless of size (once its own size has been checked
/// against `max_bytes`), guaranteeing a non-empty result for a non-empty input — unlike the
/// reference implementation, which can return an empty slice (and loop forever) when a single
/// item's size leaves no room under `max_bytes` for even itself plus the array wrapper.
///
/// Returns [`ApifyClientError::InvalidArgument`] if a single request's JSON exceeds `max_bytes`
/// on its own (mirroring the reference client's thrown error).
fn slice_requests_by_byte_length(
    requests: &[RequestQueueRequest],
    max_bytes: usize,
    start_index: usize,
) -> ApifyClientResult<Vec<RequestQueueRequest>> {
    if json_byte_len(requests)? < max_bytes {
        return Ok(requests.to_vec());
    }
    let mut out = Vec::new();
    let mut byte_length = 2usize; // 2 bytes for the empty array `[]`.
    for (offset, request) in requests.iter().enumerate() {
        let item_bytes = json_byte_len(request)?;
        if item_bytes > max_bytes {
            return Err(ApifyClientError::InvalidArgument(format!(
                "RequestQueueClient::batch_add_requests: the request at index {} exceeds the \
                 maximum allowed size ({max_bytes} bytes)",
                start_index + offset
            )));
        }
        if !out.is_empty() && byte_length + item_bytes >= max_bytes {
            break;
        }
        byte_length += item_bytes;
        out.push(request.clone());
    }
    Ok(out)
}

/// Splits `requests` into chunks that each satisfy both the per-call count limit
/// ([`MAX_REQUESTS_PER_BATCH_OPERATION`]) and the payload byte-size limit
/// (`max_bytes`), mirroring the reference client's chunking loop in `batchAddRequests`.
fn chunk_requests_for_batch_add(
    requests: &[RequestQueueRequest],
    max_bytes: usize,
) -> ApifyClientResult<Vec<Vec<RequestQueueRequest>>> {
    let mut chunks = Vec::new();
    let mut i = 0;
    while i < requests.len() {
        let group_end = (i + MAX_REQUESTS_PER_BATCH_OPERATION).min(requests.len());
        let chunk = slice_requests_by_byte_length(&requests[i..group_end], max_bytes, i)?;
        // `slice_requests_by_byte_length` always returns at least one item for a non-empty
        // input, so this advances on every iteration.
        i += chunk.len();
        chunks.push(chunk);
    }
    Ok(chunks)
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
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the queue metadata (e.g. `name`, `title`).
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<RequestQueue> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the queue.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Lists requests from the head of the queue (without locking them).
    pub async fn list_head(&self, limit: Option<i64>) -> ApifyClientResult<RequestQueueHead> {
        let mut params = self.base_params();
        params.add_int("limit", limit);
        get_resource_required(&self.ctx, Some("head"), &params).await
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
        post_with_body(
            &self.ctx,
            Some("requests"),
            &params,
            Some(body),
            "application/json",
        )
        .await
    }

    /// Gets a request by ID, or `None` if it does not exist.
    pub async fn get_request(&self, id: &str) -> ApifyClientResult<Option<RequestQueueRequest>> {
        get_resource(
            &self.ctx,
            Some(&format!("requests/{}", encode_path_segment(id))),
            &self.base_params(),
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
        let url = params.apply_to_url(
            &self
                .ctx
                .url(Some(&format!("requests/{}", encode_path_segment(&id)))),
        );
        let body = serde_json::to_vec(request)?;
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let response = self
            .ctx
            .http
            .call(HttpRequest {
                method: HttpMethod::Put,
                url,
                headers,
                body: Some(body),
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        crate::common::parse_data_envelope(&response.body)
    }

    /// Deletes a request by ID.
    pub async fn delete_request(&self, id: &str) -> ApifyClientResult<()> {
        let params = self.base_params();
        let url = params.apply_to_url(
            &self
                .ctx
                .url(Some(&format!("requests/{}", encode_path_segment(id)))),
        );
        self.ctx
            .http
            .call(HttpRequest {
                method: HttpMethod::Delete,
                url,
                headers: Default::default(),
                body: None,
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        Ok(())
    }

    /// Lists and locks requests from the head of the queue for `lock_secs` seconds.
    pub async fn list_and_lock_head(
        &self,
        lock_secs: i64,
        limit: Option<i64>,
    ) -> ApifyClientResult<LockedRequestQueueHead> {
        let mut params = self.base_params();
        params
            .add_int("lockSecs", Some(lock_secs))
            .add_int("limit", limit);
        post_action(&self.ctx, Some("head/lock"), &params, None, None).await
    }

    /// Adds multiple requests to the queue in a single logical operation.
    ///
    /// This is significantly more efficient than calling [`add_request`](Self::add_request)
    /// once per request, especially for large batches: the input is automatically split into
    /// chunks that respect both the API's per-call request-count limit
    /// ([`MAX_REQUESTS_PER_BATCH_OPERATION`]) and its request-body byte-size limit
    /// ([`MAX_PAYLOAD_SIZE_BYTES`]), chunks are sent with up to `options.max_parallel` API calls
    /// in flight at once, and any request an API call reports as `unprocessed` (typically due to
    /// rate limiting) is retried with exponential backoff — matching the reference client's
    /// `batchAddRequests`. Every request must be identifiable by [`RequestQueueRequest::unique_key`]
    /// (or, if left unset, by `url`, the API's own fallback) so a retried request can be matched
    /// back to the original input.
    ///
    /// Unlike most methods here, this does not propagate per-chunk API errors: a chunk that fails
    /// even after retries has its requests reported in the result's `unprocessed_requests`
    /// instead, so a batch add of many requests never fails outright over one bad chunk (matching
    /// the reference client). A [`ApifyClientError::InvalidArgument`] is still returned before any
    /// request is sent if a single request's JSON is too large to ever fit in a chunk.
    pub async fn batch_add_requests(
        &self,
        requests: &[RequestQueueRequest],
        options: BatchAddRequestsOptions,
    ) -> ApifyClientResult<BatchRequestsOperationResult> {
        if requests.is_empty() {
            return Ok(BatchRequestsOperationResult::default());
        }
        let max_parallel = options
            .max_parallel
            .unwrap_or(DEFAULT_MAX_PARALLEL_BATCH_ADD_REQUESTS)
            .max(1);
        let payload_limit_bytes = MAX_PAYLOAD_SIZE_BYTES
            - (MAX_PAYLOAD_SIZE_BYTES as f64 * SAFETY_BUFFER_PERCENT).ceil() as usize;
        let chunks = chunk_requests_for_batch_add(requests, payload_limit_bytes)?;

        let merged = stream::iter(chunks)
            .map(|chunk| {
                let client = self.clone();
                let options = options.clone();
                async move {
                    client
                        .batch_add_requests_chunk_with_retries(chunk, options)
                        .await
                }
            })
            .buffer_unordered(max_parallel)
            .fold(
                BatchRequestsOperationResult::default(),
                |mut acc, chunk_result| async move {
                    acc.processed_requests
                        .extend(chunk_result.processed_requests);
                    acc.unprocessed_requests
                        .extend(chunk_result.unprocessed_requests);
                    acc
                },
            )
            .await;
        Ok(merged)
    }

    /// Sends one already-byte/count-limited chunk, retrying requests reported as `unprocessed`
    /// with exponential backoff (matching the reference client's `_batchAddRequestsWithRetries`).
    ///
    /// Never returns an `Err`: a transport/API failure that survives `HttpClient`'s own retries
    /// marks every request still outstanding in this chunk as unprocessed instead of propagating,
    /// so a single bad chunk cannot fail the whole (possibly-parallel) `batch_add_requests` call.
    async fn batch_add_requests_chunk_with_retries(
        &self,
        chunk: Vec<RequestQueueRequest>,
        options: BatchAddRequestsOptions,
    ) -> BatchRequestsOperationResult {
        let max_retries = options
            .max_unprocessed_requests_retries
            .unwrap_or(DEFAULT_MAX_UNPROCESSED_REQUESTS_RETRIES);
        let min_delay = options
            .min_delay_between_unprocessed_requests_retries
            .unwrap_or(DEFAULT_MIN_DELAY_BETWEEN_UNPROCESSED_REQUESTS_RETRIES);

        let mut remaining = chunk;
        let mut processed = Vec::new();

        for attempt in 0..=max_retries {
            match self
                .batch_add_requests_raw(&remaining, options.forefront)
                .await
            {
                Ok(result) => {
                    let processed_keys: HashSet<&str> = result
                        .processed_requests
                        .iter()
                        .filter_map(|p| p.unique_key.as_deref())
                        .collect();
                    remaining.retain(|r| !processed_keys.contains(dedup_key(r)));
                    processed.extend(result.processed_requests);
                    if remaining.is_empty() {
                        return BatchRequestsOperationResult {
                            processed_requests: processed,
                            unprocessed_requests: Vec::new(),
                        };
                    }
                    if attempt == max_retries {
                        return BatchRequestsOperationResult {
                            processed_requests: processed,
                            unprocessed_requests: result.unprocessed_requests,
                        };
                    }
                }
                Err(_) => {
                    // A hard failure (already retried by `HttpClient` for transient errors):
                    // treat every request still outstanding in this chunk as unprocessed rather
                    // than propagating, matching the reference client's "never throws" contract.
                    return BatchRequestsOperationResult {
                        processed_requests: processed,
                        unprocessed_requests: remaining
                            .iter()
                            .map(|r| UnprocessedRequest {
                                unique_key: dedup_key(r).to_string(),
                                url: r.url.clone(),
                                method: r.method.clone(),
                            })
                            .collect(),
                    };
                }
            }
            // Exponential backoff with jitter before the next retry, matching the reference
            // client's `(1 + random()) * 2^attempt * minDelay` (see `randomized_delay`, which
            // returns a value in `[base, 2*base)`, i.e. `(1 + random()) * base`).
            let backoff = min_delay.saturating_mul(2u32.saturating_pow(attempt));
            sleep_public(crate::http_client::randomized_delay(backoff)).await;
        }
        // Unreachable: the loop above always returns on its last iteration (`attempt ==
        // max_retries` is handled inside the `Ok` arm, and `Err` returns unconditionally). Kept
        // as a safe fallback rather than `unreachable!()` so a future refactor of the loop bounds
        // fails safe (reporting the batch unprocessed) instead of panicking.
        BatchRequestsOperationResult {
            processed_requests: processed,
            unprocessed_requests: remaining
                .iter()
                .map(|r| crate::models::UnprocessedRequest {
                    unique_key: dedup_key(r).to_string(),
                    url: r.url.clone(),
                    method: r.method.clone(),
                })
                .collect(),
        }
    }

    /// Sends a single `POST requests/batch` call (at most [`MAX_REQUESTS_PER_BATCH_OPERATION`]
    /// requests, and within the byte-size budget already enforced by the caller).
    async fn batch_add_requests_raw(
        &self,
        requests: &[RequestQueueRequest],
        forefront: bool,
    ) -> ApifyClientResult<BatchRequestsOperationResult> {
        let mut params = self.base_params();
        params.add_bool("forefront", Some(forefront));
        let body = serde_json::to_vec(requests)?;
        post_with_body(
            &self.ctx,
            Some("requests/batch"),
            &params,
            Some(body),
            "application/json",
        )
        .await
    }

    /// Deletes multiple requests in a single batch operation.
    ///
    /// Unlike [`batch_add_requests`](Self::batch_add_requests), this does not auto-chunk: the API
    /// accepts at most [`MAX_REQUESTS_PER_BATCH_OPERATION`] requests per call (matching the
    /// reference client, which validates rather than chunks), so a larger `requests` returns
    /// [`ApifyClientError::InvalidArgument`] before any request is sent.
    pub async fn batch_delete_requests<T: Serialize>(
        &self,
        requests: &[T],
    ) -> ApifyClientResult<BatchRequestsOperationResult> {
        if requests.is_empty() || requests.len() > MAX_REQUESTS_PER_BATCH_OPERATION {
            return Err(ApifyClientError::InvalidArgument(format!(
                "RequestQueueClient::batch_delete_requests accepts between 1 and {MAX_REQUESTS_PER_BATCH_OPERATION} requests per call, got {}",
                requests.len()
            )));
        }
        delete_with_body(
            &self.ctx,
            Some("requests/batch"),
            &self.base_params(),
            &requests,
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
    ) -> ApifyClientResult<RequestQueueRequestsPage> {
        let mut params = self.base_params();
        params
            .add_int("limit", options.limit)
            .add_str("exclusiveStartId", options.exclusive_start_id)
            .add_str("cursor", options.cursor)
            .add_csv("filter", options.filter.as_deref());
        get_resource_required(&self.ctx, Some("requests"), &params).await
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
    ) -> ApifyClientResult<RequestLockInfo> {
        let mut params = self.base_params();
        params
            .add_int("lockSecs", Some(lock_secs))
            .add_bool("forefront", Some(forefront));
        let url = params.apply_to_url(
            &self
                .ctx
                .url(Some(&format!("requests/{}/lock", encode_path_segment(id)))),
        );
        let response = self
            .ctx
            .http
            .call(HttpRequest {
                method: HttpMethod::Put,
                url,
                headers: Default::default(),
                body: None,
                timeout: crate::clients::base::MEDIUM_REQUEST_TIMEOUT,
            })
            .await?;
        crate::common::parse_data_envelope(&response.body)
    }

    /// Releases the lock on a request so other clients can process it.
    ///
    /// If `forefront` is `true`, the request moves to the front of the queue.
    pub async fn delete_request_lock(&self, id: &str, forefront: bool) -> ApifyClientResult<()> {
        let mut params = self.base_params();
        params.add_bool("forefront", Some(forefront));
        let url = params.apply_to_url(
            &self
                .ctx
                .url(Some(&format!("requests/{}/lock", encode_path_segment(id)))),
        );
        self.ctx
            .http
            .call(HttpRequest {
                method: HttpMethod::Delete,
                url,
                headers: Default::default(),
                body: None,
                timeout: crate::clients::base::SMALL_REQUEST_TIMEOUT,
            })
            .await?;
        Ok(())
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
    pub async fn unlock_requests(&self) -> ApifyClientResult<UnlockRequestsResult> {
        post_action(
            &self.ctx,
            Some("requests/unlock"),
            &self.base_params(),
            None,
            None,
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

        if page.items.is_empty() {
            self.exhausted = true;
            return Ok(None);
        }

        // Advance the cursor; stop when the API stops returning one.
        match page.next_cursor {
            Some(cursor) if !cursor.is_empty() => self.next_cursor = Some(cursor),
            _ => self.exhausted = true,
        }

        self.buffer.extend(page.items);
        Ok(self.buffer.pop_front())
    }
}

#[cfg(test)]
mod batch_add_tests {
    use super::{
        chunk_requests_for_batch_add, dedup_key, slice_requests_by_byte_length,
        MAX_REQUESTS_PER_BATCH_OPERATION,
    };
    use crate::models::RequestQueueRequest;

    fn request(url: &str, unique_key: Option<&str>) -> RequestQueueRequest {
        RequestQueueRequest {
            id: None,
            url: url.to_string(),
            unique_key: unique_key.map(str::to_string),
            method: None,
            user_data: None,
            extra: Default::default(),
        }
    }

    /// A request without an explicit `unique_key` is correlated by `url`, matching the API's own
    /// deduplication fallback.
    #[test]
    fn dedup_key_falls_back_to_url() {
        let with_key = request("https://example.com", Some("k1"));
        assert_eq!(dedup_key(&with_key), "k1");

        let without_key = request("https://example.com/no-key", None);
        assert_eq!(dedup_key(&without_key), "https://example.com/no-key");
    }

    /// A slice that already fits under the byte budget is returned unchanged.
    #[test]
    fn byte_slice_returns_everything_when_under_budget() {
        let requests: Vec<_> = (0..5)
            .map(|i| request(&format!("https://example.com/{i}"), None))
            .collect();
        let sliced = slice_requests_by_byte_length(&requests, 1_000_000, 0).unwrap();
        assert_eq!(sliced.len(), 5);
    }

    /// When the whole slice exceeds the byte budget, only a byte-limited prefix is taken — but
    /// never an empty one, even if the very first item alone leaves no room for a second.
    #[test]
    fn byte_slice_takes_a_limited_prefix() {
        let requests: Vec<_> = (0..10)
            .map(|i| request(&format!("https://example.com/{i}"), None))
            .collect();
        // Each request serializes to roughly 30 bytes; a budget of 50 fits one comfortably but
        // never two.
        let sliced = slice_requests_by_byte_length(&requests, 50, 0).unwrap();
        assert_eq!(
            sliced.len(),
            1,
            "budget of 50 bytes should admit exactly one ~30-byte request"
        );
    }

    /// A single request whose own JSON exceeds the byte budget is a hard error, not a silently
    /// dropped item — the caller could never send it in any chunk.
    #[test]
    fn byte_slice_errors_on_oversized_single_request() {
        let huge_url = format!("https://example.com/{}", "x".repeat(1000));
        let requests = vec![request(&huge_url, None)];
        let err = slice_requests_by_byte_length(&requests, 100, 3).unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("index 3"),
            "error should name the absolute index of the oversized request: {message}"
        );
    }

    /// Chunking respects the per-call count cap even when every request is tiny (byte budget is
    /// never the limiting factor).
    #[test]
    fn chunking_splits_by_count_when_bytes_are_plentiful() {
        let requests: Vec<_> = (0..(MAX_REQUESTS_PER_BATCH_OPERATION * 2 + 3))
            .map(|i| request(&format!("https://example.com/{i}"), None))
            .collect();
        let chunks = chunk_requests_for_batch_add(&requests, 10_000_000).unwrap();
        let sizes: Vec<usize> = chunks.iter().map(Vec::len).collect();
        assert_eq!(
            sizes,
            vec![
                MAX_REQUESTS_PER_BATCH_OPERATION,
                MAX_REQUESTS_PER_BATCH_OPERATION,
                3
            ]
        );
        let total: usize = sizes.iter().sum();
        assert_eq!(total, requests.len());
    }

    /// Chunking also respects the byte budget, producing more (smaller) chunks than the count
    /// cap alone would when requests are large.
    #[test]
    fn chunking_splits_by_byte_budget_when_tighter_than_count_cap() {
        let requests: Vec<_> = (0..6)
            .map(|i| request(&format!("https://example.com/{i}"), None))
            .collect();
        // ~30 bytes/request; a 100-byte budget forces multiple chunks well under the 25-item cap.
        let chunks = chunk_requests_for_batch_add(&requests, 100).unwrap();
        assert!(
            chunks.len() > 1,
            "a tight byte budget must force more than one chunk, got {}",
            chunks.len()
        );
        let total: usize = chunks.iter().map(Vec::len).sum();
        assert_eq!(
            total,
            requests.len(),
            "every request must end up in exactly one chunk"
        );
    }
}
