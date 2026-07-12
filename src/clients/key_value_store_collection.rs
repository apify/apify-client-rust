//! Client for the key-value store collection (`/v2/key-value-stores`).

use crate::clients::base::{get_or_create_named, list_resource, ResourceContext};
use crate::clients::pagination::{list_iterator, ListIterator};
use crate::common::{PaginationList, QueryParams, StorageListOptions};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::KeyValueStore;

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
        get_or_create_named(&self.ctx, name).await
    }
}
