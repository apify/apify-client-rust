//! Client for an Actor-run collection (`/v2/actor-runs`, `/v2/actors/{id}/runs`, etc.).

use crate::clients::base::{list_resource, ResourceContext};
use crate::common::{ListOptions, PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorRun;

/// Filtering options for [`RunCollectionClient::list`], covering the spec query parameters of
/// `GET /v2/actor-runs` and `GET /v2/actors/{actorId}/runs`.
///
/// As in the reference client, the same options type is reused for the task-scoped run
/// collection (`GET /v2/actor-tasks/{actorTaskId}/runs`), whose spec does not define
/// `startedAfter`/`startedBefore`; those params are simply ignored server-side when set on a
/// task-scoped list. This deliberate reference-parity reuse keeps the API surface uniform.
#[derive(Debug, Default, Clone)]
pub struct RunListOptions {
    /// Only return runs with this status (e.g. `SUCCEEDED`, `RUNNING`).
    pub status: Option<String>,
    /// Only return runs started at or after this ISO 8601 timestamp. (Actor-scoped lists only.)
    pub started_after: Option<String>,
    /// Only return runs started at or before this ISO 8601 timestamp. (Actor-scoped lists only.)
    pub started_before: Option<String>,
}

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

    /// Lists runs with offset/limit pagination and optional status/started-time filtering.
    pub async fn list(
        &self,
        options: ListOptions,
        filter: RunListOptions,
    ) -> ApifyClientResult<PaginationList<ActorRun>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc)
            .add_str("status", filter.status)
            .add_str("startedAfter", filter.started_after)
            .add_str("startedBefore", filter.started_before);
        list_resource(&self.ctx, None, &params).await
    }
}
