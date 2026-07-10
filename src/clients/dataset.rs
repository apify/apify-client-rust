//! Client for a single dataset (`/v2/datasets/{datasetId}` and run-nested variants).

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::clients::base::{delete_resource, get_resource, update_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{parse_data_envelope, sign_storage_content, PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, HttpMethod, HttpRequest};
use crate::models::Dataset;

/// Options for listing or downloading dataset items.
///
/// Covers the filtering, projection and transformation parameters of
/// `GET /v2/datasets/{datasetId}/items`.
#[derive(Debug, Default, Clone)]
pub struct DatasetListItemsOptions {
    /// Number of items to skip.
    pub offset: Option<i64>,
    /// Maximum number of items to return.
    pub limit: Option<i64>,
    /// Return items newest-first.
    pub desc: Option<bool>,
    /// Only include these fields.
    pub fields: Option<Vec<String>>,
    /// Positionally renames the fields selected by `fields` in the output (requires `fields`
    /// to be set). The i-th name here becomes the output name of the i-th `fields` entry.
    pub output_fields: Option<Vec<String>>,
    /// Exclude these fields.
    pub omit: Option<Vec<String>>,
    /// Skip empty items.
    pub skip_empty: Option<bool>,
    /// Skip hidden fields (those starting with `#`).
    pub skip_hidden: Option<bool>,
    /// Only return clean (non-empty, non-hidden) items.
    pub clean: Option<bool>,
    /// Unwind these fields (each array element becomes a separate item).
    pub unwind: Option<Vec<String>>,
    /// Flatten these nested fields into dot-notation keys.
    pub flatten: Option<Vec<String>>,
    /// Use a predefined dataset view for field selection.
    pub view: Option<String>,
    /// Return simplified (flattened, cleaned) items.
    pub simplified: Option<bool>,
    /// Skip items that come from failed pages.
    pub skip_failed_pages: Option<bool>,
    /// Pre-shared URL signature granting access to a private dataset without an API token.
    pub signature: Option<String>,
}

impl DatasetListItemsOptions {
    fn apply(&self, params: &mut QueryParams) {
        params
            .add_int("offset", self.offset)
            .add_int("limit", self.limit)
            .add_bool("desc", self.desc)
            .add_csv("fields", self.fields.as_deref())
            .add_csv("outputFields", self.output_fields.as_deref())
            .add_csv("omit", self.omit.as_deref())
            .add_bool("skipEmpty", self.skip_empty)
            .add_bool("skipHidden", self.skip_hidden)
            .add_bool("clean", self.clean)
            .add_csv("unwind", self.unwind.as_deref())
            .add_csv("flatten", self.flatten.as_deref())
            .add_str("view", self.view.clone())
            .add_bool("simplified", self.simplified)
            .add_bool("skipFailedPages", self.skip_failed_pages)
            .add_str("signature", self.signature.clone());
    }
}

/// Output formats supported by [`DatasetClient::download_items`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadItemsFormat {
    /// JSON array.
    Json,
    /// Newline-delimited JSON.
    Jsonl,
    /// Comma-separated values.
    Csv,
    /// Microsoft Excel (XLSX).
    Xlsx,
    /// XML.
    Xml,
    /// RSS feed.
    Rss,
    /// HTML table.
    Html,
}

impl DownloadItemsFormat {
    fn as_str(&self) -> &'static str {
        match self {
            DownloadItemsFormat::Json => "json",
            DownloadItemsFormat::Jsonl => "jsonl",
            DownloadItemsFormat::Csv => "csv",
            DownloadItemsFormat::Xlsx => "xlsx",
            DownloadItemsFormat::Xml => "xml",
            DownloadItemsFormat::Rss => "rss",
            DownloadItemsFormat::Html => "html",
        }
    }
}

/// Format-specific options for [`DatasetClient::download_items`], on top of the shared
/// filtering/projection options.
#[derive(Debug, Default, Clone)]
pub struct DatasetDownloadOptions {
    /// Shared item filtering/projection options.
    pub items: DatasetListItemsOptions,
    /// Set `Content-Disposition: attachment` on the response.
    pub attachment: Option<bool>,
    /// Prepend a UTF-8 BOM (useful for Excel-compatible CSV).
    pub bom: Option<bool>,
    /// CSV field delimiter (default `,`).
    pub delimiter: Option<String>,
    /// Omit the CSV header row.
    pub skip_header_row: Option<bool>,
    /// Name of the root XML element (default `items`).
    pub xml_root: Option<String>,
    /// Name of the per-item XML element (default `item`).
    pub xml_row: Option<String>,
    /// Title to use for RSS/Atom feed exports.
    pub feed_title: Option<String>,
    /// Description to use for RSS/Atom feed exports.
    pub feed_description: Option<String>,
}

impl DatasetDownloadOptions {
    fn apply(&self, params: &mut QueryParams) {
        self.items.apply(params);
        params
            .add_bool("attachment", self.attachment)
            .add_bool("bom", self.bom)
            .add_str("delimiter", self.delimiter.clone())
            .add_bool("skipHeaderRow", self.skip_header_row)
            .add_str("xmlRoot", self.xml_root.clone())
            .add_str("xmlRow", self.xml_row.clone())
            .add_str("feedTitle", self.feed_title.clone())
            .add_str("feedDescription", self.feed_description.clone());
    }
}

/// Client for a specific dataset.
#[derive(Debug, Clone)]
pub struct DatasetClient {
    ctx: ResourceContext,
}

