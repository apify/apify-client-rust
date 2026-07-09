//! Client for a single Actor task (`/v2/actor-tasks/{actorTaskId}`).

use serde::Serialize;
use serde_json::Value;

use crate::client::ApifyClient;
use crate::clients::actor::ActorStartOptions;
use crate::clients::base::{
    delete_resource, get_resource, post_with_body, update_resource, ResourceContext,
};
use crate::clients::run::{LastRunOptions, RunClient};
use crate::clients::run_collection::RunCollectionClient;
use crate::clients::webhook_collection::WebhookCollectionClient;
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::{ActorRun, RunStatus, Task};

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
    /// `input` overrides the task's saved input (or `None` to use the saved input).
    pub async fn start<T: Serialize>(
        &self,
        input: Option<&T>,
        options: ActorStartOptions,
    ) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        options.apply(&mut params);
        let body = match input {
            Some(value) => Some(serde_json::to_vec(value)?),
            None => None,
        };
        post_with_body(&self.ctx, Some("runs"), &params, body, "application/json").await
    }

    /// Starts the task and waits (client-side polling) for it to finish.
    ///
    /// `wait_secs` controls the wait budget:
    /// - `None` polls indefinitely until the run reaches a terminal state.
    /// - `Some(n)` bounds the wait to roughly `n` seconds; if the run has not finished by
    ///   then, the **last fetched (still non-terminal) run is returned** rather than an
    ///   error. Check `status` / `is_terminal()` on the result when using `Some`.
    pub async fn call<T: Serialize>(
        &self,
        input: Option<&T>,
        options: ActorStartOptions,
        wait_secs: Option<i64>,
    ) -> ApifyClientResult<ActorRun> {
        let run = self.start(input, options).await?;
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
        let url = self.ctx.url(Some("input"));
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let response = self
            .ctx
            .http
            .call(crate::http_client::HttpRequest {
                method: crate::http_client::HttpMethod::Put,
                url,
                headers,
                body: Some(body),
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        Ok(serde_json::from_slice(&response.body)?)
    }

    /// Returns a client for the last run of this task, optionally filtered by run status.
    ///
    /// `status` filters by run status (e.g. [`RunStatus::Succeeded`]); pass `None` to leave it
    /// unfiltered. This maps to the `status` query parameter on
    /// `GET /v2/actor-tasks/{actorTaskId}/runs/last` and mirrors the reference client's
    /// `lastRun({ status })`. To also filter by `origin`, use [`TaskClient::last_run_with_options`].
    pub fn last_run(&self, status: Option<RunStatus>) -> RunClient {
        self.last_run_with_options(LastRunOptions {
            status,
            origin: None,
        })
    }

    /// Returns a client for the last run of this task, applying the given [`LastRunOptions`]
    /// (e.g. [`LastRunOptions::status`] and/or [`LastRunOptions::origin`]).
    ///
    /// `status` filters by run status (e.g. [`RunStatus::Succeeded`]); `origin` filters by how the
    /// run was started (e.g. [`crate::models::RunOrigin::Api`]). Both are documented optional query
    /// parameters on `GET /v2/actor-tasks/{actorTaskId}/runs/last` and match the reference client's
    /// `lastRun({ status, origin })`; leave a field as `None` to omit it.
    pub fn last_run_with_options(&self, options: LastRunOptions) -> RunClient {
        let mut client = RunClient::new(self.ctx.http.clone(), &self.ctx.url(None), "runs", "last");
        if let Some(status) = &options.status {
            client.set_base_param("status", status.as_str());
        }
        if let Some(origin) = &options.origin {
            client.set_base_param("origin", origin.as_str());
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
