//! Client for a single Actor task (`/v2/actor-tasks/{actorTaskId}`).

use serde::Serialize;
use serde_json::Value;

use crate::client::ApifyClient;
use crate::clients::actor::encode_webhooks;
use crate::clients::base::{
    delete_resource, get_resource, post_with_body, put_action_raw, update_resource, ResourceContext,
};
use crate::clients::run::{LastRunOptions, RunClient};
use crate::clients::run_collection::RunCollectionClient;
use crate::clients::webhook_collection::WebhookCollectionClient;
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, CONTENT_TYPE_JSON};
use crate::models::{ActorRun, Task};

/// Options for [`TaskClient::start`].
///
/// Like [`ActorStartOptions`](crate::clients::actor::ActorStartOptions), but without
/// `content_type` (a task's input content type is fixed as `application/json` — a task's input
/// is predefined, unlike an ad-hoc Actor start) or `force_permission_level` (not accepted by the
/// task-run endpoint). This mirrors the JS reference client's `TaskStartOptions`, typed as
/// `Omit<ActorStartOptions, 'contentType' | 'forcePermissionLevel'>`.
#[derive(Debug, Default, Clone)]
pub struct TaskStartOptions {
    /// Tag or number of the build to run (e.g. `latest`, `0.1.2`).
    pub build: Option<String>,
    /// Memory in megabytes allocated for the run.
    pub memory_mbytes: Option<i64>,
    /// Timeout for the run in seconds (`0` means no timeout).
    pub timeout_secs: Option<i64>,
    /// Maximum seconds to wait server-side for the run to finish (max 60).
    pub wait_for_finish: Option<i64>,
    /// Maximum number of dataset items to charge (pay-per-result Actors).
    pub max_items: Option<i64>,
    /// Maximum total charge in USD (pay-per-event Actors).
    pub max_total_charge_usd: Option<f64>,
    /// Whether to restart the run if it fails.
    pub restart_on_error: Option<bool>,
    /// Ad-hoc webhooks to attach to this run. Encoded as base64-encoded JSON as the `webhooks`
    /// query parameter, matching the reference clients.
    pub webhooks: Option<Vec<serde_json::Value>>,
}

impl TaskStartOptions {
    /// Serializes these options into run-start query parameters.
    fn apply(&self, params: &mut QueryParams) {
        params
            .add_str("build", self.build.clone())
            .add_int("memory", self.memory_mbytes)
            .add_int("timeout", self.timeout_secs)
            .add_int("waitForFinish", self.wait_for_finish)
            .add_int("maxItems", self.max_items)
            .add_float("maxTotalChargeUsd", self.max_total_charge_usd)
            .add_bool("restartOnError", self.restart_on_error)
            .add_str("webhooks", encode_webhooks(&self.webhooks));
    }
}

/// Options for [`TaskClient::call`].
///
/// Like [`TaskStartOptions`], but without `wait_for_finish` (the server-side wait): `call`'s
/// separate `wait_secs` argument controls the client-side wait instead, so the two should not be
/// set together. This mirrors the JS reference client's `TaskCallOptions`, typed as
/// `Omit<TaskStartOptions, 'waitForFinish'>`.
#[derive(Debug, Default, Clone)]
pub struct TaskCallOptions {
    /// Tag or number of the build to run (e.g. `latest`, `0.1.2`).
    pub build: Option<String>,
    /// Memory in megabytes allocated for the run.
    pub memory_mbytes: Option<i64>,
    /// Timeout for the run in seconds (`0` means no timeout).
    pub timeout_secs: Option<i64>,
    /// Maximum number of dataset items to charge (pay-per-result Actors).
    pub max_items: Option<i64>,
    /// Maximum total charge in USD (pay-per-event Actors).
    pub max_total_charge_usd: Option<f64>,
    /// Whether to restart the run if it fails.
    pub restart_on_error: Option<bool>,
    /// Ad-hoc webhooks to attach to this run. Encoded as base64-encoded JSON as the `webhooks`
    /// query parameter, matching the reference clients.
    pub webhooks: Option<Vec<serde_json::Value>>,
}

impl From<TaskCallOptions> for TaskStartOptions {
    fn from(options: TaskCallOptions) -> Self {
        TaskStartOptions {
            build: options.build,
            memory_mbytes: options.memory_mbytes,
            timeout_secs: options.timeout_secs,
            wait_for_finish: None,
            max_items: options.max_items,
            max_total_charge_usd: options.max_total_charge_usd,
            restart_on_error: options.restart_on_error,
            webhooks: options.webhooks,
        }
    }
}

/// Client for a specific Actor task.
#[derive(Debug, Clone)]
pub struct TaskClient {
    root: ApifyClient,
    ctx: ResourceContext,
}