impl DatasetClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, resource_path: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, resource_path, id),
        }
    }

    /// Creates a dataset client for a run's default dataset (no ID; nested path only).
    pub(crate) fn nested(http: HttpClient, base_url: &str, sub_path: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, sub_path),
        }
    }

    /// Sets the public origin used when building shareable URLs.
    pub(crate) fn with_public_base(mut self, public_base_url: &str) -> Self {
        self.ctx = self.ctx.with_public_origin(public_base_url);
        self
    }

    /// Fetches the dataset metadata, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Dataset>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the dataset metadata (e.g. `name`, `title`).
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<Dataset> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the dataset.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Lists items from the dataset.
    ///
    /// The dataset items endpoint returns a bare JSON array (not a `data` envelope) and
    /// reports pagination via `X-Apify-Pagination-*` headers, which are surfaced in the
    /// returned [`PaginationList`].
    pub async fn list_items<T: DeserializeOwned>(
        &self,
        options: DatasetListItemsOptions,
    ) -> ApifyClientResult<PaginationList<T>> {
        let mut params = QueryParams::new();
        options.apply(&mut params);
        let url = params.apply_to_url(&self.ctx.url(Some("items")));
        let response = self
            .ctx
            .http
            .call(HttpRequest {
                method: HttpMethod::Get,
                url,
                headers: Default::default(),
                body: None,
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;

        let items: Vec<T> = serde_json::from_slice(&response.body)?;
        let count = items.len() as i64;
        let total = response
            .header("x-apify-pagination-total")
            .and_then(|v| v.parse().ok())
            .unwrap_or(count);
        let offset = response
            .header("x-apify-pagination-offset")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let limit = response
            .header("x-apify-pagination-limit")
            .and_then(|v| v.parse().ok())
            .unwrap_or(count);

        Ok(PaginationList {
            total,
            offset,
            limit,
            count,
            desc: options.desc.unwrap_or(false),
            items,
        })
    }

    /// Lazily iterates over all items in the dataset, fetching pages on demand.
    ///
    /// The idiomatic-Rust counterpart of the reference client's async-iterable
    /// `listItems`/`iterateItems`: yields one deserialized item of type `T` at a time,
    /// transparently paging with the caller's `options` (its `limit` acts as the page size).
    pub fn iterate_items<T: DeserializeOwned + Send + 'static>(
        &self,
        options: DatasetListItemsOptions,
    ) -> ListIterator<T> {
        let client = self.clone();
        let start = options.offset.unwrap_or(0);
        ListIterator::new(
            start,
            Box::new(move |offset| {
                let client = client.clone();
                let mut options = options.clone();
                options.offset = Some(offset);
                Box::pin(async move { client.list_items::<T>(options).await })
            }),
        )
    }

    /// Downloads dataset items serialized in the given `format`, returning the raw bytes.
    ///
    /// Unlike [`list_items`](Self::list_items), which returns parsed items, this returns the
    /// items already serialized to JSON, CSV, XLSX, XML, RSS or HTML — useful for exporting.
    /// Use [`DatasetDownloadOptions`] to control export-specific behaviour (BOM, CSV
    /// delimiter/header, XML element names, attachment disposition).
    pub async fn download_items(
        &self,
        format: DownloadItemsFormat,
        options: DatasetDownloadOptions,
    ) -> ApifyClientResult<Vec<u8>> {
        let mut params = QueryParams::new();
        params.add_str("format", Some(format.as_str()));
        options.apply(&mut params);
        let url = params.apply_to_url(&self.ctx.url(Some("items")));
        let response = self
            .ctx
            .http
            .call(HttpRequest {
                method: HttpMethod::Get,
                url,
                headers: Default::default(),
                body: None,
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        Ok(response.body)
    }

    /// Pushes one or more items to the dataset.
    ///
    /// `items` must serialize to a JSON object or an array of objects.
    pub async fn push_items<T: Serialize>(&self, items: &T) -> ApifyClientResult<()> {
        let body = serde_json::to_vec(items)?;
        let url = self.ctx.url(Some("items"));
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        self.ctx
            .http
            .call(HttpRequest {
                method: HttpMethod::Post,
                url,
                headers,
                body: Some(body),
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        Ok(())
    }

    /// Builds a public URL for downloading this dataset's items.
    ///
    /// Mirrors the reference client's `createItemsPublicUrl`: it fetches the dataset, and if
    /// the dataset exposes a URL-signing secret key (i.e. it is private), appends an
    /// HMAC-SHA256 `signature` so the URL grants access without an API token. `expires_in_secs`
    /// optionally bounds the validity of a signed URL. The URL is built from the configured
    /// public base URL.
    pub async fn create_items_public_url(
        &self,
        options: DatasetListItemsOptions,
        expires_in_secs: Option<i64>,
    ) -> ApifyClientResult<String> {
        let mut params = QueryParams::new();
        options.apply(&mut params);

        if let Some(dataset) = self.get().await? {
            if let Some(secret) = dataset
                .extra
                .get("urlSigningSecretKey")
                .and_then(|v| v.as_str())
            {
                let signature = sign_storage_content(secret, &dataset.id, expires_in_secs);
                params.add_str("signature", Some(signature));
            }
        }
        Ok(params.apply_to_url(&self.ctx.public_url(Some("items"))))
    }

    /// Returns statistical information about the dataset, or `None` if unavailable.
    pub async fn get_statistics(&self) -> ApifyClientResult<Option<Value>> {
        let result: ApifyClientResult<Value> = async {
            let response = self
                .ctx
                .http
                .call(HttpRequest {
                    method: HttpMethod::Get,
                    url: self.ctx.url(Some("statistics")),
                    headers: Default::default(),
                    body: None,
                    timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
                })
                .await?;
            parse_data_envelope(&response.body)
        }
        .await;
        crate::common::catch_not_found(result)
    }
}
