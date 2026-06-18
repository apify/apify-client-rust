//! Client for a single request queue (`/v2/request-queues/{queueId}` and variants).

use serde::Serialize;

use crate::clients::base::{
    delete_resource, delete_with_body, get_resource, get_resource_required, post_action,
    post_with_body, update_resource, ResourceContext,
};
use crate::common::{encode_path_segment, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, HttpMethod, HttpRequest};
use crate::models::{
    RequestQueue, RequestQueueHead, RequestQueueOperationInfo, RequestQueueRequest,
};

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
    /// Server-side filter expression for the returned requests.
    pub filter: Option<String>,
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
    ) -> ApifyClientResult<serde_json::Value> {
        let mut params = self.base_params();
        params
            .add_int("lockSecs", Some(lock_secs))
            .add_int("limit", limit);
        post_action(&self.ctx, Some("head/lock"), &params, None, None).await
    }

    /// Adds multiple requests to the queue in a single batch operation.
    pub async fn batch_add_requests(
        &self,
        requests: &[RequestQueueRequest],
        forefront: bool,
    ) -> ApifyClientResult<serde_json::Value> {
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
    pub async fn batch_delete_requests<T: Serialize>(
        &self,
        requests: &[T],
    ) -> ApifyClientResult<serde_json::Value> {
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
    ) -> ApifyClientResult<serde_json::Value> {
        let mut params = self.base_params();
        params
            .add_int("limit", options.limit)
            .add_str("exclusiveStartId", options.exclusive_start_id)
            .add_str("cursor", options.cursor)
            .add_str("filter", options.filter);
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
    ) -> ApifyClientResult<serde_json::Value> {
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
    pub async fn unlock_requests(&self) -> ApifyClientResult<serde_json::Value> {
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
