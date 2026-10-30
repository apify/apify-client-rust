//! Client for a single Actor task (`/v2/actor-tasks/{actorTaskId}`).

use serde::Serialize;
use serde_json::Value;

use crate::client::ApifyClient;
use crate::clients::actor::{ActorStartOptions, TaskStartOptions};
use crate::clients::base::{
    delete_resource, get_resource, post_creating, update_resource, ResourceContext,
};
use crate::clients::run::{LastRunOptions, RunClient};
use crate::clients::run_collection::RunCollectionClient;
use crate::clients::webhook_collection::WebhookCollectionClient;
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::{ActorRun, Task};
use crate::timeouts::Timeout;

/// Client for a specific Actor task.
#[derive(Debug, Clone)]
pub struct TaskClient {
    root: ApifyClient,
    ctx: ResourceContext,
}

crate::clients::base::impl_with_timeout!(TaskClient);

impl TaskClient {
    pub(crate) fn new(root: ApifyClient, http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            root,
            ctx: ResourceContext::single(http, base_url, "actor-tasks", id),
        }
    }

    /// Fetches the task object, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Task>> {
        get_resource(&self.ctx, None, &QueryParams::new(), Timeout::Short).await
    }

    /// Updates the task with the given fields.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<Task> {
        update_resource(&self.ctx, None, new_fields, Timeout::Short).await
    }

    /// Deletes the task.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None, Timeout::Short).await
    }

    /// Publishes the task on its public landing page, by setting `isPublic: true` through
    /// [`TaskClient::update`].
    ///
    /// The task's Actor must be public and the task must already have its public display
    /// configuration ([`Task::public_config`]) set up. An Actor can have at most 10 published
    /// tasks, and an account at most 100 across all its Actors; contact Apify support to raise
    /// either limit. Publishing an already-published task does nothing. The returned
    /// [`Task::is_public`] reflects the new publication state.
    pub async fn publish(&self) -> ApifyClientResult<Task> {
        self.update(&serde_json::json!({ "isPublic": true })).await
    }

    /// Unpublishes the task from its public landing page, by setting `isPublic: false` through
    /// [`TaskClient::update`].
    ///
    /// The public display configuration ([`Task::public_config`]) is preserved, so the task can
    /// be published again later without re-entering it. Unpublishing a task that is not
    /// published does nothing.
    pub async fn unpublish(&self) -> ApifyClientResult<Task> {
        self.update(&serde_json::json!({ "isPublic": false })).await
    }

    /// Starts the task and returns immediately with the created run.
    ///
    /// `input` overrides the task's saved input (or `None` to use the saved input). Set
    /// [`TaskStartOptions::wait_for_resources`] to retry the start while the account lacks the
    /// memory or a concurrent-run slot for it.
    pub async fn start<T: Serialize>(
        &self,
        input: Option<&T>,
        options: TaskStartOptions,
    ) -> ApifyClientResult<ActorRun> {
        self.start_with_tier(input, options, Timeout::Medium).await
    }

    /// Starts the task with `tier` as the timeout of the request (unless the client was given
    /// one with `with_timeout`).
    async fn start_with_tier<T: Serialize>(
        &self,
        input: Option<&T>,
        options: TaskStartOptions,
        tier: Timeout,
    ) -> ApifyClientResult<ActorRun> {
        let options = ActorStartOptions::from(options);
        let mut params = QueryParams::new();
        options.apply(&mut params);
        let body = match input {
            Some(value) => Some(serde_json::to_vec(value)?),
            None => None,
        };
        let timeout = self
            .ctx
            .timeout_with_wait_hold(tier, options.wait_for_finish);
        crate::clients::base::retry_while_resource_limited(options.wait_for_resources, || {
            post_creating(
                &self.ctx,
                Some("runs"),
                &params,
                body.clone(),
                "application/json",
                timeout,
            )
        })
        .await
    }

    /// Starts the task and waits (client-side polling) for it to finish.
    ///
    /// `wait_secs` controls the wait budget:
    /// - `None` polls indefinitely until the run reaches a terminal state.
    /// - `Some(n)` bounds the wait to roughly `n` seconds; if the run has not finished by
    ///   then, the **last fetched (still non-terminal) run is returned** rather than an
    ///   error. Check `status` / `is_terminal()` on the result when using `Some`.
    ///
    /// Time spent retrying under [`TaskStartOptions::wait_for_resources`] does not count
    /// toward `wait_secs`, since that budget only starts once the run has actually started.
    ///
    /// Like [`ActorClient::call`](crate::clients::actor::ActorClient::call), the start request
    /// has no client-side timeout unless the client was given one with `with_timeout`, which
    /// then applies to the polls as well. The log of the run is not redirected.
    pub async fn call<T: Serialize>(
        &self,
        input: Option<&T>,
        options: TaskStartOptions,
        wait_secs: Option<i64>,
    ) -> ApifyClientResult<ActorRun> {
        let run = self
            .start_with_tier(input, options, Timeout::NoTimeout)
            .await?;
        let mut run_client = self.root.run(run.id);
        if let Some(timeout) = self.ctx.timeout {
            run_client = run_client.with_timeout(timeout);
        }
        run_client.wait_for_finish(wait_secs).await
    }

    /// Fetches the task's saved input.
    ///
    /// A task that does not exist is a [`NotFound`](crate::ApiError::is_not_found) error: the
    /// input is a fixed sub-path of the task, so a `404` cannot be told apart from a missing
    /// input.
    pub async fn get_input(&self) -> ApifyClientResult<Value> {
        let response = crate::clients::base::get_raw_required(
            &self.ctx,
            Some("input"),
            &QueryParams::new(),
            Timeout::Short,
        )
        .await?;
        Ok(serde_json::from_slice(&response.body)?)
    }

    /// Updates the task's saved input.
    pub async fn update_input<T: Serialize>(&self, input: &T) -> ApifyClientResult<Value> {
        let body = serde_json::to_vec(input)?;
        let url = self.ctx.url(Some("input"));
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let response = self
            .ctx
            .send_with(
                crate::http_client::HttpMethod::Put,
                url,
                headers,
                Some(body),
                self.ctx.timeout(Timeout::Short),
            )
            .await?;
        Ok(serde_json::from_slice(&response.body)?)
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
        let mut client = RunClient::new(
            self.root.clone(),
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "runs",
            "last",
        );
        if let Some(status) = options.status.as_deref() {
            client.set_base_param("status", status);
        }
        if let Some(origin) = options.origin.as_deref() {
            client.set_base_param("origin", origin);
        }
        client
    }

    /// Returns a client for this task's run collection.
    pub fn runs(&self) -> RunCollectionClient {
        RunCollectionClient::new(self.ctx.http.clone(), &self.ctx.url(None), "runs")
    }

    /// Returns a client for this task's webhook collection.
    pub fn webhooks(&self) -> WebhookCollectionClient {
        WebhookCollectionClient::with_base(self.ctx.http.clone(), &self.ctx.url(None))
    }
}
