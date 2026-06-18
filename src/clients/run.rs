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

/// Header the API uses to deduplicate charge requests (matching the reference client).
const CHARGE_IDEMPOTENCY_HEADER: &str = "idempotency-key";

/// Options for resurrecting a finished run.
#[derive(Debug, Default, Clone)]
pub struct RunResurrectOptions {
    /// Build tag/number to use; defaults to the original run's build.
    pub build: Option<String>,
    /// Memory in megabytes.
    pub memory_mbytes: Option<i64>,
    /// Timeout in seconds.
    pub timeout_secs: Option<i64>,
    /// Maximum number of dataset items to charge (pay-per-result Actors).
    pub max_items: Option<i64>,
    /// Maximum total charge in USD (pay-per-event Actors).
    pub max_total_charge_usd: Option<f64>,
    /// If `true`, restart the run automatically when it fails.
    pub restart_on_error: Option<bool>,
}

/// Options for transforming a run into another Actor's run (metamorph).
#[derive(Debug, Default, Clone)]
pub struct RunMetamorphOptions {
    /// Build tag/number of the target Actor to use (defaults to the target's default build).
    pub build: Option<String>,
    /// Content type of the input body. Defaults to `application/json` when unset.
    pub content_type: Option<String>,
}

/// Options for charging a pay-per-event run via [`RunClient::charge`].
#[derive(Debug, Default, Clone)]
pub struct RunChargeOptions {
    /// Name of the event to charge for. Required.
    pub event_name: String,
    /// Number of times to charge the event (defaults to `1`).
    pub count: Option<i64>,
    /// Idempotency key deduplicating the charge across retries. If `None`, one is
    /// auto-generated as `{runId}-{eventName}-{timestampMillis}-{random}`, matching the
    /// reference client, so a transport-retried charge is applied at most once.
    pub idempotency_key: Option<String>,
}

/// Client for a specific Actor run.
///
/// In addition to CRUD-style operations, this client exposes the run's lifecycle
/// actions (abort, metamorph, reboot, resurrect, charge) and provides access to the
/// run's default dataset, key-value store, request queue and log.
#[derive(Debug, Clone)]
pub struct RunClient {
    ctx: ResourceContext,
    /// The run ID, retained so `charge` can build a per-run idempotency key.
    id: String,
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
            id: id.to_string(),
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

    /// Aborts the run. `gracefully` is optional, matching the reference client's optional
    /// `gracefully` option and the Go sibling's `Option<bool>`: `Some(true)` lets the run
    /// perform cleanup first, `Some(false)` aborts immediately, and `None` omits the parameter
    /// entirely so the server applies its default (immediate abort).
    pub async fn abort(&self, gracefully: Option<bool>) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        params.add_bool("gracefully", gracefully);
        post_action(&self.ctx, Some("abort"), &params, None, None).await
    }

    /// Transforms the run into a run of another Actor (metamorph).
    ///
    /// `options.content_type` sets the content type of the input body (defaulting to
    /// `application/json`), matching the reference client's `metamorph(..., { contentType })`.
    pub async fn metamorph<T: Serialize>(
        &self,
        target_actor_id: &str,
        input: Option<&T>,
        options: RunMetamorphOptions,
    ) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        params
            .add_str("targetActorId", Some(to_safe_id(target_actor_id)))
            .add_str("build", options.build);
        let body = match input {
            Some(value) => Some(serde_json::to_vec(value)?),
            None => None,
        };
        let content_type = options
            .content_type
            .as_deref()
            .unwrap_or("application/json");
        post_with_body(&self.ctx, Some("metamorph"), &params, body, content_type).await
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
            .add_int("timeout", options.timeout_secs)
            .add_int("maxItems", options.max_items)
            .add_float("maxTotalChargeUsd", options.max_total_charge_usd)
            .add_bool("restartOnError", options.restart_on_error);
        post_action(&self.ctx, Some("resurrect"), &params, None, None).await
    }

    /// Charges the run for a pay-per-event run, recording occurrences of a named event.
    ///
    /// An idempotency key is always sent (auto-generated when `options.idempotency_key` is
    /// `None`), so a charge that is retried by the transport is applied at most once — matching
    /// the reference client and preventing double-charging.
    ///
    /// The charge endpoint returns an empty body on success, so this issues the request
    /// directly and treats any 2xx response as success (errors still surface normally).
    pub async fn charge(&self, options: RunChargeOptions) -> ApifyClientResult<()> {
        let count = options.count.unwrap_or(1);
        let idempotency_key = options
            .idempotency_key
            .unwrap_or_else(|| self.generate_idempotency_key(&options.event_name));
        let body = serde_json::json!({ "eventName": options.event_name, "count": count });
        let body_bytes = serde_json::to_vec(&body)?;
        let url = self.ctx.url(Some("charge"));
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert(CHARGE_IDEMPOTENCY_HEADER.to_string(), idempotency_key);
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

    /// Builds a per-charge idempotency key of the form
    /// `{runId}-{eventName}-{timestampMillis}-{random}`, matching the reference client. The
    /// suffix only needs to be unique enough to avoid collisions within the same millisecond;
    /// it is derived from the sub-millisecond part of the current time (no crypto needed).
    fn generate_idempotency_key(&self, event_name: &str) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let millis = now.as_millis();
        // Sub-millisecond nanos give a cheap, non-crypto "random" suffix (cf. JS Math.random()).
        let random_suffix = now.subsec_nanos() % 1_000_000;
        format!("{}-{event_name}-{millis}-{random_suffix}", self.id)
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
