//! Client for the key-value store collection (`/v2/key-value-stores`).

use crate::clients::base::{get_or_create_named_with_schema, list_resource, ResourceContext};
use crate::clients::pagination::{list_iterator, ListIterator};
use crate::common::{PaginationList, QueryParams, StorageListOptions};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::KeyValueStore;

/// Options for getting-or-creating a key-value store via
/// [`KeyValueStoreCollectionClient::get_or_create_with_options`].
///
/// `schema` is a JS-reference-only convenience (not documented by the OpenAPI spec, which only
/// declares the `name` query parameter on `POST /v2/key-value-stores`); it is sent as the
/// request's `{ "schema": ... }` JSON body, matching the reference client's
/// `KeyValueStoreCollectionClientGetOrCreateOptions`.
#[derive(Debug, Default, Clone)]
pub struct KeyValueStoreGetOrCreateOptions {
    /// JSON schema to associate with the key-value store.
    pub schema: Option<serde_json::Value>,
}

/// Client for listing key-value stores and getting-or-creating one by name.
#[derive(Debug, Clone)]
pub struct KeyValueStoreCollectionClient {
    ctx: ResourceContext,
}

impl KeyValueStoreCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "key-value-stores"),
        }
    }

    /// Lists key-value stores with offset/limit pagination, optionally filtering by
    /// `unnamed`/`ownership`.
    pub async fn list(
        &self,
        options: StorageListOptions,
    ) -> ApifyClientResult<PaginationList<KeyValueStore>> {
        let mut params = QueryParams::new();
        options.apply(&mut params);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all key-value stores matching `options`, fetching pages on demand.
    ///
    /// `options.limit` caps the *total* number of items yielded across all pages, unlike
    /// [`list`](Self::list) where `limit` is a single page's size. Set the per-page fetch size
    /// with [`with_chunk_size`](crate::ListIterator::with_chunk_size); see
    /// [`ListIterator`] for details.
    pub fn iterate(&self, options: StorageListOptions) -> ListIterator<KeyValueStore> {
        list_iterator!(self, options, list)
    }

    /// Gets the store with the given `name`, creating it if it does not exist.
    pub async fn get_or_create(&self, name: Option<&str>) -> ApifyClientResult<KeyValueStore> {
        self.get_or_create_with_options(name, KeyValueStoreGetOrCreateOptions::default())
            .await
    }

    /// Gets the store with the given `name`, creating it if it does not exist, applying the
    /// given [`KeyValueStoreGetOrCreateOptions`] (e.g.
    /// [`KeyValueStoreGetOrCreateOptions::schema`], applied only when the store is created).
    pub async fn get_or_create_with_options(
        &self,
        name: Option<&str>,
        options: KeyValueStoreGetOrCreateOptions,
    ) -> ApifyClientResult<KeyValueStore> {
        get_or_create_named_with_schema(&self.ctx, name, options.schema.as_ref()).await
    }
}
