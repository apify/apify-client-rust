//! Client for the dataset collection (`/v2/datasets`).

use crate::clients::base::{get_or_create_named, list_resource, ResourceContext};
use crate::common::{ListOptions, PaginationList, QueryParams};
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

    /// Lists datasets with offset/limit pagination.
    pub async fn list(&self, options: ListOptions) -> ApifyClientResult<PaginationList<Dataset>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc);
        list_resource(&self.ctx, None, &params).await
    }

    /// Gets the dataset with the given `name`, creating it if it does not exist.
    ///
    /// Passing `None` for `name` creates an unnamed dataset.
    pub async fn get_or_create(&self, name: Option<&str>) -> ApifyClientResult<Dataset> {
        get_or_create_named(&self.ctx, name).await
    }
}
