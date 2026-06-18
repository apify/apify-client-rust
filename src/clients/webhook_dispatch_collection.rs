//! Client for the webhook dispatch collection (`/v2/webhook-dispatches`).

use crate::clients::base::{list_resource, ResourceContext};
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
}
