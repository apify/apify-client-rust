//! Client for the request queue collection (`/v2/request-queues`).

use crate::clients::base::{get_or_create_named, list_resource, ResourceContext};
use crate::common::{ListOptions, PaginationList, QueryParams};
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

    /// Lists request queues with offset/limit pagination.
    pub async fn list(
        &self,
        options: ListOptions,
    ) -> ApifyClientResult<PaginationList<RequestQueue>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc);
        list_resource(&self.ctx, None, &params).await
    }

    /// Gets the queue with the given `name`, creating it if it does not exist.
    pub async fn get_or_create(&self, name: Option<&str>) -> ApifyClientResult<RequestQueue> {
        get_or_create_named(&self.ctx, name).await
    }
}
