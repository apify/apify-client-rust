//! Generic lazy pagination shared by every collection client.
//!
//! The reference JavaScript client returns an `AsyncIterable` from every collection `list()`
//! (via its base `_listPaginated`), so callers can iterate across all pages without manually
//! tracking offsets. Rust cannot return a value that is simultaneously a `Future` and a
//! `Stream`, so the idiomatic equivalent here is a dedicated `iterate()` method on each
//! collection client that returns a [`ListIterator`]. The paging logic lives here once so every
//! client stays a thin wrapper over it (don't-repeat-yourself).

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
    /// When `true`, the underlying endpoint is not offset-paginated: the first fetch returns the
    /// whole result set, so the iterator stops after it rather than requesting a second page.
    single_page: bool,
    /// Set once the listing has been fully consumed.
    exhausted: bool,
}

impl<T> ListIterator<T> {
    /// Builds an iterator that starts at `start_offset` and fetches offset-paginated pages via
    /// `fetch`, walking every page until the listing is exhausted.
    pub(crate) fn new(start_offset: i64, fetch: PageFetcher<T>) -> Self {
        Self {
            fetch,
            buffer: VecDeque::new(),
            next_offset: start_offset,
            single_page: false,
            exhausted: false,
        }
    }

    /// Builds an iterator over an endpoint that is **not** offset-paginated (it returns every
    /// item in one response, e.g. an Actor version's environment variables). `fetch` is called
    /// exactly once; the iterator does not attempt a second page, so it does not depend on the
    /// endpoint reporting a page `limit` and cannot refetch the same items.
    pub(crate) fn new_single_page(fetch: PageFetcher<T>) -> Self {
        Self {
            single_page: true,
            ..Self::new(0, fetch)
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

        // Decide whether more pages remain. A single-page endpoint returned everything at once,
        // so stop unconditionally after the first fetch. Otherwise the primary signal is a
        // "short" page: the API returned fewer items than the effective page size it reports
        // (`page.limit`), which only happens on the final page. Short-page detection is robust
        // even where `total` is unreliable — the dataset-items endpoint, for instance, reports
        // `total = 0`. A non-positive reported `limit` is treated as a final page as a safety net.
        // `total`, when the endpoint reports it (> 0), is used only as an early stop so a full
        // final page does not cost one extra empty request.
        if self.single_page {
            self.exhausted = true;
        } else {
            let effective_limit = page.limit;
            let reached_total = page.total > 0 && self.next_offset >= page.total;
            if effective_limit <= 0 || received < effective_limit || reached_total {
                self.exhausted = true;
            }
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

#[cfg(test)]
mod tests {
    use super::{ListIterator, PageFetcher};
    use crate::common::PaginationList;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Builds a fetcher that serves `all` as offset-paginated pages of at most `page_size`
    /// items, reporting `limit = page_size` and `total = all.len()` only when `report_total`.
    /// Counts how many times it is called so tests can assert page-request behaviour.
    fn slicing_fetcher(
        all: Vec<i64>,
        page_size: i64,
        report_total: bool,
        calls: Arc<AtomicUsize>,
    ) -> PageFetcher<i64> {
        let all = Arc::new(all);
        Box::new(move |offset| {
            calls.fetch_add(1, Ordering::SeqCst);
            let all = all.clone();
            Box::pin(async move {
                let start = (offset.max(0) as usize).min(all.len());
                let end = (start + page_size.max(0) as usize).min(all.len());
                let items = all[start..end].to_vec();
                Ok(PaginationList {
                    total: if report_total { all.len() as i64 } else { 0 },
                    offset,
                    limit: page_size,
                    count: items.len() as i64,
                    desc: false,
                    items,
                })
            })
        })
    }

    #[tokio::test]
    async fn walks_all_pages_using_reported_total() {
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(0, slicing_fetcher((0..5).collect(), 2, true, calls.clone()));
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3, 4]);
        // Pages: [0,1] [2,3] [4]. The last page is short, so no extra empty request.
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn walks_all_pages_when_total_is_zero() {
        // Emulates the dataset-items endpoint, which reports total = 0.
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(0, slicing_fetcher((0..5).collect(), 2, false, calls));
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn total_zero_exact_multiple_terminates_on_empty_page() {
        // 4 items, page size 2, no usable total: pages [0,1] [2,3] [] — the empty page ends it.
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(
            0,
            slicing_fetcher((0..4).collect(), 2, false, calls.clone()),
        );
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn reported_total_avoids_extra_request_on_exact_multiple() {
        // 4 items, page size 2, total known: stops after the second full page (no empty fetch).
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(0, slicing_fetcher((0..4).collect(), 2, true, calls.clone()));
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn honours_caller_start_offset() {
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(2, slicing_fetcher((0..5).collect(), 10, true, calls));
        assert_eq!(iter.collect_all().await.unwrap(), vec![2, 3, 4]);
    }

    #[tokio::test]
    async fn single_page_fetches_once_and_stops() {
        // A non-paginated endpoint: the fetcher ignores offset and returns everything every call,
        // reporting limit == count. Multi-page mode would loop forever here; single-page must not.
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let fetch: PageFetcher<i64> = Box::new(move |offset| {
            counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                Ok(PaginationList {
                    total: 0,
                    offset,
                    limit: 3,
                    count: 3,
                    desc: false,
                    items: vec![10, 20, 30],
                })
            })
        });
        let iter = ListIterator::new_single_page(fetch);
        assert_eq!(iter.collect_all().await.unwrap(), vec![10, 20, 30]);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "single-page must fetch exactly once"
        );
    }

    #[tokio::test]
    async fn empty_first_page_yields_nothing() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut iter = ListIterator::new(0, slicing_fetcher(vec![], 5, true, calls));
        assert!(iter.next().await.unwrap().is_none());
    }
}
