//! Client for an Actor-build collection (`/v2/actor-builds`, `/v2/actors/{id}/builds`).

use crate::clients::base::{list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{ListOptions, PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Build;

/// Client for listing Actor builds.
#[derive(Debug, Clone)]
pub struct BuildCollectionClient {
    ctx: ResourceContext,
}

impl BuildCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "actor-builds"),
        }
    }

    /// Creates a build collection client nested under another resource (e.g. an Actor).
    pub(crate) fn with_base(http: HttpClient, base_url: &str, resource_path: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, resource_path),
        }
    }

    /// Lists builds with offset/limit pagination.
    pub async fn list(&self, options: ListOptions) -> ApifyClientResult<PaginationList<Build>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all builds matching `options`, fetching pages on demand.
    pub fn iterate(&self, options: ListOptions) -> ListIterator<Build> {
        let client = self.clone();
        let start = options.offset.unwrap_or(0);
        ListIterator::new(
            start,
            Box::new(move |offset| {
                let client = client.clone();
                let mut options = options.clone();
                options.offset = Some(offset);
                Box::pin(async move { client.list(options).await })
            }),
        )
    }
}
