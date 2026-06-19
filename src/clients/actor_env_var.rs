//! Client for a single Actor-version environment variable.

use crate::clients::base::{delete_resource, get_resource, update_resource, ResourceContext};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorEnvVar;

/// Client for a specific environment variable of an Actor version.
#[derive(Debug, Clone)]
pub struct ActorEnvVarClient {
    ctx: ResourceContext,
}

impl ActorEnvVarClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, name: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "env-vars", name),
        }
    }

    /// Fetches the environment variable, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<ActorEnvVar>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the environment variable.
    pub async fn update(&self, env_var: &ActorEnvVar) -> ApifyClientResult<ActorEnvVar> {
        update_resource(&self.ctx, None, env_var).await
    }

    /// Deletes the environment variable.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }
}
