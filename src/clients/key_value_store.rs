//! Client for a single key-value store (`/v2/key-value-stores/{storeId}` and variants).

use std::collections::VecDeque;

use serde::Serialize;

use crate::clients::base::{
    delete_resource, get_raw, get_resource, get_resource_required, head_exists, put_raw,
    update_resource, ResourceContext,
};
use crate::common::{
    create_hmac_signature, encode_path_segment, sign_storage_content, QueryParams,
};
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, HttpMethod, HttpRequest};
use crate::models::{KeyValueStore, KeyValueStoreKey, KeyValueStoreKeysPage, KeyValueStoreRecord};

/// Options for listing keys in a key-value store.
#[derive(Debug, Default, Clone)]
pub struct ListKeysOptions {
    /// Key limit. Its meaning depends on the method: for [`KeyValueStoreClient::list_keys`] it is a
    /// single page's size (max keys one call returns, capped at 1000 by the API); for
    /// [`KeyValueStoreClient::iterate_keys`] it is a cap on the *total* number of keys yielded
    /// across all pages (unset/`0` iterates the whole store).
    pub limit: Option<i64>,
    /// Start listing after this key (exclusive), for pagination.
    pub exclusive_start_key: Option<String>,
    /// Only return keys with this prefix.
    pub prefix: Option<String>,
    /// Only return keys belonging to this collection.
    pub collection: Option<String>,
    /// URL-signing signature granting access to a private store's key listing.
    pub signature: Option<String>,
}

/// Options for downloading all records as a ZIP archive via
/// [`KeyValueStoreClient::get_records`].
///
/// Covers the spec query parameters of `GET /v2/key-value-stores/{storeId}/records`.
#[derive(Debug, Default, Clone)]
pub struct GetRecordsOptions {
    /// Only include records belonging to this collection from the store schema.
    pub collection: Option<String>,
    /// Only include records whose key starts with this prefix.
    pub prefix: Option<String>,
    /// URL-signing signature granting access to a private store's records.
    pub signature: Option<String>,
}

/// Options for reading a single record via [`KeyValueStoreClient::get_record_with_options`].
///
/// Covers the spec query parameters of
/// `GET /v2/key-value-stores/{storeId}/records/{recordKey}`.
#[derive(Debug, Default, Clone)]
pub struct GetRecordOptions {
    /// Request the record with a `Content-Disposition: attachment` response header. When unset,
    /// the client sends `attachment=true`, matching the reference client's unconditional default.
    pub attachment: Option<bool>,
    /// URL-signing signature granting access to a record in a private store.
    pub signature: Option<String>,
}

/// Client for a specific key-value store.
#[derive(Debug, Clone)]
pub struct KeyValueStoreClient {
    ctx: ResourceContext,
}

