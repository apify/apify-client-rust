//! Client for an Actor version's environment variable collection.

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorEnvVar;

/// Client for listing and creating environment variables of an Actor version.
#[derive(Debug, Clone)]
pub struct ActorEnvVarCollectionClient {
    ctx: ResourceContext,
}

impl ActorEnvVarCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "env-vars"),
        }
    }

    /// Lists the environment variables of the Actor version.
    pub async fn list(&self) -> ApifyClientResult<PaginationList<ActorEnvVar>> {
        list_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Lazily iterates over the Actor version's environment variables.
    ///
    /// The env-var listing is not offset-paginated (the API returns every variable in a single
    /// page), so this yields all variables from that one page and then completes. It exists for
    /// interface parity with the other collection clients and the reference client. Built with
    /// [`ListIterator::new_single_page`], which fetches exactly once and never re-requests.
    pub fn iterate(&self) -> ListIterator<ActorEnvVar> {
        let client = self.clone();
        ListIterator::new_single_page(Box::new(move |_offset, _page_limit| {
            let client = client.clone();
            Box::pin(async move { client.list().await })
        }))
    }

    /// Creates a new environment variable.
    pub async fn create(&self, env_var: &ActorEnvVar) -> ApifyClientResult<ActorEnvVar> {
        create_resource(&self.ctx, &QueryParams::new(), env_var).await
    }
}
