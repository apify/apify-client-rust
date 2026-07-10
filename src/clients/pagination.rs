//! Generic lazy pagination shared by every collection client.
//!
//! The reference JavaScript client returns an `AsyncIterable` from every collection `list()`
//! (via its base `_listPaginated`), so callers can iterate across all pages without manually
//! tracking offsets. Rust cannot return a value that is simultaneously a `Future` and a
//! `Stream`, so the idiomatic equivalent here is a dedicated `iterate()` method on each
//! collection client that returns a [`ListIterator`]. This module implements the paging logic
//! once (the DRY principle the requirements call for) so every client stays a thin wrapper.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;

use crate::common::PaginationList;
use crate::error::ApifyClientResult;

/// A boxed future yielding one page of results. Boxed so [`ListIterator`] can hold a fetcher
/// for any concrete collection client without being generic over its (unnameable) future type.
type PageFuture<T> = Pin<Box<dyn Future<Output = ApifyClientResult<PaginationList<T>>> + Send>>;

/// Fetches the page starting at the given absolute `offset`. Implementations capture a clone of
/// the collection client and the caller's list options, overriding only the offset per page.
type PageFetcher<T> = Box<dyn Fn(i64) -> PageFuture<T> + Send + Sync>;

/// A lazy, page-fetching async iterator over an offset/limit-paginated list endpoint.
///
/// Created by a collection client's `iterate()` method. Each call to [`next`](Self::next)
/// returns the next item, transparently fetching the following page from the API once the
/// local buffer drains, until every item across all pages has been yielded. The caller's
/// per-page `limit` (if any) is honoured as the page size; iteration always walks the full
/// result set regardless.
///
/// # Example
/// ```no_run
/// use apify_client::ApifyClient;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ApifyClient::new("my-api-token");
/// let mut it = client.actors().iterate(Default::default());
/// while let Some(actor) = it.next().await? {
///     println!("{}", actor.id);
/// }
/// # Ok(())
/// # }
/// ```
pub struct ListIterator<T> {
    fetch: PageFetcher<T>,
    buffer: VecDeque<T>,
    /// Absolute offset of the next page to request.
    next_offset: i64,
    /// Set once the listing has been fully consumed.
    exhausted: bool,
}

impl<T> ListIterator<T> {
    /// Builds an iterator that starts at `start_offset` and fetches pages via `fetch`.
    pub(crate) fn new(start_offset: i64, fetch: PageFetcher<T>) -> Self {
        Self {
            fetch,
            buffer: VecDeque::new(),
            next_offset: start_offset,
            exhausted: false,
        }
    }

    /// Returns the next item, or `None` when the listing is exhausted. Fetches another page from
    /// the API when the local buffer is empty.
    pub async fn next(&mut self) -> ApifyClientResult<Option<T>> {
        if let Some(item) = self.buffer.pop_front() {
            return Ok(Some(item));
        }
        if self.exhausted {
            return Ok(None);
        }

        let page = (self.fetch)(self.next_offset).await?;
        let received = page.items.len() as i64;
        if received == 0 {
            self.exhausted = true;
            return Ok(None);
        }
        self.next_offset += received;

        // Decide whether more pages remain. Primary signal is a "short" page: the API returns
        // fewer items than the effective page size it reports (`page.limit`), which only happens
        // on the final page. This is robust even where `total` is unreliable — the dataset-items
        // endpoint, for instance, reports `total = 0`. A non-positive `limit` means the endpoint
        // is not offset-paginated (it returned everything at once), so stop after this page to
        // avoid refetching it forever. `total`, when the endpoint reports it (> 0), is used only
        // as an early stop so a full final page does not cost one extra empty request.
        let effective_limit = page.limit;
        let reached_total = page.total > 0 && self.next_offset >= page.total;
        if effective_limit <= 0 || received < effective_limit || reached_total {
            self.exhausted = true;
        }

        self.buffer.extend(page.items);
        Ok(self.buffer.pop_front())
    }

    /// Eagerly drains the iterator into a single `Vec`, fetching every remaining page.
    ///
    /// Convenience for callers that want all items at once; prefer [`next`](Self::next) to
    /// process items as they stream in without buffering the whole result set.
    pub async fn collect_all(mut self) -> ApifyClientResult<Vec<T>> {
        let mut out = Vec::new();
        while let Some(item) = self.next().await? {
            out.push(item);
        }
        Ok(out)
    }
}
