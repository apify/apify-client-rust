//! Client for the key-value store collection (`/v2/key-value-stores`).

use crate::clients::base::{get_or_create_named, list_resource, ResourceContext};
use crate::common::{ListOptions, PaginationList, QueryParams};
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

    /// Lists key-value stores with offset/limit pagination.
    pub async fn list(
        &self,
        options: ListOptions,
    ) -> ApifyClientResult<PaginationList<KeyValueStore>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc);
        list_resource(&self.ctx, None, &params).await
    }

    /// Gets the store with the given `name`, creating it if it does not exist.
    pub async fn get_or_create(&self, name: Option<&str>) -> ApifyClientResult<KeyValueStore> {
        get_or_create_named(&self.ctx, name).await
    }
}
