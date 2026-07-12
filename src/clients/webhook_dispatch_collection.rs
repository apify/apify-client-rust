//! Client for the webhook dispatch collection (`/v2/webhook-dispatches`).

use crate::clients::base::{list_resource, ResourceContext};
use crate::clients::pagination::{list_iterator, ListIterator};
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
    ///
    /// `options.limit` caps the *total* number of items yielded across all pages, unlike
    /// [`list`](Self::list) where `limit` is a single page's size. Set the per-page fetch size
    /// with [`with_chunk_size`](crate::ListIterator::with_chunk_size); see
    /// [`ListIterator`] for details.
    pub fn iterate(&self, options: ListOptions) -> ListIterator<WebhookDispatch> {
        list_iterator!(self, options, list)
    }
}
