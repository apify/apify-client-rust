//! Client for the dataset collection (`/v2/datasets`).

use crate::clients::base::{get_or_create_named_with_schema, list_resource, ResourceContext};
use crate::clients::pagination::{list_iterator, ListIterator};
use crate::common::{PaginationList, QueryParams, StorageListOptions};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Dataset;

/// Options for getting-or-creating a dataset via
/// [`DatasetCollectionClient::get_or_create_with_options`].
///
/// `schema` is a JS-reference-only convenience (not documented by the OpenAPI spec, which only
/// declares the `name` query parameter on `POST /v2/datasets`); it is sent as the request's
/// `{ "schema": ... }` JSON body, matching the reference client's
/// `DatasetCollectionClientGetOrCreateOptions`.
#[derive(Debug, Default, Clone)]
pub struct DatasetGetOrCreateOptions {
    /// JSON schema to associate with the dataset.
    pub schema: Option<serde_json::Value>,
}

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
    ///
    /// `options.limit` caps the *total* number of items yielded across all pages, unlike
    /// [`list`](Self::list) where `limit` is a single page's size. Set the per-page fetch size
    /// with [`with_chunk_size`](crate::ListIterator::with_chunk_size); see
    /// [`ListIterator`] for details.
    pub fn iterate(&self, options: StorageListOptions) -> ListIterator<Dataset> {
        list_iterator!(self, options, list)
    }

    /// Gets the dataset with the given `name`, creating it if it does not exist.
    ///
    /// Passing `None` for `name` creates an unnamed dataset.
    pub async fn get_or_create(&self, name: Option<&str>) -> ApifyClientResult<Dataset> {
        self.get_or_create_with_options(name, DatasetGetOrCreateOptions::default())
            .await
    }

    /// Gets the dataset with the given `name`, creating it if it does not exist, applying the
    /// given [`DatasetGetOrCreateOptions`] (e.g. [`DatasetGetOrCreateOptions::schema`], applied
    /// only when the dataset is created).
    pub async fn get_or_create_with_options(
        &self,
        name: Option<&str>,
        options: DatasetGetOrCreateOptions,
    ) -> ApifyClientResult<Dataset> {
        get_or_create_named_with_schema(&self.ctx, name, options.schema.as_ref()).await
    }
}
