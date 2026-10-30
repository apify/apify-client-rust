//! Client for a single Actor build (`/v2/actor-builds/{buildId}`).

use crate::clients::base::{
    delete_resource, get_resource, post_action, wait_for_finish, ResourceContext,
};
use crate::clients::log::LogClient;
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Build;
use crate::timeouts::Timeout;

/// Client for a specific Actor build.
#[derive(Debug, Clone)]
pub struct BuildClient {
    ctx: ResourceContext,
}

crate::clients::base::impl_with_timeout!(BuildClient);

impl BuildClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "actor-builds", id),
        }
    }

    /// Fetches the build object, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Build>> {
        get_resource(&self.ctx, None, &QueryParams::new(), Timeout::Short).await
    }

    /// Aborts the build.
    pub async fn abort(&self) -> ApifyClientResult<Build> {
        post_action(
            &self.ctx,
            Some("abort"),
            &QueryParams::new(),
            None,
            None,
            self.ctx.timeout(Timeout::Short),
        )
        .await
    }

    /// Deletes the build.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None, Timeout::Short).await
    }

    /// Waits (by client-side polling) for the build to reach a terminal state.
    ///
    /// `wait_secs` controls the wait budget:
    /// - `None` polls indefinitely until the build reaches a terminal state.
    /// - `Some(n)` bounds the wait to roughly `n` seconds; if the build has not finished by
    ///   then, the **last fetched (still non-terminal) build is returned** rather than an
    ///   error. Check `status` / `is_terminal()` on the result when using `Some`.
    ///
    /// With `None`, the polls have no client-side timeout. With `Some(n)`, each poll is allowed the
    /// `Medium` tier plus the hold (up to a minute) it asks the API for, so a stalled connection
    /// cannot hang the wait. A timeout set with `with_timeout` is used as given instead.
    pub async fn wait_for_finish(&self, wait_secs: Option<i64>) -> ApifyClientResult<Build> {
        wait_for_finish(&self.ctx, wait_secs, |b: &Build| b.is_terminal()).await
    }

    /// Retrieves the OpenAPI definition generated for this build.
    ///
    /// Corresponds to `GET /v2/actor-builds/{buildId}/openapi.json`. The response is a raw
    /// OpenAPI document (not wrapped in a `data` envelope), returned as JSON. A build that does
    /// not exist, or has no OpenAPI definition, is a [`NotFound`](crate::ApiError::is_not_found)
    /// error: the response cannot tell the two apart.
    pub async fn get_openapi_definition(&self) -> ApifyClientResult<serde_json::Value> {
        let response = crate::clients::base::get_raw_required(
            &self.ctx,
            Some("openapi.json"),
            &QueryParams::new(),
            Timeout::Medium,
        )
        .await?;
        Ok(serde_json::from_slice(&response.body)?)
    }

    /// Returns a client for the build's log.
    pub fn log(&self) -> LogClient {
        LogClient::nested(
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "log",
            &QueryParams::new(),
        )
    }
}
