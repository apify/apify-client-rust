//! Client for browsing the Apify Store (`/v2/store`).

use std::collections::VecDeque;

use futures_util::Stream;

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
    /// Returns a [`Stream`] that yields one Actor at a time (as
    /// `ApifyClientResult<ActorStoreListItem>`), transparently fetching subsequent pages. Pin it
    /// (e.g. with [`futures_util::pin_mut!`]) and drive it with
    /// [`StreamExt::next`](futures_util::StreamExt::next).
    pub fn iterate(
        &self,
        options: StoreListOptions,
    ) -> impl Stream<Item = ApifyClientResult<ActorStoreListItem>> {
        let state = StoreIterState {
            client: self.clone(),
            options,
            buffer: VecDeque::new(),
            next_offset: 0,
            total: None,
            exhausted: false,
        };
        futures_util::stream::try_unfold(state, |mut state| async move {
            if let Some(item) = state.buffer.pop_front() {
                return Ok(Some((item, state)));
            }
            if state.exhausted {
                return Ok(None);
            }

            // Honour a caller-provided starting offset on the first fetch.
            let start_offset = state.options.offset.unwrap_or(0) + state.next_offset;
            let mut page_options = state.options.clone();
            page_options.offset = Some(start_offset);

            let page = state.client.list(page_options).await?;
            if state.total.is_none() {
                state.total = Some(page.total);
            }
            if page.items.is_empty() {
                return Ok(None);
            }

            state.next_offset += page.items.len() as i64;
            // Stop once we have walked past the total number of available items.
            if let Some(total) = state.total {
                if start_offset + page.items.len() as i64 >= total {
                    state.exhausted = true;
                }
            }
            state.buffer.extend(page.items);
            Ok(state.buffer.pop_front().map(|item| (item, state)))
        })
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

/// Internal pagination state driving the [`StoreCollectionClient::iterate`] stream: the
/// current page buffer plus the cursor needed to fetch the next page.
struct StoreIterState {
    client: StoreCollectionClient,
    options: StoreListOptions,
    buffer: VecDeque<ActorStoreListItem>,
    next_offset: i64,
    total: Option<i64>,
    exhausted: bool,
}
