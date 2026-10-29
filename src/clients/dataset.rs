//! Client for a single dataset (`/v2/datasets/{datasetId}` and run-nested variants).

use serde::de::{DeserializeOwned, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::clients::base::{
    delete_resource, get_byte_stream, get_resource, get_resource_required, update_resource,
    ResourceContext,
};
use crate::clients::pagination::ListIterator;
use crate::common::{sign_storage_content, PaginationList, QueryParams};
use crate::error::{ApifyClientError, ApifyClientResult};
use crate::http_client::{CallOptions, HttpClient, HttpMethod};
use crate::models::Dataset;
use crate::timeouts::Timeout;

/// Checks that a serialized `push_items` body is a JSON object or an array of JSON objects,
/// without building a value out of it.
fn check_items_shape(body: &[u8]) -> ApifyClientResult<()> {
    serde_json::from_slice::<ItemsShape>(body)
        .map(|_| ())
        .map_err(|err| {
            ApifyClientError::InvalidArgument(format!(
                "push_items takes a JSON object or an array of JSON objects: {err}"
            ))
        })
}

/// Marker deserialized from a JSON object or an array of JSON objects, and nothing else.
struct ItemsShape;

impl<'de> Deserialize<'de> for ItemsShape {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ItemsShapeVisitor)
    }
}

struct ItemsShapeVisitor;

impl<'de> Visitor<'de> for ItemsShapeVisitor {
    type Value = ItemsShape;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a JSON object or an array of JSON objects")
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<ItemsShape, A::Error> {
        skip_object(map)?;
        Ok(ItemsShape)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<ItemsShape, A::Error> {
        while seq.next_element::<ObjectShape>()?.is_some() {}
        Ok(ItemsShape)
    }
}

/// Marker deserialized from a JSON object and nothing else.
struct ObjectShape;

impl<'de> Deserialize<'de> for ObjectShape {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor;

        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = ObjectShape;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<ObjectShape, A::Error> {
                skip_object(map)?;
                Ok(ObjectShape)
            }
        }

        deserializer.deserialize_map(ObjectVisitor)
    }
}

/// Reads the entries of a JSON object without keeping them.
fn skip_object<'de, A: MapAccess<'de>>(mut map: A) -> Result<(), A::Error> {
    while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
    Ok(())
}

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

crate::clients::base::impl_with_timeout!(DatasetClient);

