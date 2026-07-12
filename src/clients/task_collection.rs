//! Client for the Actor task collection (`/v2/actor-tasks`).

use serde::Serialize;

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::clients::pagination::{list_iterator, ListIterator};
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
    ///
    /// `options.limit` caps the *total* number of items yielded across all pages, unlike
    /// [`list`](Self::list) where `limit` is a single page's size. Set the per-page fetch size
    /// with [`with_chunk_size`](crate::ListIterator::with_chunk_size); see
    /// [`ListIterator`] for details.
    pub fn iterate(&self, options: ListOptions) -> ListIterator<Task> {
        list_iterator!(self, options, list)
    }

    /// Creates a new task from the given definition.
    pub async fn create<T: Serialize>(&self, task: &T) -> ApifyClientResult<Task> {
        create_resource(&self.ctx, &QueryParams::new(), task).await
    }
}
