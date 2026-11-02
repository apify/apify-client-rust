//! Client for an Actor version's environment variable collection.

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::clients::pagination::{single_page_iterator, ListIterator};
use crate::common::{PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorEnvVar;
use crate::timeouts::Timeout;

/// Client for listing and creating environment variables of an Actor version.
#[derive(Debug, Clone)]
pub struct ActorEnvVarCollectionClient {
    ctx: ResourceContext,
}

crate::clients::base::impl_with_timeout!(ActorEnvVarCollectionClient);

impl ActorEnvVarCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "env-vars"),
        }
    }

    /// Lists the environment variables of the Actor version.
    pub async fn list(&self) -> ApifyClientResult<PaginationList<ActorEnvVar>> {
        list_resource(&self.ctx, None, &QueryParams::new(), Timeout::Short).await
    }

    /// Lazily iterates over the Actor version's environment variables.
    ///
    /// The listing is not offset-paginated (the API returns every variable in a single page), so this
    /// yields all of them from that one page and then completes. It exists for interface parity
    /// with the other collection clients and the reference client.
    pub fn iterate(&self) -> ListIterator<ActorEnvVar> {
        single_page_iterator!(self)
    }

    /// Creates a new environment variable.
    pub async fn create(&self, env_var: &ActorEnvVar) -> ApifyClientResult<ActorEnvVar> {
        create_resource(&self.ctx, &QueryParams::new(), env_var, Timeout::Short).await
    }
}
