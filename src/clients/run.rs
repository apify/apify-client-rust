//! Client for a single Actor run (`/v2/actor-runs/{runId}` and nested routes).

use serde::Serialize;

use crate::clients::base::{
    delete_resource, get_resource, post_action, post_with_body, update_resource, wait_for_finish,
    ResourceContext,
};
use crate::clients::dataset::DatasetClient;
use crate::clients::key_value_store::KeyValueStoreClient;
use crate::clients::log::LogClient;
use crate::clients::request_queue::RequestQueueClient;
use crate::common::{to_safe_id, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorRun;

/// Options for resurrecting a finished run.
#[derive(Debug, Default, Clone)]
pub struct RunResurrectOptions {
    /// Build tag/number to use; defaults to the original run's build.
    pub build: Option<String>,
    /// Memory in megabytes.
    pub memory_mbytes: Option<i64>,
    /// Timeout in seconds.
    pub timeout_secs: Option<i64>,
}

/// Client for a specific Actor run.
///
/// In addition to CRUD-style operations, this client exposes the run's lifecycle
/// actions (abort, metamorph, reboot, resurrect, charge) and provides access to the
/// run's default dataset, key-value store, request queue and log.
#[derive(Debug, Clone)]
pub struct RunClient {
    ctx: ResourceContext,
}

impl RunClient {
    pub(crate) fn new(
        _root: crate::client::ApifyClient,
        http: HttpClient,
        base_url: &str,
        resource_path: &str,
        id: &str,
    ) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, resource_path, id),
        }
    }

    /// Adds a `status` filter to this run client (used by `actor.last_run`).
    pub(crate) fn set_status_param(&mut self, status: &str) {
        self.ctx
            .base_params
            .push_raw("status".to_string(), status.to_string());
    }

    /// Fetches the run object, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<ActorRun>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the run (e.g. its status message) and returns the updated object.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<ActorRun> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the run.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Aborts the run. If `gracefully` is `true`, the run can perform cleanup first.
    pub async fn abort(&self, gracefully: bool) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        params.add_bool("gracefully", Some(gracefully));
        post_action(&self.ctx, Some("abort"), &params, None, None).await
    }

    /// Transforms the run into a run of another Actor (metamorph).
    pub async fn metamorph<T: Serialize>(
        &self,
        target_actor_id: &str,
        input: Option<&T>,
        build: Option<&str>,
    ) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        params
            .add_str("targetActorId", Some(to_safe_id(target_actor_id)))
            .add_str("build", build.map(|s| s.to_string()));
        let body = match input {
            Some(value) => Some(serde_json::to_vec(value)?),
            None => None,
        };
        post_with_body(
            &self.ctx,
            Some("metamorph"),
            &params,
            body,
            "application/json",
        )
        .await
    }

    /// Reboots the run (restarts its container, preserving the run ID and storages).
    pub async fn reboot(&self) -> ApifyClientResult<ActorRun> {
        post_action(&self.ctx, Some("reboot"), &QueryParams::new(), None, None).await
    }

    /// Resurrects a finished run, starting it again with (optionally overridden) settings.
    pub async fn resurrect(&self, options: RunResurrectOptions) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        params
            .add_str("build", options.build)
            .add_int("memory", options.memory_mbytes)
            .add_int("timeout", options.timeout_secs);
        post_action(&self.ctx, Some("resurrect"), &params, None, None).await
    }

    /// Charges the run for a pay-per-event `event_name`, `count` times.
    ///
    /// The charge endpoint returns an empty body on success, so this issues the request
    /// directly and treats any 2xx response as success (errors still surface normally).
    pub async fn charge(&self, event_name: &str, count: i64) -> ApifyClientResult<()> {
        let body = serde_json::json!({ "eventName": event_name, "count": count });
        let body_bytes = serde_json::to_vec(&body)?;
        let url = self.ctx.url(Some("charge"));
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        // A successful `HttpClient::call` already guarantees a 2xx status; the (empty) body
        // is intentionally ignored rather than parsed as a `data` envelope.
        self.ctx
            .http
            .call(crate::http_client::HttpRequest {
                method: crate::http_client::HttpMethod::Post,
                url,
                headers,
                body: Some(body_bytes),
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        Ok(())
    }

    /// Waits (by client-side polling) for the run to reach a terminal state.
    ///
    /// `wait_secs` bounds the wait; `None` waits indefinitely.
    pub async fn wait_for_finish(&self, wait_secs: Option<i64>) -> ApifyClientResult<ActorRun> {
        wait_for_finish(&self.ctx, wait_secs, |r: &ActorRun| r.is_terminal()).await
    }

    /// Returns a client for the run's default dataset.
    pub fn dataset(&self) -> DatasetClient {
        DatasetClient::nested(self.ctx.http.clone(), &self.ctx.url(None), "dataset")
    }

    /// Returns a client for the run's default key-value store.
    pub fn key_value_store(&self) -> KeyValueStoreClient {
        KeyValueStoreClient::nested(
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "key-value-store",
        )
    }

    /// Returns a client for the run's default request queue.
    pub fn request_queue(&self) -> RequestQueueClient {
        RequestQueueClient::nested(self.ctx.http.clone(), &self.ctx.url(None), "request-queue")
    }

    /// Returns a client for the run's log.
    pub fn log(&self) -> LogClient {
        LogClient::nested(self.ctx.http.clone(), &self.ctx.url(None), "log")
    }

    /// Opens a live stream of the run's log for redirection.
    ///
    /// Convenience equivalent to `run.log().stream()` (mirrors the reference client's
    /// `getStreamedLog`): yields log chunks as they arrive, so callers can forward them to
    /// their own logger/stdout while the run is in progress.
    pub async fn get_streamed_log(
        &self,
    ) -> ApifyClientResult<impl futures_util::Stream<Item = ApifyClientResult<Vec<u8>>>> {
        self.log().stream().await
    }
}