impl KeyValueStoreClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, resource_path: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, resource_path, id),
        }
    }

    /// Creates a KVS client for a run's default store (nested path, no ID).
    pub(crate) fn nested(http: HttpClient, base_url: &str, sub_path: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, sub_path),
        }
    }

    /// Sets the public origin used when building shareable URLs.
    pub(crate) fn with_public_base(mut self, public_base_url: &str) -> Self {
        self.ctx = self.ctx.with_public_origin(public_base_url);
        self
    }

    /// Fetches the store metadata, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<KeyValueStore>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the store metadata (e.g. `name`, `title`).
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<KeyValueStore> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the store.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Lists the keys in the store (key-based pagination).
    pub async fn list_keys(
        &self,
        options: ListKeysOptions,
    ) -> ApifyClientResult<KeyValueStoreKeysPage> {
        let mut params = QueryParams::new();
        params
            .add_int("limit", options.limit)
            .add_str("exclusiveStartKey", options.exclusive_start_key)
            .add_str("prefix", options.prefix)
            .add_str("collection", options.collection)
            .add_str("signature", options.signature);
        get_resource_required(&self.ctx, Some("keys"), &params).await
    }

    /// Lazily iterates over every key in the store, fetching pages on demand.
    ///
    /// Returns a [`KeyValueStoreKeysIterator`]; call its `next()` to get one key at a time,
    /// transparently fetching the following page once the local buffer drains, until the store
    /// is exhausted. This is the auto-paginating counterpart to the single-page
    /// [`list_keys`](Self::list_keys), matching the reference client's `listKeys()`
    /// `AsyncIterable`.
    ///
    /// Key-value stores use cursor-based (not offset) pagination: each page is anchored by the
    /// previous page's `nextExclusiveStartKey`, so the iterator threads that cursor through
    /// automatically. The `prefix`, `collection` and `signature` filters from `options` are
    /// carried into every page.
    ///
    /// `options.limit` caps the *total* number of keys yielded across all pages; leaving it unset
    /// (or `0`) iterates the entire store, matching the reference client. It is honoured across as
    /// many pages as needed — each individual request is bounded to the endpoint's maximum page
    /// size ([`KEY_LIST_MAX_LIMIT`]), so a cap larger than one page still works.
    /// `options.exclusive_start_key`, when set, resumes iteration after that key.
    pub fn iterate_keys(&self, options: ListKeysOptions) -> KeyValueStoreKeysIterator {
        let remaining = options.limit.filter(|&l| l > 0);
        KeyValueStoreKeysIterator {
            client: self.clone(),
            options,
            remaining,
            next_exclusive_start_key: None,
            buffer: VecDeque::new(),
            first_page: true,
            exhausted: false,
        }
    }

    /// Downloads all records from the store as a ZIP archive (raw bytes).
    ///
    /// Each record is stored as a separate file in the archive, with the filename equal to the
    /// record key. Use [`GetRecordsOptions`] to filter by `collection` or `prefix`, or to pass a
    /// URL-signing `signature` for a private store. Wraps
    /// `GET /v2/key-value-stores/{storeId}/records`.
    pub async fn get_records(&self, options: GetRecordsOptions) -> ApifyClientResult<Vec<u8>> {
        let mut params = QueryParams::new();
        params
            .add_str("collection", options.collection)
            .add_str("prefix", options.prefix)
            .add_str("signature", options.signature);
        let url = self
            .ctx
            .merged_params(&params)
            .apply_to_url(&self.ctx.url(Some("records")));
        let response = self
            .ctx
            .http
            .call(HttpRequest {
                method: HttpMethod::Get,
                url,
                headers: Default::default(),
                body: None,
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        Ok(response.body)
    }

    /// Returns `true` if a record with the given key exists.
    pub async fn record_exists(&self, key: &str) -> ApifyClientResult<bool> {
        head_exists(
            &self.ctx,
            Some(&format!("records/{}", encode_path_segment(key))),
            &QueryParams::new(),
        )
        .await
    }

    /// Gets a record's raw value (and content type), or `None` if it does not exist.
    ///
    /// Like the reference client's `getRecord`, this sends `attachment=true`. Use
    /// [`get_record_with_options`](Self::get_record_with_options) to override the attachment
    /// behaviour or to pass a URL-signing `signature` for a private store.
    pub async fn get_record(&self, key: &str) -> ApifyClientResult<Option<KeyValueStoreRecord>> {
        self.get_record_with_options(key, GetRecordOptions::default())
            .await
    }

    /// Gets a record with explicit options.
    ///
    /// The `attachment` option controls the `Content-Disposition: attachment` response header;
    /// when unset it defaults to `true`, matching the reference client's unconditional behaviour.
    /// `signature` supplies a URL-signing signature for accessing a record in a private store.
    pub async fn get_record_with_options(
        &self,
        key: &str,
        options: GetRecordOptions,
    ) -> ApifyClientResult<Option<KeyValueStoreRecord>> {
        let mut params = QueryParams::new();
        // Default to `attachment=true` (reference parity); honour an explicit override.
        params
            .add_bool("attachment", Some(options.attachment.unwrap_or(true)))
            .add_str("signature", options.signature);
        let response = get_raw(
            &self.ctx,
            Some(&format!("records/{}", encode_path_segment(key))),
            &params,
        )
        .await?;
        Ok(response.map(|r| {
            let content_type = r.header("content-type").map(|s| s.to_string());
            KeyValueStoreRecord {
                key: key.to_string(),
                value: r.body,
                content_type,
            }
        }))
    }

    /// Stores a record with raw bytes and an explicit content type.
    pub async fn set_record_raw(
        &self,
        key: &str,
        value: Vec<u8>,
        content_type: &str,
    ) -> ApifyClientResult<()> {
        put_raw(
            &self.ctx,
            Some(&format!("records/{}", encode_path_segment(key))),
            &QueryParams::new(),
            value,
            content_type,
        )
        .await
    }

    /// Stores a record as JSON (the value is serialized and content type set to JSON).
    pub async fn set_record_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> ApifyClientResult<()> {
        let bytes = serde_json::to_vec(value)?;
        self.set_record_raw(key, bytes, "application/json; charset=utf-8")
            .await
    }

    /// Builds a public URL for reading the record with the given key.
    ///
    /// Mirrors the reference client's `getRecordPublicUrl`: it fetches the store, and if the
    /// store exposes a URL-signing secret key (private store), appends an HMAC-SHA256
    /// `signature` over the record key so the URL works without an API token. The URL is
    /// built from the configured public base URL.
    pub async fn get_record_public_url(&self, key: &str) -> ApifyClientResult<String> {
        let mut params = QueryParams::new();
        if let Some(store) = self.get().await? {
            if let Some(secret) = store
                .extra
                .get("urlSigningSecretKey")
                .and_then(|v| v.as_str())
            {
                params.add_str("signature", Some(create_hmac_signature(secret, key)));
            }
        }
        Ok(params.apply_to_url(
            &self
                .ctx
                .public_url(Some(&format!("records/{}", encode_path_segment(key)))),
        ))
    }

    /// Builds a public URL for listing this store's keys.
    ///
    /// Like [`get_record_public_url`](Self::get_record_public_url), signs the URL with an
    /// HMAC-SHA256 `signature` for private stores. `expires_in_secs` optionally bounds a
    /// signed URL's validity.
    pub async fn create_keys_public_url(
        &self,
        expires_in_secs: Option<i64>,
    ) -> ApifyClientResult<String> {
        let mut params = QueryParams::new();
        if let Some(store) = self.get().await? {
            if let Some(secret) = store
                .extra
                .get("urlSigningSecretKey")
                .and_then(|v| v.as_str())
            {
                let signature = sign_storage_content(secret, &store.id, expires_in_secs);
                params.add_str("signature", Some(signature));
            }
        }
        Ok(params.apply_to_url(&self.ctx.public_url(Some("keys"))))
    }

    /// Deletes the record with the given key.
    pub async fn delete_record(&self, key: &str) -> ApifyClientResult<()> {
        let url = self
            .ctx
            .url(Some(&format!("records/{}", encode_path_segment(key))));
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
}

/// The maximum number of keys the `GET /v2/key-value-stores/{storeId}/keys` endpoint accepts in
/// its `limit` query parameter (per the OpenAPI spec: `minimum: 1, maximum: 1000`). Each page the
/// key iterator requests is bounded to this value so a large total cap still paginates correctly
/// instead of asking the API for an out-of-range `limit`.
pub const KEY_LIST_MAX_LIMIT: i64 = 1000;

/// A lazy, page-fetching async iterator over the keys in a key-value store.
///
/// Created by [`KeyValueStoreClient::iterate_keys`]. Each call to [`next`](Self::next) returns
/// the next key, fetching another page from the API when the local buffer is exhausted, until
/// every key has been yielded (or the caller's total-key cap is reached).
///
/// Unlike the offset/limit-paginated [`ListIterator`](crate::ListIterator), key-value stores use
/// cursor-based pagination: each page is anchored by the previous page's
/// `nextExclusiveStartKey`. Termination mirrors the reference client's `listKeys()` generator —
/// the walk stops once a page comes back empty, the API stops returning a next cursor, or the
/// caller's `limit` is exhausted.
pub struct KeyValueStoreKeysIterator {
    client: KeyValueStoreClient,
    /// Base listing options. The `prefix`/`collection`/`signature` filters are carried into every
    /// page unchanged; `limit` and `exclusive_start_key` are overridden per page after the first.
    options: ListKeysOptions,
    /// Keys still allowed under the caller's total cap (`options.limit`); `None` = uncapped.
    /// Decremented by each page's key count. Each request asks for `min(remaining,
    /// KEY_LIST_MAX_LIMIT)` so the cap is honoured across pages without exceeding the endpoint's
    /// maximum `limit`.
    remaining: Option<i64>,
    /// Cursor for the next page: the previous page's `next_exclusive_start_key`.
    next_exclusive_start_key: Option<String>,
    buffer: VecDeque<KeyValueStoreKey>,
    /// `true` until the first page has been fetched. Only the first page honours the caller's
    /// `exclusive_start_key`; later pages are driven by the cursor. The request `limit` is derived
    /// from `remaining` on every page (never sent verbatim), so a `limit` of `0`/unset is treated
    /// as "no limit" rather than sending an out-of-range `limit=0`.
    first_page: bool,
    exhausted: bool,
}

impl KeyValueStoreKeysIterator {
    /// Returns the next key, or `None` when the store is exhausted (or the caller's `limit` is
    /// reached). Fetches another page from the API when the local buffer is empty.
    pub async fn next(&mut self) -> ApifyClientResult<Option<KeyValueStoreKey>> {
        if let Some(item) = self.buffer.pop_front() {
            return Ok(Some(item));
        }
        if self.exhausted {
            return Ok(None);
        }

        // Build this page's options. The request `limit` is always derived from the remaining
        // budget (never the caller's raw `limit`), clamped to the endpoint maximum: this normalizes
        // an unset/`0` cap to "no limit" and keeps a large finite cap within the accepted range so
        // it paginates instead of 400-ing. Only the first page uses the caller's
        // `exclusive_start_key`; later pages advance the cursor.
        let mut page_options = self.options.clone();
        page_options.limit = self.remaining.map(|rem| rem.min(KEY_LIST_MAX_LIMIT));
        if !self.first_page {
            page_options.exclusive_start_key = self.next_exclusive_start_key.clone();
        }
        self.first_page = false;

        let page = self.client.list_keys(page_options).await?;
        let mut items = page.items;
        let received = items.len() as i64;

        // Enforce the caller's total cap exactly, even if the API returns more than requested
        // (defensive parity with `ListIterator`).
        if let Some(rem) = self.remaining {
            if received > rem {
                items.truncate(rem.max(0) as usize);
            }
        }
        if let Some(rem) = self.remaining.as_mut() {
            *rem -= received;
        }
        self.next_exclusive_start_key = page.next_exclusive_start_key;

        // Stop when the page is empty, the API returns no further cursor, or the caller's cap is
        // reached — the same three termination conditions as the reference `listKeys()` loop.
        if received == 0
            || self.next_exclusive_start_key.is_none()
            || matches!(self.remaining, Some(r) if r <= 0)
        {
            self.exhausted = true;
        }

        self.buffer.extend(items);
        Ok(self.buffer.pop_front())
    }
}
