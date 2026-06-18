//! Client for an Actor-run collection (`/v2/actor-runs`, `/v2/actors/{id}/runs`, etc.).

use crate::clients::base::{list_resource, ResourceContext};
use crate::common::{ListOptions, PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorRun;

/// Client for listing Actor runs.
#[derive(Debug, Clone)]
pub struct RunCollectionClient {
    ctx: ResourceContext,
}

impl RunCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, resource_path: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, resource_path),
        }
    }

    /// Lists runs, with optional `status` filtering and offset/limit pagination.
    pub async fn list(
        &self,
        options: ListOptions,
        status: Option<&str>,
    ) -> ApifyClientResult<PaginationList<ActorRun>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc)
            .add_str("status", status.map(|s| s.to_string()));
        list_resource(&self.ctx, None, &params).await
    }
}
