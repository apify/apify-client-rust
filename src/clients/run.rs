//! Client for a single Actor run (`/v2/actor-runs/{runId}` and nested routes).

use serde::Serialize;

use crate::clients::base::{
    delete_resource, get_resource, post_action, post_with_body, update_resource, wait_for_finish,
    ResourceContext,
};
use crate::clients::dataset::{DatasetClient, DatasetListItemsOptions};
use crate::clients::key_value_store::KeyValueStoreClient;
use crate::clients::log::LogClient;
use crate::clients::request_queue::RequestQueueClient;
use crate::clients::run_dataset_items::RunDatasetItemsIterator;
use crate::clients::streamed_log::{
    LogRedirect, LogSink, StdoutLogSink, StreamedLog, StreamedLogOptions,
};
use crate::common::{to_safe_id, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorRun;
use crate::timeouts::Timeout;
use std::sync::Arc;

/// Header the API uses to deduplicate charge requests (matching the reference client).
const CHARGE_IDEMPOTENCY_HEADER: &str = "idempotency-key";
/// Nanoseconds in a millisecond: the sub-millisecond part of a timestamp is below it.
const NANOS_PER_MILLI: u32 = 1_000_000;

/// Options for fetching an Actor's or task's last run
/// ([`crate::clients::actor::ActorClient::last_run_with_options`] /
/// [`crate::clients::task::TaskClient::last_run_with_options`]).
///
/// Both are sent as query parameters to the last-run endpoints
/// (`GET /v2/actors/{actorId}/runs/last`, `GET /v2/actor-tasks/{actorTaskId}/runs/last`).
/// `status` and `origin` are both documented optional filters on those endpoints and match the
/// reference client's `lastRun({ status, origin })`.
#[derive(Debug, Default, Clone)]
pub struct LastRunOptions {
    /// Filter by run status (e.g. `"SUCCEEDED"`, `"FAILED"`, `"RUNNING"`). `None` leaves it
    /// unfiltered.
    pub status: Option<String>,
    /// Filter by how the run was started; accepted values are the platform's run origins
    /// (e.g. `"DEVELOPMENT"`, `"WEB"`, `"API"`, `"SCHEDULER"`). `None` leaves it unfiltered.
    pub origin: Option<String>,
}

/// Options for resurrecting a finished run.
#[derive(Debug, Default, Clone)]
pub struct RunResurrectOptions {
    /// Build tag/number to use; defaults to the original run's build.
    pub build: Option<String>,
    /// Memory in megabytes.
    pub memory_mbytes: Option<i64>,
    /// Timeout of the run itself in seconds. Not the timeout of the API request, which is set with
    /// the client's `with_timeout`.
    pub run_timeout_secs: Option<i64>,
    /// Maximum number of dataset items to charge (pay-per-result Actors).
    pub max_items: Option<i64>,
    /// Maximum total cost of the run in USD, across all pricing models.
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
    /// Idempotency key deduplicating the charge across retries. The API requires this header
    /// on every charge request and forgets each key 3 minutes after the charge, so a retry sent
    /// past that window with the same key creates a new charge rather than being deduplicated.
    /// If `None`, one is auto-generated as `{runId}-{eventName}-{timestampMillis}-{random}`,
    /// matching the reference client, so a transport-retried charge is applied at most once.
    pub idempotency_key: Option<String>,
    /// ID of the Server Actor request being charged, taken from the `X-Actor-Request-ID` header
    /// of the incoming request. Required for multi-tenant Server Actor runs, where it maps the
    /// charge to the caller's account instead of the account that started the run. Omitted from
    /// the request body when `None`.
    pub request_id: Option<String>,
}

/// Client for a specific Actor run.
///
/// In addition to CRUD-style operations, this client exposes the run's lifecycle
/// actions (abort, metamorph, reboot, resurrect, charge) and provides access to the
/// run's default dataset, key-value store, request queue and log.
#[derive(Debug, Clone)]
pub struct RunClient {
    root: crate::client::ApifyClient,
    ctx: ResourceContext,
    /// The run ID, retained so `charge` can build a per-run idempotency key.
    id: String,
}

crate::clients::base::impl_with_timeout!(RunClient);

impl RunClient {
    pub(crate) fn new(
        root: crate::client::ApifyClient,
        http: HttpClient,
        base_url: &str,
        resource_path: &str,
        id: &str,
    ) -> Self {
        Self {
            root,
            ctx: ResourceContext::single(http, base_url, resource_path, id),
            id: id.to_string(),
        }
    }

    /// The run ID this client addresses; `last` for the client of a `last_run()`.
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    /// The timeout set with `with_timeout`, if any.
    pub(crate) fn timeout(&self) -> Option<Timeout> {
        self.ctx.timeout
    }

    /// A client for the run with the given ID.
    pub(crate) fn pinned_to(&self, run_id: &str) -> RunClient {
        self.root.run(run_id)
    }

    /// Adds a base query parameter to this run client, inherited by its requests (used by
    /// `actor.last_run` / `task.last_run` to thread the `status` and `origin` filters).
    pub(crate) fn set_base_param(&mut self, key: &str, value: &str) {
        self.ctx
            .base_params
            .push_raw(key.to_string(), value.to_string());
    }

    /// Fetches the run object, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<ActorRun>> {
        get_resource(&self.ctx, None, &QueryParams::new(), Timeout::Short).await
    }

    /// Updates the run (e.g. its status message) and returns the updated object.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<ActorRun> {
        update_resource(&self.ctx, None, new_fields, Timeout::Short).await
    }

    /// Deletes the run.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None, Timeout::Short).await
    }

    /// Aborts the run. `gracefully` is optional, matching the reference client's optional
    /// `gracefully` option and the Go sibling's `Option<bool>`: `Some(true)` lets the run
    /// perform cleanup first, `Some(false)` aborts immediately, and `None` omits the parameter
    /// entirely so the server applies its default (immediate abort).
    pub async fn abort(&self, gracefully: Option<bool>) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        params.add_bool("gracefully", gracefully);
        post_action(
            &self.ctx,
            Some("abort"),
            &params,
            None,
            None,
            self.ctx.timeout(Timeout::Medium),
        )
        .await
    }

    /// Transforms the run into a run of another Actor (metamorph).
    ///
    /// `options.content_type` sets the content type of the input body (defaulting to
    /// `application/json`), matching the reference client's `metamorph(..., { contentType })`.
    /// To send a non-JSON input as raw bytes instead, use [`metamorph_raw`](Self::metamorph_raw).
    pub async fn metamorph<T: Serialize>(
        &self,
        target_actor_id: &str,
        input: Option<&T>,
        options: RunMetamorphOptions,
    ) -> ApifyClientResult<ActorRun> {
        let body = match input {
            Some(value) => Some(serde_json::to_vec(value)?),
            None => None,
        };
        self.metamorph_with_body(target_actor_id, body, "application/json", options)
            .await
    }

    /// Transforms the run into a run of another Actor (metamorph) with a raw request body
    /// instead of a JSON-serializable value.
    ///
    /// The bytes are sent exactly as given, with no JSON serialization — pair this with
    /// `options.content_type` for a non-JSON input. For an object or array input, use
    /// [`metamorph`](Self::metamorph) instead.
    pub async fn metamorph_raw(
        &self,
        target_actor_id: &str,
        input: &[u8],
        options: RunMetamorphOptions,
    ) -> ApifyClientResult<ActorRun> {
        self.metamorph_with_body(
            target_actor_id,
            Some(input.to_vec()),
            "application/octet-stream",
            options,
        )
        .await
    }

    /// Shared implementation of [`metamorph`](Self::metamorph) and
    /// [`metamorph_raw`](Self::metamorph_raw).
    async fn metamorph_with_body(
        &self,
        target_actor_id: &str,
        body: Option<Vec<u8>>,
        default_content_type: &str,
        options: RunMetamorphOptions,
    ) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        params
            .add_str("targetActorId", Some(to_safe_id(target_actor_id)))
            .add_str("build", options.build);
        let content_type = options
            .content_type
            .as_deref()
            .unwrap_or(default_content_type)
            .to_string();
        post_with_body(
            &self.ctx,
            Some("metamorph"),
            &params,
            body,
            &content_type,
            self.ctx.timeout(Timeout::Medium),
        )
        .await
    }

    /// Reboots the run (restarts its container, preserving the run ID and storages).
    pub async fn reboot(&self) -> ApifyClientResult<ActorRun> {
        post_action(
            &self.ctx,
            Some("reboot"),
            &QueryParams::new(),
            None,
            None,
            self.ctx.timeout(Timeout::Medium),
        )
        .await
    }

    /// Resurrects a finished run, starting it again with (optionally overridden) settings.
    ///
    /// Only runs in a terminal state (`SUCCEEDED`, `FAILED`, `ABORTED` or `TIMED-OUT`) can be
    /// resurrected. The run continues with the same storages, and its timeout counts from the
    /// resurrection. `max_total_charge_usd` applies to the whole run including the cost before the
    /// resurrection, so raise it to continue a run that stopped at its cost limit; otherwise the
    /// resurrected run keeps the original limit and stops right after it starts.
    pub async fn resurrect(&self, options: RunResurrectOptions) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        params
            .add_str("build", options.build)
            .add_int("memory", options.memory_mbytes)
            .add_int("timeout", options.run_timeout_secs)
            .add_int("maxItems", options.max_items)
            .add_float("maxTotalChargeUsd", options.max_total_charge_usd)
            .add_bool("restartOnError", options.restart_on_error);
        post_action(
            &self.ctx,
            Some("resurrect"),
            &params,
            None,
            None,
            self.ctx.timeout(Timeout::Medium),
        )
        .await
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
        let mut body = serde_json::json!({ "eventName": options.event_name, "count": count });
        if let Some(request_id) = options.request_id {
            body["requestId"] = serde_json::Value::String(request_id);
        }
        let body_bytes = serde_json::to_vec(&body)?;
        let url = self
            .ctx
            .url_with_params(Some("charge"), &QueryParams::new());
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert(CHARGE_IDEMPOTENCY_HEADER.to_string(), idempotency_key);
        // A successful `HttpClient::call` already guarantees a 2xx status; the (empty) body
        // is intentionally ignored rather than parsed as a `data` envelope.
        self.ctx
            .send_with(
                crate::http_client::HttpMethod::Post,
                url,
                headers,
                Some(body_bytes),
                self.ctx.timeout(Timeout::Short),
            )
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
        let random_suffix = now.subsec_nanos() % NANOS_PER_MILLI;
        format!("{}-{event_name}-{millis}-{random_suffix}", self.id)
    }

    /// Waits (by client-side polling) for the run to reach a terminal state.
    ///
    /// `wait_secs` controls the wait budget:
    /// - `None` polls indefinitely until the run reaches a terminal state.
    /// - `Some(n)` bounds the wait to roughly `n` seconds; if the run has not finished by
    ///   then, the **last fetched (still non-terminal) run is returned** rather than an
    ///   error. Check `status` / `is_terminal()` on the result when using `Some`.
    ///
    /// With `None`, the polls have no client-side timeout. With `Some(n)`, each poll is allowed the
    /// `Medium` tier plus the hold (up to a minute) it asks the API for, so a stalled connection
    /// cannot hang the wait. A timeout set with `with_timeout` is used as given instead.
    pub async fn wait_for_finish(&self, wait_secs: Option<i64>) -> ApifyClientResult<ActorRun> {
        wait_for_finish(&self.ctx, wait_secs, |r: &ActorRun| r.is_terminal()).await
    }

    /// Returns a client for the run's default dataset.
    pub fn dataset(&self) -> DatasetClient {
        DatasetClient::nested(
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "dataset",
            &self.ctx.base_params,
        )
    }

    /// Returns a client for the run's default key-value store.
    pub fn key_value_store(&self) -> KeyValueStoreClient {
        KeyValueStoreClient::nested(
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "key-value-store",
            &self.ctx.base_params,
        )
    }

    /// Returns a client for the run's default request queue.
    pub fn request_queue(&self) -> RequestQueueClient {
        RequestQueueClient::nested(
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "request-queue",
            &self.ctx.base_params,
        )
    }

    /// Returns a client for the run's log.
    pub fn log(&self) -> LogClient {
        LogClient::nested(
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "log",
            &self.ctx.base_params,
        )
    }

    /// Iterates over the items of the run's default dataset while the run is still producing them.
    ///
    /// Returns a [`RunDatasetItemsIterator`]; call its `next()` to get one item at a time. Unlike
    /// [`DatasetClient::iterate_items`], which stops at the end of the dataset as it is, this keeps
    /// polling until the run finishes, so a consumer can process the items of a long-running Actor
    /// as they appear. See [`RunDatasetItemsIterator`] for how the rows are read, and its
    /// `with_chunk_size` and `with_poll_interval` for the page size and the polling interval.
    ///
    /// `options` filters and projects the items exactly like
    /// [`DatasetClient::list_items`]: `offset` is the number of dataset rows to skip and `limit`
    /// the maximum number of rows to scan. `desc: Some(true)` is rejected with
    /// [`ApifyClientError::InvalidArgument`](crate::error::ApifyClientError::InvalidArgument), since
    /// the end of a dataset that is still growing is not known: the rows are always read oldest
    /// first.
    ///
    /// # Example
    /// ```no_run
    /// use apify_client::ApifyClient;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ApifyClient::new("my-api-token");
    /// let mut items = client
    ///     .run("my-run-id")
    ///     .iterate_dataset_items::<serde_json::Value>(Default::default());
    /// while let Some(item) = items.next().await? {
    ///     println!("{item}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn iterate_dataset_items<T: serde::de::DeserializeOwned>(
        &self,
        options: DatasetListItemsOptions,
    ) -> RunDatasetItemsIterator<T> {
        RunDatasetItemsIterator::new(self.clone(), options)
    }

    /// Returns the redirection of the run's log, or `None` if `options.to_log` is
    /// [`LogRedirect::Disabled`] or, for [`LogRedirect::Default`], the run does not exist. With a
    /// [`LogRedirect::Sink`] nothing is requested, so a missing run shows up as the error of
    /// [`StreamedLog::stop`]. Call [`StreamedLog::start`] to begin
    /// redirecting and [`StreamedLog::stop`] to end it. Mirrors the reference client's
    /// `getStreamedLog`.
    ///
    /// With [`LogRedirect::Default`], the messages go to the standard output prefixed with
    /// `"{actor name} runId:{run id} -> "`, which costs a request for the run and one for its
    /// Actor. With [`LogRedirect::Sink`], they go to the sink as they are, and nothing is
    /// requested. To read the log as raw chunks instead, use
    /// [`LogClient::stream_with_options`](crate::clients::log::LogClient::stream_with_options) of
    /// [`RunClient::log`].
    pub async fn get_streamed_log(
        &self,
        options: StreamedLogOptions,
    ) -> ApifyClientResult<Option<StreamedLog>> {
        let StreamedLogOptions { to_log, from_start } = options;
        let (sink, prefix): (Arc<dyn LogSink>, String) = match to_log {
            LogRedirect::Disabled => return Ok(None),
            LogRedirect::Sink(sink) => (sink, String::new()),
            LogRedirect::Default => {
                let Some(run) =
                    get_resource::<ActorRun>(&self.ctx, None, &QueryParams::new(), Timeout::Long)
                        .await?
                else {
                    return Ok(None);
                };
                let actor_name = match run.act_id.as_deref().filter(|id| !id.is_empty()) {
                    Some(actor_id) => self
                        .root
                        .actor(actor_id)
                        .with_timeout(Timeout::Long)
                        .get()
                        .await?
                        .and_then(|actor| actor.name),
                    None => None,
                };
                let name = [actor_name, Some(format!("runId:{}", run.id))]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ");
                (Arc::new(StdoutLogSink), format!("{name} -> "))
            }
        };
        Ok(Some(StreamedLog::new(self.log(), sink, prefix, from_start)))
    }
}
