//! Client for browsing the Apify Store (`/v2/store`).

use crate::clients::base::{list_resource, ResourceContext};
use crate::clients::pagination::{list_iterator, ListIterator};
use crate::common::{PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorStoreListItem;
use crate::timeouts::Timeout;

/// A lazy, page-fetching iterator over Apify Store Actors.
///
/// Returned by [`StoreCollectionClient::iterate`]. This is an alias for the shared
/// [`ListIterator`]; call its `next()` to yield one Actor at a time, fetching further pages
/// from the API transparently until the listing is exhausted.
pub type StoreActorIterator = ListIterator<ActorStoreListItem>;

/// Options for searching the Apify Store.
#[derive(Debug, Default, Clone)]
pub struct StoreListOptions {
    /// Number of items to skip. The API ignores it when `search` is used.
    pub offset: Option<i64>,
    /// Item limit. The default and the maximum are 1000. When `search` is used, they are 100, and
    /// a larger value is reduced to 100. With `include_input_schema`, they are 10. Its meaning
    /// depends on the method: for [`StoreCollectionClient::list`] it is a single page's size (the
    /// maximum items that one call returns); for
    /// [`StoreCollectionClient::iterate`] it is a cap on the *total* number of items yielded across
    /// all pages. See each method's docs.
    pub limit: Option<i64>,
    /// Search query. With it, the API ranks the results by relevance and returns a single page of
    /// at most 100 items: `offset` is ignored and there is no second page. To narrow the results,
    /// use `category`, `username`, or `pricing_model`.
    pub search: Option<String>,
    /// Deprecated and ignored by the API, which accepts it for backward compatibility. Results
    /// are ordered by relevance when `search` is used, and by Store position otherwise. Sort the
    /// results on the client if you need a different order.
    #[deprecated(note = "the API ignores `sortBy`: sort the results on the client")]
    pub sort_by: Option<String>,
    /// Filter by category. Accepts one or more category IDs separated by commas, case-insensitive
    /// (for example `AI_AND_ML`). An Actor must match all of the listed categories. An unknown
    /// category makes the API answer with a `400` error.
    pub category: Option<String>,
    /// Filter by owner username.
    pub username: Option<String>,
    /// Filter by pricing model: `FREE`, `PRICE_PER_DATASET_ITEM` or `PAY_PER_EVENT`.
    pub pricing_model: Option<String>,
    /// By default, results exclude Actors that are not safe to run automatically, such as Actors
    /// from developers who have not passed KYC or full-permission Actors without a large user
    /// base. Set to `true` to bypass this safety filtering and include all Actors.
    pub include_unrunnable_actors: Option<bool>,
    /// If `true`, returns only Actors that allow agentic users. If `false`, returns only Actors
    /// that do not allow agentic users.
    pub allows_agentic_users: Option<bool>,
    /// Response format requested from the API (`full` or `agent`, the API default is `full`).
    pub response_format: Option<String>,
    /// If `true`, each Actor in the results includes an `inputSchema` field with a compact version
    /// of the input schema of its default build (property types and required properties only).
    /// With this option, the API defaults `limit` to `10` and does not accept a larger one.
    pub include_input_schema: Option<bool>,
}

/// Client for the Apify Store.
#[derive(Debug, Clone)]
pub struct StoreCollectionClient {
    ctx: ResourceContext,
}

crate::clients::base::impl_with_timeout!(StoreCollectionClient);

impl StoreCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "store"),
        }
    }

    /// Lists Actors from the Apify Store matching the given options.
    ///
    /// Awaiting once returns a single page; use [`StoreCollectionClient::iterate`] to lazily
    /// iterate across pages.
    pub async fn list(
        &self,
        options: StoreListOptions,
    ) -> ApifyClientResult<PaginationList<ActorStoreListItem>> {
        let params = self.build_params(&options);
        list_resource(&self.ctx, None, &params, Timeout::Medium).await
    }

    /// Lazily iterates all Store Actors matching `options`, fetching pages on demand.
    ///
    /// Returns a [`StoreActorIterator`] whose `next()` method yields one Actor at a time,
    /// transparently fetching subsequent pages.
    ///
    /// `options.limit` caps the *total* number of items yielded across all pages, unlike
    /// [`list`](Self::list) where `limit` is a single page's size. Set the per-page fetch size
    /// with [`with_chunk_size`](crate::ListIterator::with_chunk_size); see
    /// [`ListIterator`] for details.
    ///
    /// With `options.search`, the API returns a single page of at most 100 Actors and ignores
    /// `offset`, so the iterator fetches that one page and stops. `with_chunk_size` has no effect
    /// then, and `options.limit` is sent as given.
    pub fn iterate(&self, options: StoreListOptions) -> StoreActorIterator {
        if options.search.is_none() {
            return list_iterator!(self, options, list);
        }
        let client = self.clone();
        ListIterator::new_single_page(Box::new(move |_offset, _page_limit| {
            let client = client.clone();
            let options = options.clone();
            Box::pin(async move { client.list(options).await })
        }))
    }

    #[allow(deprecated)] // `sort_by` is deprecated, but it is still sent as the caller set it.
    fn build_params(&self, options: &StoreListOptions) -> QueryParams {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_str("search", options.search.clone())
            .add_str("sortBy", options.sort_by.clone())
            .add_str("category", options.category.clone())
            .add_str("username", options.username.clone())
            .add_str("pricingModel", options.pricing_model.clone())
            .add_bool("includeUnrunnableActors", options.include_unrunnable_actors)
            .add_bool("allowsAgenticUsers", options.allows_agentic_users)
            .add_str("responseFormat", options.response_format.clone())
            .add_bool("includeInputSchema", options.include_input_schema);
        params
    }
}
