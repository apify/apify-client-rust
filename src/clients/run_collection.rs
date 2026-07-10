//! Client for an Actor-run collection (`/v2/actor-runs`, `/v2/actors/{id}/runs`, etc.).

use crate::clients::base::{list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
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
    /// Only return runs with one of these statuses (e.g. `SUCCEEDED`, `RUNNING`). The API
    /// accepts multiple statuses; they are sent as a comma-separated list. Empty means no
    /// status filter. Matches the reference client, whose `status` accepts a string or array.
    pub status: Vec<String>,
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
    ///
    /// Unlike other collections, run listing takes two arguments: the shared [`ListOptions`]
    /// (offset/limit/desc) and a [`RunListOptions`] filter (status and start-time bounds).
    ///
    /// # Example
    /// ```no_run
    /// use apify_client::{ApifyClient, ListOptions, RunListOptions};
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ApifyClient::new("my-api-token");
    /// let runs = client
    ///     .runs()
    ///     .list(
    ///         ListOptions { limit: Some(10), desc: Some(true), ..Default::default() },
    ///         RunListOptions { status: vec!["SUCCEEDED".to_string()], ..Default::default() },
    ///     )
    ///     .await?;
    /// for run in runs.items {
    ///     println!("{}", run.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
            .add_csv("status", Some(&filter.status))
            .add_str("startedAfter", filter.started_after)
            .add_str("startedBefore", filter.started_before);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all runs matching `options`/`filter`, fetching pages on demand.
    ///
    /// The idiomatic-Rust counterpart of the reference client's async-iterable run listing;
    /// yields one [`ActorRun`] at a time across all pages.
    pub fn iterate(&self, options: ListOptions, filter: RunListOptions) -> ListIterator<ActorRun> {
        let client = self.clone();
        let start = options.offset.unwrap_or(0);
        ListIterator::new(
            start,
            Box::new(move |offset| {
                let client = client.clone();
                let mut options = options.clone();
                let filter = filter.clone();
                options.offset = Some(offset);
                Box::pin(async move { client.list(options, filter).await })
            }),
        )
    }
}
