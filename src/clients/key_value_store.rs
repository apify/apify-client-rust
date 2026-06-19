//! Client for a single key-value store (`/v2/key-value-stores/{storeId}` and variants).

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
use crate::models::{KeyValueStore, KeyValueStoreKeysPage, KeyValueStoreRecord};

/// Options for listing keys in a key-value store.
#[derive(Debug, Default, Clone)]
pub struct ListKeysOptions {
    /// Maximum number of keys to return.
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
