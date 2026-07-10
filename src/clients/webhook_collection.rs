//! Client for the webhook collection (`/v2/webhooks`, `/v2/actors/{id}/webhooks`, etc.).

use serde::Serialize;

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::clients::pagination::{list_iterator, ListIterator};
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
    ///
    /// `options.limit` caps the *total* number of items yielded across all pages, unlike
    /// [`list`](Self::list) where `limit` is a single page's size. Set the per-page fetch size
    /// with [`with_chunk_size`](crate::ListIterator::with_chunk_size); see
    /// [`ListIterator`] for details.
    pub fn iterate(&self, options: ListOptions) -> ListIterator<Webhook> {
        list_iterator!(self, options, list)
    }

    /// Creates a new webhook from the given definition.
    pub async fn create<T: Serialize>(&self, webhook: &T) -> ApifyClientResult<Webhook> {
        create_resource(&self.ctx, &QueryParams::new(), webhook).await
    }
}
