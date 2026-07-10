//! Client for an Actor's version collection (`/v2/actors/{actorId}/versions`).

use serde::Serialize;

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::clients::pagination::{list_iterator, ListIterator};
use crate::common::{ListOptions, PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorVersion;

/// Client for listing and creating Actor versions.
#[derive(Debug, Clone)]
pub struct ActorVersionCollectionClient {
    ctx: ResourceContext,
}

impl ActorVersionCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "versions"),
        }
    }

    /// Lists the Actor's versions.
    pub async fn list(
        &self,
        options: ListOptions,
    ) -> ApifyClientResult<PaginationList<ActorVersion>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all versions matching `options`, fetching pages on demand.
    ///
    /// `options.limit` caps the *total* number of items yielded across all pages, unlike
    /// [`list`](Self::list) where `limit` is a single page's size. Set the per-page fetch size
    /// with [`with_chunk_size`](crate::ListIterator::with_chunk_size); see
    /// [`ListIterator`] for details.
    pub fn iterate(&self, options: ListOptions) -> ListIterator<ActorVersion> {
        list_iterator!(self, options, list)
    }

    /// Creates a new Actor version.
    pub async fn create<T: Serialize>(&self, version: &T) -> ApifyClientResult<ActorVersion> {
        create_resource(&self.ctx, &QueryParams::new(), version).await
    }
}
