//! Client for the request queue collection (`/v2/request-queues`).

use crate::clients::base::{get_or_create_named, list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{PaginationList, QueryParams, StorageListOptions};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::RequestQueue;

/// Client for listing request queues and getting-or-creating one by name.
#[derive(Debug, Clone)]
pub struct RequestQueueCollectionClient {
    ctx: ResourceContext,
}

impl RequestQueueCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "request-queues"),
        }
    }

    /// Lists request queues with offset/limit pagination, optionally filtering by
    /// `unnamed`/`ownership`.
    pub async fn list(
        &self,
        options: StorageListOptions,
    ) -> ApifyClientResult<PaginationList<RequestQueue>> {
        let mut params = QueryParams::new();
        options.apply(&mut params);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all request queues matching `options`, fetching pages on demand.
    pub fn iterate(&self, options: StorageListOptions) -> ListIterator<RequestQueue> {
        let client = self.clone();
        let start = options.offset.unwrap_or(0);
        let total_limit = options.limit;
        ListIterator::new(
            start,
            total_limit,
            Box::new(move |offset, page_limit| {
                let client = client.clone();
                let mut options = options.clone();
                options.offset = Some(offset);
                options.limit = page_limit;
                Box::pin(async move { client.list(options).await })
            }),
        )
    }

    /// Gets the queue with the given `name`, creating it if it does not exist.
    pub async fn get_or_create(&self, name: Option<&str>) -> ApifyClientResult<RequestQueue> {
        get_or_create_named(&self.ctx, name).await
    }
}
