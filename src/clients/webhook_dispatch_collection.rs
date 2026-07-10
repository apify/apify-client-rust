//! Client for the webhook dispatch collection (`/v2/webhook-dispatches`).

use crate::clients::base::{list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{ListOptions, PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::WebhookDispatch;

/// Client for listing webhook dispatches.
#[derive(Debug, Clone)]
pub struct WebhookDispatchCollectionClient {
    ctx: ResourceContext,
}

impl WebhookDispatchCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "webhook-dispatches"),
        }
    }

    /// Creates a dispatch collection client nested under a webhook.
    pub(crate) fn with_base(http: HttpClient, base_url: &str, resource_path: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, resource_path),
        }
    }

    /// Lists webhook dispatches with offset/limit pagination.
    pub async fn list(
        &self,
        options: ListOptions,
    ) -> ApifyClientResult<PaginationList<WebhookDispatch>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all webhook dispatches matching `options`, fetching pages on demand.
    pub fn iterate(&self, options: ListOptions) -> ListIterator<WebhookDispatch> {
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
}
