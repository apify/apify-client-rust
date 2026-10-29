//! Client for an Actor's version collection (`/v2/actors/{actorId}/versions`).

use serde::Serialize;

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::clients::pagination::{single_page_iterator, ListIterator};
use crate::common::{PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorVersion;
use crate::timeouts::Timeout;

/// Client for listing and creating Actor versions.
#[derive(Debug, Clone)]
pub struct ActorVersionCollectionClient {
    ctx: ResourceContext,
}

crate::clients::base::impl_with_timeout!(ActorVersionCollectionClient);

impl ActorVersionCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "versions"),
        }
    }

    /// Lists the Actor's versions. The endpoint takes no query parameters and returns every
    /// version in a single page.
    pub async fn list(&self) -> ApifyClientResult<PaginationList<ActorVersion>> {
        list_resource(&self.ctx, None, &QueryParams::new(), Timeout::Short).await
    }

    /// Lazily iterates over the Actor's versions.
    ///
    /// The listing is not offset-paginated (the API returns every version in a single page), so this
    /// yields all of them from that one page and then completes. It exists for interface parity
    /// with the other collection clients and the reference client.
    pub fn iterate(&self) -> ListIterator<ActorVersion> {
        single_page_iterator!(self)
    }

    /// Creates a new Actor version.
    pub async fn create<T: Serialize>(&self, version: &T) -> ApifyClientResult<ActorVersion> {
        create_resource(&self.ctx, &QueryParams::new(), version, Timeout::Short).await
    }
}
