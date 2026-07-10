//! Client for browsing the Apify Store (`/v2/store`).

use crate::clients::base::{list_resource, ResourceContext};
use crate::clients::pagination::ListIterator;
use crate::common::{PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorStoreListItem;

/// A lazy, page-fetching iterator over Apify Store Actors.
///
/// Returned by [`StoreCollectionClient::iterate`]. This is an alias for the shared
/// [`ListIterator`]; call its `next()` to yield one Actor at a time, fetching further pages
/// from the API transparently until the listing is exhausted.
pub type StoreActorIterator = ListIterator<ActorStoreListItem>;

/// Options for searching the Apify Store.
#[derive(Debug, Default, Clone)]
pub struct StoreListOptions {
    /// Number of items to skip.
    pub offset: Option<i64>,
    /// Maximum number of items to return per page.
    pub limit: Option<i64>,
    /// Full-text search query.
    pub search: Option<String>,
    /// Sort key (e.g. `popularity`, `newest`).
    pub sort_by: Option<String>,
    /// Filter by category.
    pub category: Option<String>,
    /// Filter by owner username.
    pub username: Option<String>,
    /// Filter by pricing model.
    pub pricing_model: Option<String>,
    /// Include Actors that the current user cannot run (e.g. needing a missing integration).
    pub include_unrunnable_actors: Option<bool>,
    /// Filter to Actors that allow agentic (x402/Skyfire) users.
    pub allows_agentic_users: Option<bool>,
    /// Response format requested from the API (e.g. `json`).
    pub response_format: Option<String>,
}

/// Client for the Apify Store.
#[derive(Debug, Clone)]
pub struct StoreCollectionClient {
    ctx: ResourceContext,
}

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
        list_resource(&self.ctx, None, &params).await
    }

    /// Lazily iterates all Store Actors matching `options`, fetching pages on demand.
    ///
    /// Returns a [`StoreActorIterator`] whose `next()` method yields one Actor at a time,
    /// transparently fetching subsequent pages.
    pub fn iterate(&self, options: StoreListOptions) -> StoreActorIterator {
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
            .add_str("responseFormat", options.response_format.clone());
        params
    }
}
