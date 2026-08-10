//! Client for a single webhook (`/v2/webhooks/{webhookId}`).

use serde::Serialize;

use crate::clients::base::{
    delete_resource, get_resource, post_action_optional, update_resource, ResourceContext,
};
use crate::clients::webhook_dispatch_collection::WebhookDispatchCollectionClient;
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::{Webhook, WebhookDispatch};

/// Client for a specific webhook.
#[derive(Debug, Clone)]
pub struct WebhookClient {
    ctx: ResourceContext,
}

impl WebhookClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "webhooks", id),
        }
    }

    /// Fetches the webhook, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Webhook>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the webhook with the given fields.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<Webhook> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the webhook.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Tests the webhook by dispatching it immediately, returning the dispatch, or `None` if the
    /// webhook no longer exists (e.g. it was deleted concurrently). The spec lists `404` as a
    /// valid response for `POST /v2/webhooks/{webhookId}/test`, and the reference client wraps
    /// the call in `catchNotFoundOrThrow`.
    pub async fn test(&self) -> ApifyClientResult<Option<WebhookDispatch>> {
        post_action_optional(&self.ctx, Some("test"), &QueryParams::new(), None, None).await
    }

    /// Returns a client for this webhook's dispatch collection.
    pub fn dispatches(&self) -> WebhookDispatchCollectionClient {
        WebhookDispatchCollectionClient::with_base(
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "dispatches",
        )
    }
}
