//! Client for a single Actor version (`/v2/actors/{actorId}/versions/{versionNumber}`).

use serde::Serialize;

use crate::clients::actor_env_var::ActorEnvVarClient;
use crate::clients::actor_env_var_collection::ActorEnvVarCollectionClient;
use crate::clients::base::{delete_resource, get_resource, update_resource, ResourceContext};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorVersion;

/// Client for a specific Actor version.
#[derive(Debug, Clone)]
pub struct ActorVersionClient {
    ctx: ResourceContext,
}

impl ActorVersionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, version_number: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "versions", version_number),
        }
    }

    /// Fetches the version, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<ActorVersion>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the version with the given fields.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<ActorVersion> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the version.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Returns a client for a specific environment variable of this version.
    pub fn env_var(&self, name: &str) -> ActorEnvVarClient {
        ActorEnvVarClient::new(self.ctx.http.clone(), &self.ctx.url(None), name)
    }

    /// Returns a client for this version's environment variable collection.
    pub fn env_vars(&self) -> ActorEnvVarCollectionClient {
        ActorEnvVarCollectionClient::new(self.ctx.http.clone(), &self.ctx.url(None))
    }
}
