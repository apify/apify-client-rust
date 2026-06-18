//! Client for a single webhook dispatch (`/v2/webhook-dispatches/{dispatchId}`).

use crate::clients::base::{get_resource, ResourceContext};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::WebhookDispatch;

/// Client for a specific webhook dispatch.
#[derive(Debug, Clone)]
pub struct WebhookDispatchClient {
    ctx: ResourceContext,
}

impl WebhookDispatchClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "webhook-dispatches", id),
        }
    }

    /// Fetches the webhook dispatch, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<WebhookDispatch>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }
}
