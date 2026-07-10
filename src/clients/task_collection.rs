//! Client for the Actor task collection (`/v2/actor-tasks`).

use serde::Serialize;

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{ListOptions, PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Task;

/// Client for listing and creating Actor tasks.
#[derive(Debug, Clone)]
pub struct TaskCollectionClient {
    ctx: ResourceContext,
}

impl TaskCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "actor-tasks"),
        }
    }

    /// Lists tasks with offset/limit pagination.
    pub async fn list(&self, options: ListOptions) -> ApifyClientResult<PaginationList<Task>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all tasks matching `options`, fetching pages on demand.
    pub fn iterate(&self, options: ListOptions) -> ListIterator<Task> {
        let client = self.clone();
        let start = options.offset.unwrap_or(0);
        ListIterator::new(
            start,
            Box::new(move |offset| {
                let client = client.clone();
                let mut options = options.clone();
                options.offset = Some(offset);
                Box::pin(async move { client.list(options).await })
            }),
        )
    }

    /// Creates a new task from the given definition.
    pub async fn create<T: Serialize>(&self, task: &T) -> ApifyClientResult<Task> {
        create_resource(&self.ctx, &QueryParams::new(), task).await
    }
}
