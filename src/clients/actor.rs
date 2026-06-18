//! Client for a single Actor (`/v2/actors/{actorId}`).

use serde::Serialize;
use serde_json::Value;

use crate::client::ApifyClient;
use crate::clients::actor_version::ActorVersionClient;
use crate::clients::actor_version_collection::ActorVersionCollectionClient;
use crate::clients::base::{
    delete_resource, get_resource, post_with_body, update_resource, ResourceContext,
};
use crate::clients::build::BuildClient;
use crate::clients::build_collection::BuildCollectionClient;
use crate::clients::run::RunClient;
use crate::clients::run_collection::RunCollectionClient;
use crate::clients::webhook_collection::WebhookCollectionClient;
use crate::common::{parse_data_envelope, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::{Actor, ActorRun, Build};

/// Options shared by [`ActorClient::start`] and [`ActorClient::call`] (and the task
/// equivalents).
#[derive(Debug, Default, Clone)]
pub struct ActorStartOptions {
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
    /// Content type of the input body. Defaults to `application/json`.
    pub content_type: Option<String>,
    /// Whether to restart the run if it fails.
    pub restart_on_error: Option<bool>,
    /// Override the Actor's permission level for this run.
    pub force_permission_level: Option<String>,
    /// Ad-hoc webhooks to attach to this run. Serialized to base64-encoded JSON as the
    /// `webhooks` query parameter, matching the reference clients.
    pub webhooks: Option<Vec<serde_json::Value>>,
}

impl ActorStartOptions {
    /// Serializes these options into run-start query parameters. Shared by the Actor and
    /// task start methods (DRY).
    pub(crate) fn apply(&self, params: &mut QueryParams) {
        params
            .add_str("build", self.build.clone())
            .add_int("memory", self.memory_mbytes)
            .add_int("timeout", self.timeout_secs)
            .add_int("waitForFinish", self.wait_for_finish)
            .add_int("maxItems", self.max_items)
            .add_float("maxTotalChargeUsd", self.max_total_charge_usd)
            .add_bool("restartOnError", self.restart_on_error)
            .add_str("forcePermissionLevel", self.force_permission_level.clone())
            .add_str("webhooks", self.encoded_webhooks());
    }

    /// Encodes the `webhooks` array as base64-encoded JSON, as required by the API.
    fn encoded_webhooks(&self) -> Option<String> {
        use base64::Engine;
        let webhooks = self.webhooks.as_ref()?;
        let json = serde_json::to_vec(webhooks).ok()?;
        Some(base64::engine::general_purpose::STANDARD.encode(json))
    }
}

/// Options for building an Actor.
#[derive(Debug, Default, Clone)]
pub struct ActorBuildOptions {
    /// If `true`, use beta versions of Apify packages.
    pub beta_packages: Option<bool>,
    /// Tag to apply to the build (e.g. `latest`).
    pub tag: Option<String>,
    /// Whether to use the Docker build cache (default `true`).
    pub use_cache: Option<bool>,
    /// Maximum seconds to wait server-side for the build to finish (max 60).
    pub wait_for_finish: Option<i64>,
}

/// Client for a specific Actor.
///
/// Provides CRUD methods plus convenience helpers to start/call the Actor, build it,
/// and access its runs, builds, versions and webhooks.
#[derive(Debug, Clone)]
pub struct ActorClient {
    root: ApifyClient,
    ctx: ResourceContext,
    base_url: String,
    id: String,
}

impl ActorClient {
    pub(crate) fn new(root: ApifyClient, http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            root,
            ctx: ResourceContext::single(http, base_url, "actors", id),
            base_url: base_url.to_string(),
            id: id.to_string(),
        }
    }

    /// Fetches the Actor object, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Actor>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the Actor with the given fields and returns the updated object.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<Actor> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the Actor.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Starts the Actor and returns immediately with the created run.
    ///
    /// `input` is any JSON-serializable value (or `None` for no input).
    pub async fn start<T: Serialize>(
        &self,
        input: Option<&T>,
        options: ActorStartOptions,
    ) -> ApifyClientResult<ActorRun> {
        let mut params = QueryParams::new();
        options.apply(&mut params);
        let content_type = options
            .content_type
            .clone()
            .unwrap_or_else(|| "application/json".to_string());
        let body = match input {
            Some(value) => Some(serde_json::to_vec(value)?),
            None => None,
        };
        post_with_body(&self.ctx, Some("runs"), &params, body, &content_type).await
    }

    /// Starts the Actor and waits (client-side polling) for it to finish.
    ///
    /// `wait_secs` bounds the wait; `None` waits indefinitely. Returns the finished run
    /// (or the still-running run if the wait budget was exhausted).
    pub async fn call<T: Serialize>(
        &self,
        input: Option<&T>,
        options: ActorStartOptions,
        wait_secs: Option<i64>,
    ) -> ApifyClientResult<ActorRun> {
        let run = self.start(input, options).await?;
        // Use the root client's run client so polling targets the canonical run route.
        self.root.run(run.id).wait_for_finish(wait_secs).await
    }

    /// Builds the given version of the Actor and returns the created build.
    pub async fn build(
        &self,
        version_number: &str,
        options: ActorBuildOptions,
    ) -> ApifyClientResult<Build> {
        let mut params = QueryParams::new();
        params
            .add_str("version", Some(version_number.to_string()))
            .add_bool("betaPackages", options.beta_packages)
            .add_str("tag", options.tag)
            .add_bool("useCache", options.use_cache)
            .add_int("waitForFinish", options.wait_for_finish);
        post_with_body(&self.ctx, Some("builds"), &params, None, "application/json").await
    }

    /// Resolves the Actor's default build and returns a client for it.
    ///
    /// `wait_for_finish` optionally bounds how long (in seconds) the API waits for the build
    /// to finish before responding, matching the reference client's `defaultBuild(options)`.
    pub async fn default_build(
        &self,
        wait_for_finish: Option<i64>,
    ) -> ApifyClientResult<BuildClient> {
        let mut params = QueryParams::new();
        params.add_int("waitForFinish", wait_for_finish);
        let url = params.apply_to_url(&self.ctx.url(Some("builds/default")));
        let response = self
            .ctx
            .http
            .call(crate::http_client::HttpRequest {
                method: crate::http_client::HttpMethod::Get,
                url,
                headers: Default::default(),
                body: None,
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        let build: Build = parse_data_envelope(&response.body)?;
        Ok(BuildClient::new(
            self.ctx.http.clone(),
            &self.base_url,
            &build.id,
        ))
    }

    /// Validates the given input against the Actor's input schema.
    pub async fn validate_input<T: Serialize>(&self, input: &T) -> ApifyClientResult<Value> {
        let body = serde_json::to_vec(input)?;
        crate::clients::base::post_with_body(
            &self.ctx,
            Some("validate-input"),
            &QueryParams::new(),
            Some(body),
            "application/json",
        )
        .await
    }

    /// Returns a client for the last run of this Actor, optionally filtered by status.
    pub fn last_run(&self, status: Option<&str>) -> RunClient {
        let mut client = RunClient::new(
            self.root.clone(),
            self.ctx.http.clone(),
            &self.ctx.url(None),
            "runs",
            "last",
        );
        if let Some(status) = status {
            client.set_status_param(status);
        }
        client
    }

    /// Returns a client for this Actor's build collection.
    pub fn builds(&self) -> BuildCollectionClient {
        BuildCollectionClient::with_base(self.ctx.http.clone(), &self.ctx.url(None), "builds")
    }

    /// Returns a client for this Actor's run collection.
    pub fn runs(&self) -> RunCollectionClient {
        RunCollectionClient::new(self.ctx.http.clone(), &self.ctx.url(None), "runs")
    }

    /// Returns a client for a specific version of this Actor.
    pub fn version(&self, version_number: &str) -> ActorVersionClient {
        ActorVersionClient::new(self.ctx.http.clone(), &self.ctx.url(None), version_number)
    }

    /// Returns a client for this Actor's version collection.
    pub fn versions(&self) -> ActorVersionCollectionClient {
        ActorVersionCollectionClient::new(self.ctx.http.clone(), &self.ctx.url(None))
    }

    /// Returns a client for this Actor's webhook collection.
    pub fn webhooks(&self) -> WebhookCollectionClient {
        WebhookCollectionClient::with_base(self.ctx.http.clone(), &self.ctx.url(None))
    }

    /// The Actor's ID (or `username~name`) as provided.
    pub fn id(&self) -> &str {
        &self.id
    }
}
