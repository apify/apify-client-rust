//! Client for browsing the Apify Store (`/v2/store`).

use crate::clients::base::{list_resource, ResourceContext};
use crate::common::{PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::ActorStoreListItem;

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
    /// Returns a [`StoreActorIterator`] whose [`next`](StoreActorIterator::next) method
    /// yields one Actor at a time, transparently fetching subsequent pages.
    pub fn iterate(&self, options: StoreListOptions) -> StoreActorIterator {
        StoreActorIterator {
            client: self.clone(),
            options,
            buffer: std::collections::VecDeque::new(),
            next_offset: 0,
            total: None,
            exhausted: false,
        }
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

/// A lazy, page-fetching iterator over Apify Store Actors.
///
/// Created by [`StoreCollectionClient::iterate`]. Each call to [`next`](Self::next)
/// returns the next Actor, fetching another page from the API when the local buffer is
/// exhausted, until all matching Actors have been yielded.
pub struct StoreActorIterator {
    client: StoreCollectionClient,
    options: StoreListOptions,
    buffer: std::collections::VecDeque<ActorStoreListItem>,
    next_offset: i64,
    total: Option<i64>,
    exhausted: bool,
}

impl StoreActorIterator {
    /// Returns the next Store Actor, or `None` when the listing is exhausted.
    pub async fn next(&mut self) -> ApifyClientResult<Option<ActorStoreListItem>> {
        if let Some(item) = self.buffer.pop_front() {
            return Ok(Some(item));
        }
        if self.exhausted {
            return Ok(None);
        }

        // Honour a caller-provided starting offset on the first fetch.
        let start_offset = self.options.offset.unwrap_or(0) + self.next_offset;
        let mut page_options = self.options.clone();
        page_options.offset = Some(start_offset);

        let page = self.client.list(page_options).await?;
        if self.total.is_none() {
            self.total = Some(page.total);
        }
        if page.items.is_empty() {
            self.exhausted = true;
            return Ok(None);
        }

        self.next_offset += page.items.len() as i64;
        // Stop once we have walked past the total number of available items.
        if let Some(total) = self.total {
            if start_offset + page.items.len() as i64 >= total {
                self.exhausted = true;
            }
        }
        self.buffer.extend(page.items);
        Ok(self.buffer.pop_front())
    }
}
