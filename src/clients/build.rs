//! Client for a single Actor build (`/v2/actor-builds/{buildId}`).

use crate::clients::base::{
    delete_resource, get_resource, post_action, wait_for_finish, ResourceContext,
};
use crate::clients::log::LogClient;
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Build;

/// Client for a specific Actor build.
#[derive(Debug, Clone)]
pub struct BuildClient {
    ctx: ResourceContext,
}

impl BuildClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "actor-builds", id),
        }
    }

    /// Fetches the build object, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Build>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Aborts the build.
    pub async fn abort(&self) -> ApifyClientResult<Build> {
        post_action(&self.ctx, Some("abort"), &QueryParams::new(), None, None).await
    }

    /// Deletes the build.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Waits (by client-side polling) for the build to reach a terminal state.
    pub async fn wait_for_finish(&self, wait_secs: Option<i64>) -> ApifyClientResult<Build> {
        wait_for_finish(&self.ctx, wait_secs, |b: &Build| b.is_terminal()).await
    }

    /// Retrieves the OpenAPI definition generated for this build, if available.
    ///
    /// Corresponds to `GET /v2/actor-builds/{buildId}/openapi.json`. The response is a raw
    /// OpenAPI document (not wrapped in a `data` envelope), returned as JSON.
    pub async fn get_openapi_definition(&self) -> ApifyClientResult<Option<serde_json::Value>> {
        let response =
            crate::clients::base::get_raw(&self.ctx, Some("openapi.json"), &QueryParams::new())
                .await?;
        match response {
            Some(r) => Ok(Some(serde_json::from_slice(&r.body)?)),
            None => Ok(None),
        }
    }

    /// Returns a client for the build's log.
    pub fn log(&self) -> LogClient {
        LogClient::nested(self.ctx.http.clone(), &self.ctx.url(None), "log")
    }
}
