//! Client for the dataset collection (`/v2/datasets`).

use crate::clients::base::{get_or_create_named, list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{PaginationList, QueryParams, StorageListOptions};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Dataset;

/// Client for listing datasets and getting-or-creating a dataset by name.
#[derive(Debug, Clone)]
pub struct DatasetCollectionClient {
    ctx: ResourceContext,
}

impl DatasetCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "datasets"),
        }
    }

    /// Lists datasets with offset/limit pagination, optionally filtering by `unnamed`/`ownership`.
    pub async fn list(
        &self,
        options: StorageListOptions,
    ) -> ApifyClientResult<PaginationList<Dataset>> {
        let mut params = QueryParams::new();
        options.apply(&mut params);
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates over all datasets matching `options`, fetching pages on demand.
    pub fn iterate(&self, options: StorageListOptions) -> ListIterator<Dataset> {
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

    /// Gets the dataset with the given `name`, creating it if it does not exist.
    ///
    /// Passing `None` for `name` creates an unnamed dataset.
    pub async fn get_or_create(&self, name: Option<&str>) -> ApifyClientResult<Dataset> {
        get_or_create_named(&self.ctx, name).await
    }
}