impl DatasetClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, resource_path: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, resource_path, id),
        }
    }

    /// Creates a dataset client for a run's default dataset (no ID; nested path only).
    pub(crate) fn nested(
        http: HttpClient,
        base_url: &str,
        sub_path: &str,
        base_params: &QueryParams,
    ) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, sub_path)
                .with_base_params(base_params),
        }
    }

    /// Sets the public origin used when building shareable URLs.
    pub(crate) fn with_public_base(mut self, public_base_url: &str) -> Self {
        self.ctx = self.ctx.with_public_origin(public_base_url);
        self
    }

    /// Fetches the dataset metadata, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Dataset>> {
        get_resource(&self.ctx, None, &QueryParams::new(), Timeout::Short).await
    }

    /// Updates the dataset metadata (e.g. `name`, `title`).
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<Dataset> {
        update_resource(&self.ctx, None, new_fields, Timeout::Short).await
    }

    /// Deletes the dataset.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None, Timeout::Short).await
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
        let url = self.ctx.url_with_params(Some("items"), &params);
        let response = self
            .ctx
            .send(HttpMethod::Get, url, self.ctx.timeout(Timeout::Long))
            .await?;

        let items: Vec<T> = serde_json::from_slice(&response.body)?;
        let count = items.len() as i64;
        // `PaginationList::total` is informational only here (`ListIterator` never consults it,
        // see `scanned` below); falling back to `0` rather than `count` just avoids implying a
        // precise total when the API did not report one.
        let total = response
            .header("x-apify-pagination-total")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let offset = response
            .header("x-apify-pagination-offset")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let limit = response
            .header("x-apify-pagination-limit")
            .and_then(|v| v.parse().ok())
            .unwrap_or(count);
        // The number of rows scanned to produce this page, when the API reports it separately
        // from the item count (see `PaginationList::scanned`): a filter can scan more rows than
        // it returns, and `unwind` can return more items than rows scanned.
        let scanned = response
            .header("x-apify-pagination-count")
            .and_then(|v| v.parse().ok());

        Ok(PaginationList {
            total,
            offset,
            limit,
            count,
            desc: options.desc.unwrap_or(false),
            items,
            scanned,
        })
    }

    /// Lazily iterates over all items in the dataset, fetching pages on demand.
    ///
    /// The idiomatic-Rust counterpart of the reference client's async-iterable
    /// `listItems`/`iterateItems`: yields one deserialized item of type `T` at a time,
    /// transparently paging. The caller's `options.limit` caps the total number of items yielded
    /// (unset = all); use [`ListIterator::with_chunk_size`] to control the per-page fetch size.
    ///
    /// Server-side filters (`skip_empty`/`clean`/`skip_hidden`) and `unwind` are forwarded on
    /// every page request. Paging advances the offset by the number of underlying rows each page
    /// *scanned* (the `x-apify-pagination-count` response header), not by `items.len()`, matching
    /// the reference JavaScript client: a filter can scan more rows than it returns, and `unwind`
    /// can return more items than rows scanned, so advancing by the returned count alone would
    /// re-scan or skip rows. The listing's reported `total` is never consulted, so a dataset a
    /// still-running Actor keeps pushing to is read to the end: iteration keeps requesting pages
    /// until one scans no rows (one extra, empty request past the last page with data), rather
    /// than stopping at the size the dataset had when the loop started.
    ///
    /// If `x-apify-pagination-count` is ever absent, the scanned count falls back to `items.len()`.
    pub fn iterate_items<T: DeserializeOwned + Send + 'static>(
        &self,
        options: DatasetListItemsOptions,
    ) -> ListIterator<T> {
        let client = self.clone();
        let start = options.offset.unwrap_or(0);
        let total_limit = options.limit;
        ListIterator::new(
            start,
            total_limit,
            Box::new(move |offset, page_limit| {
                let client = client.clone();
                let mut options = options.clone();
                options.offset = Some(offset);
                options.limit = page_limit;
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
        let url = self.ctx.url_with_params(Some("items"), &params);
        let response = self
            .ctx
            .send(HttpMethod::Get, url, self.ctx.timeout(Timeout::Long))
            .await?;
        Ok(response.body)
    }

    /// Like [`download_items`](Self::download_items), but streams the export as it arrives
    /// instead of buffering the whole response in memory.
    ///
    /// Returns a `Stream` of byte chunks; drive it with `futures_util::StreamExt::next()` (or
    /// pipe it through an `AsyncRead` adapter) to process a large export in constant memory. A
    /// gap between chunks longer than the [`Timeout::Long`] tier (6 minutes by default) ends the
    /// stream with [`ApifyClientError::Timeout`] — an
    /// inactivity timeout, not a cap on the export's total duration. Use
    /// [`with_timeout`](Self::with_timeout) to change it.
    ///
    /// Goes through the configured `HttpBackend`'s streaming path instead of the buffered one,
    /// to keep the connection open — like [`crate::clients::log::LogClient::stream`], a failed
    /// request (e.g. a `404`) still surfaces the same structured [`crate::error::ApiError`] as
    /// the buffered endpoints, from reading the error body before the stream is handed back.
    pub async fn download_items_stream(
        &self,
        format: DownloadItemsFormat,
        options: DatasetDownloadOptions,
    ) -> ApifyClientResult<impl futures_util::Stream<Item = ApifyClientResult<Vec<u8>>>> {
        let mut params = QueryParams::new();
        params.add_str("format", Some(format.as_str()));
        options.apply(&mut params);
        let url = self.ctx.url_with_params(Some("items"), &params);
        get_byte_stream(&self.ctx, url, Timeout::Long).await
    }

    /// Pushes one or more items to the dataset.
    ///
    /// `items` must serialize to a JSON object or an array of objects, the only shapes the
    /// endpoint accepts. Anything else, such as a plain string or an array of strings, is
    /// rejected with [`ApifyClientError::InvalidArgument`]
    /// before a request is sent.
    ///
    /// A push that times out is not retried, because the request may have reached the API and a
    /// retry could store the items twice; the other errors are retried as usual.
    pub async fn push_items<T: Serialize>(&self, items: &T) -> ApifyClientResult<()> {
        let body = serde_json::to_vec(items)?;
        check_items_shape(&body)?;
        let url = self.ctx.url_with_params(Some("items"), &QueryParams::new());
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        self.ctx
            .send_with_options(
                HttpMethod::Post,
                url,
                headers,
                Some(body),
                self.ctx.timeout(Timeout::Medium),
                CallOptions {
                    do_not_retry_timeouts: true,
                },
            )
            .await?;
        Ok(())
    }

    /// Builds a public URL for downloading this dataset's items.
    ///
    /// Mirrors the reference client's `createItemsPublicUrl`: it fetches the dataset, and if
    /// the dataset exposes a URL-signing secret key (i.e. it is private), appends an
    /// HMAC-SHA256 `signature` so the URL grants access without an API token. `expires_in_secs`
    /// optionally bounds the validity of a signed URL. The URL is built from the configured
    /// public base URL. The served items are JSON; to request another export format, use
    /// [`create_items_public_url_with_format`](Self::create_items_public_url_with_format).
    pub async fn create_items_public_url(
        &self,
        options: DatasetListItemsOptions,
        expires_in_secs: Option<i64>,
    ) -> ApifyClientResult<String> {
        self.create_items_public_url_with_format(options, expires_in_secs, None)
            .await
    }

    /// Like [`create_items_public_url`](Self::create_items_public_url), but lets the caller
    /// choose the `format` the items are served in (JSON, CSV, XLSX, ...) via a `format` query
    /// parameter on the generated URL. `None` defaults to `json`, matching the endpoint's own
    /// default.
    pub async fn create_items_public_url_with_format(
        &self,
        options: DatasetListItemsOptions,
        expires_in_secs: Option<i64>,
        format: Option<DownloadItemsFormat>,
    ) -> ApifyClientResult<String> {
        let mut params = QueryParams::new();
        options.apply(&mut params);
        params.add_str("format", format.map(|f| f.as_str().to_string()));

        // Like the reference client, fetches the dataset with the `Long` tier of this method.
        let dataset: Option<Dataset> =
            get_resource(&self.ctx, None, &QueryParams::new(), Timeout::Long).await?;
        if let Some(dataset) = dataset {
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

    /// Returns statistical information about the dataset.
    ///
    /// A dataset that does not exist is a [`NotFound`](crate::ApiError::is_not_found) error. The
    /// response cannot tell a missing dataset from a missing statistics resource, and a client
    /// chained from a run, such as `run.dataset()`, cannot tell a missing run either.
    pub async fn get_statistics(&self) -> ApifyClientResult<Value> {
        get_resource_required(
            &self.ctx,
            Some("statistics"),
            &QueryParams::new(),
            Timeout::Short,
        )
        .await
    }
}