impl TaskClient {
    pub(crate) fn new(root: ApifyClient, http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            root,
            ctx: ResourceContext::single(http, base_url, "actor-tasks", id),
        }
    }

    /// Fetches the task object, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Task>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the task with the given fields.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<Task> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the task.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Starts the task and returns immediately with the created run.
    ///
    /// `input` overrides the task's saved input (or `None` to use the saved input). `options` is
    /// [`TaskStartOptions`] — a task's input content type and permission level are fixed, so
    /// (unlike [`ActorClient::start`](crate::clients::actor::ActorClient::start)) there is no
    /// `content_type` or `force_permission_level` field to set.
    pub async fn start<T: Serialize>(
        &self,
        input: Option<&T>,
        options: TaskStartOptions,
    ) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        options.apply(&mut params);
        let body = match input {
            Some(value) => Some(serde_json::to_vec(value)?),
            None => None,
        };
        post_with_body(&self.ctx, Some("runs"), &params, body, CONTENT_TYPE_JSON).await
    }

    /// Starts the task and waits (client-side polling) for it to finish.
    ///
    /// `wait_secs` controls the wait budget:
    /// - `None` polls indefinitely until the run reaches a terminal state.
    /// - `Some(n)` bounds the wait to roughly `n` seconds; if the run has not finished by
    ///   then, the **last fetched (still non-terminal) run is returned** rather than an
    ///   error. Check `status` / `is_terminal()` on the result when using `Some`.
    ///
    /// `options` is [`TaskCallOptions`], which (matching the JS reference client) additionally
    /// excludes `wait_for_finish` (the server-side wait) since the client-side `wait_secs`
    /// argument is how callers control call's wait behavior.
    pub async fn call<T: Serialize>(
        &self,
        input: Option<&T>,
        options: TaskCallOptions,
        wait_secs: Option<i64>,
    ) -> ApifyClientResult<ActorRun> {
        let run = self.start(input, options.into()).await?;
        self.root.run(run.id).wait_for_finish(wait_secs).await
    }

    /// Fetches the task's saved input, or `None` if not set.
    pub async fn get_input(&self) -> ApifyClientResult<Option<Value>> {
        let response =
            crate::clients::base::get_raw(&self.ctx, Some("input"), &QueryParams::new()).await?;
        match response {
            Some(r) => Ok(Some(serde_json::from_slice(&r.body)?)),
            None => Ok(None),
        }
    }

    /// Updates the task's saved input.
    pub async fn update_input<T: Serialize>(&self, input: &T) -> ApifyClientResult<Value> {
        let body = serde_json::to_vec(input)?;
        put_action_raw(
            &self.ctx,
            Some("input"),
            &QueryParams::new(),
            body,
            CONTENT_TYPE_JSON,
        )
        .await
    }

    /// Returns a client for the last run of this task, optionally filtered by run status.
    ///
    /// `status` filters by run status (e.g. `"SUCCEEDED"`, `"FAILED"`, `"RUNNING"`); pass `None`
    /// to leave it unfiltered. This maps to the `status` query parameter on
    /// `GET /v2/actor-tasks/{actorTaskId}/runs/last` and mirrors the reference client's
    /// `lastRun({ status })`. To also filter by `origin`, use [`TaskClient::last_run_with_options`].
    pub fn last_run(&self, status: Option<&str>) -> RunClient {
        self.last_run_with_options(LastRunOptions {
            status: status.map(str::to_owned),
            origin: None,
        })
    }

    /// Returns a client for the last run of this task, applying the given [`LastRunOptions`]
    /// (e.g. [`LastRunOptions::status`] and/or [`LastRunOptions::origin`]).
    ///
    /// `status` filters by run status (e.g. `"SUCCEEDED"`, `"FAILED"`, `"RUNNING"`); `origin` filters
    /// by how the run was started, with accepted values being the platform's run origins (e.g.
    /// `"DEVELOPMENT"`, `"WEB"`, `"API"`, `"SCHEDULER"`). Both are documented optional query
    /// parameters on `GET /v2/actor-tasks/{actorTaskId}/runs/last` and match the reference client's
    /// `lastRun({ status, origin })`; leave a field as `None` to omit it.
    pub fn last_run_with_options(&self, options: LastRunOptions) -> RunClient {
        crate::clients::run::last_run_client(self.ctx.http.clone(), &self.ctx.url(None), &options)
    }

    /// Returns a client for this task's run collection.
    pub fn runs(&self) -> RunCollectionClient {
        RunCollectionClient::new(self.ctx.http.clone(), &self.ctx.url(None), "runs")
    }

    /// Returns a client for this task's webhook collection.
    pub fn webhooks(&self) -> WebhookCollectionClient {
        WebhookCollectionClient::new(self.ctx.http.clone(), &self.ctx.url(None))
    }
}
