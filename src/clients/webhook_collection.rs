//! Client for the webhook collection (`/v2/webhooks`, `/v2/actors/{id}/webhooks`, etc.).

use serde::Serialize;

use crate::clients::base::{create_resource, list_resource, ResourceContext};
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

    /// Creates a new webhook from the given definition.
    pub async fn create<T: Serialize>(&self, webhook: &T) -> ApifyClientResult<Webhook> {
        create_resource(&self.ctx, &QueryParams::new(), webhook).await
    }
}
