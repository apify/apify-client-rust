//! Client for the webhook collection (`/v2/webhooks`, `/v2/actors/{id}/webhooks`, etc.).

use serde::Serialize;

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{ListOptions, PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Webhook;

/// Client for listing and creating webhooks.
#[derive(Debug, Clone)]
pub struct WebhookCollectionClient {
    ctx: ResourceContext,
}

impl WebhookCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "webhooks"),
        }
    }

    /// Creates a webhook collection client nested under another resource.
    pub(crate) fn with_base(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "webhooks"),
        }
    }

    /// Lists webhooks with offset/limit pagination.
    pub async fn list(&self, options: ListOptions) -> ApifyClientResult<PaginationList<Webhook>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all webhooks matching `options`, fetching pages on demand.
    pub fn iterate(&self, options: ListOptions) -> ListIterator<Webhook> {
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

    /// Creates a new webhook from the given definition.
    pub async fn create<T: Serialize>(&self, webhook: &T) -> ApifyClientResult<Webhook> {
        create_resource(&self.ctx, &QueryParams::new(), webhook).await
    }
}
