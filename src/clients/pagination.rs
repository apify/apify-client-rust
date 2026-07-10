//! Generic lazy pagination shared by every collection client.
//!
//! The reference JavaScript client returns an `AsyncIterable` from every collection `list()`
//! (via its base `_listPaginatedFromCallback`), so callers can iterate across all pages without
//! manually tracking offsets. Rust cannot return a value that is simultaneously a `Future` and a
//! `Stream`, so the idiomatic equivalent here is a dedicated `iterate()` method on each collection
//! client that returns a [`ListIterator`]. The paging logic lives here once so every client stays
//! a thin wrapper over it (don't-repeat-yourself).

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;

use crate::common::PaginationList;
use crate::error::ApifyClientResult;

/// A boxed future yielding one page of results. Boxed so [`ListIterator`] can hold a fetcher
/// for any concrete collection client without being generic over its (unnameable) future type.
type PageFuture<T> = Pin<Box<dyn Future<Output = ApifyClientResult<PaginationList<T>>> + Send>>;

/// Fetches one page: the arguments are the absolute `offset` to start at and the per-page `limit`
/// to request (`None` = let the API pick its default page size). Implementations capture a clone of
/// the collection client and the caller's list options, overriding only the offset and limit per
/// page.
type PageFetcher<T> = Box<dyn Fn(i64, Option<i64>) -> PageFuture<T> + Send + Sync>;

/// Generates the body of a collection client's `iterate()` method for the common shape: an
/// options struct with `offset`/`limit` fields plus a `list(options)` method. It clones the
/// client, reads the caller's start offset and total-item cap from the options, and builds a
/// [`ListIterator`] whose per-page fetcher overrides only `offset`/`limit` before calling
/// `list`. This removes the ~15 lines of identical closure boilerplate that would otherwise be
/// copied into every collection client (the don't-repeat-yourself goal of this module).
///
/// Collections whose listing does not fit this shape build their iterator directly instead:
/// the run listing takes a separate `filter` argument, dataset items use `list_items::<T>`, and
/// an Actor's env vars are non-paginated ([`ListIterator::new_single_page`]).
macro_rules! list_iterator {
    ($self:expr, $options:expr, $list:ident) => {{
        let client = $self.clone();
        let options = $options;
        let start = options.offset.unwrap_or(0);
        let total_limit = options.limit;
        $crate::clients::pagination::ListIterator::new(
            start,
            total_limit,
            Box::new(move |offset, page_limit| {
                let client = client.clone();
                let mut options = options.clone();
                options.offset = Some(offset);
                options.limit = page_limit;
                Box::pin(async move { client.$list(options).await })
            }),
        )
    }};
}
pub(crate) use list_iterator;

/// Returns the smaller of two optional positive limits, treating a non-positive value as "no
/// limit" (`None`). Mirrors the reference client's `minForLimitParam`, where the API treats `0`
/// as an absent limit.
fn min_positive_limit(a: Option<i64>, b: Option<i64>) -> Option<i64> {
    let a = a.filter(|&x| x > 0);
    let b = b.filter(|&x| x > 0);
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

/// A lazy, page-fetching async iterator over an offset/limit-paginated list endpoint.
///
/// Created by a collection client's `iterate()` method. Each call to [`next`](Self::next)
/// returns the next item, transparently fetching the following page from the API once the
/// local buffer drains, until the listing is exhausted (or the caller's total-item cap is hit).
///
/// # `limit` vs. page size
///
/// The caller's `limit` (from the list options passed to `iterate()`) is a **cap on the total
/// number of items the iterator yields**, matching the reference JavaScript client's
/// `_listPaginatedFromCallback`, where `options.limit` bounds the whole async-iterable and a
/// separate `chunkSize` controls page size. Leaving `limit` unset (or `0`) iterates the entire
/// listing. The page size is a distinct concern: set it with [`with_chunk_size`](Self::with_chunk_size);
/// when unset, the API's default page size is used. So `iterate(opts{ limit: 10 })` yields at most
/// 10 items, and `iterate(opts).with_chunk_size(50)` fetches 50 per request while yielding
/// everything.
///
/// **Large caps and the first page.** When a total cap is set but no page size is, the first page
/// requests `limit == cap` (the reference client does the same, via
/// `minForLimitParam(options.limit, options.chunkSize)`). If you set a very large cap — larger
/// than the endpoint's maximum `limit` — also call [`with_chunk_size`](Self::with_chunk_size) with
/// a value at or below that maximum, so the first request stays within the endpoint's accepted
/// range rather than asking for the whole cap up front.
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
    /// Number of items still allowed under the caller's total-item cap; `None` = uncapped.
    /// Decremented by each page's item count as pages are fetched.
    remaining: Option<i64>,
    /// Page size to request per fetch (the reference client's `chunkSize`); `None` = let the API
    /// choose its default page size.
    chunk_size: Option<i64>,
    /// When `true`, the underlying endpoint is not offset-paginated: the first fetch returns the
    /// whole result set, so the iterator stops after it rather than requesting a second page.
    single_page: bool,
    /// Set once the listing has been fully consumed.
    exhausted: bool,
}

impl<T> ListIterator<T> {
    /// Builds an iterator that starts at `start_offset`, yields at most `total_limit` items across
    /// all pages (`None`/`0` = uncapped), and fetches offset-paginated pages via `fetch`.
    pub(crate) fn new(start_offset: i64, total_limit: Option<i64>, fetch: PageFetcher<T>) -> Self {
        let cap = total_limit.filter(|&l| l > 0);
        Self {
            fetch,
            buffer: VecDeque::new(),
            next_offset: start_offset,
            remaining: cap,
            chunk_size: None,
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
            ..Self::new(0, None, fetch)
        }
    }

    /// Sets the page size (items requested per API call) for this iteration — the reference
    /// client's `chunkSize`. This controls only how many items each page fetch requests, never how
    /// many the iterator yields in total (that is the caller's `limit`; see the type docs). A
    /// non-positive value lets the API choose its default page size.
    pub fn with_chunk_size(mut self, chunk_size: i64) -> Self {
        self.chunk_size = (chunk_size > 0).then_some(chunk_size);
        self
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

        // Request the smaller of the items still allowed under the caller's cap (`remaining`) and
        // the configured page size (`chunk_size`); `None` lets the API pick its default page size.
        // On the first page `remaining` is the full cap, matching the reference client's initial
        // `minForLimitParam(options.limit, options.chunkSize)`.
        let page_limit = min_positive_limit(self.remaining, self.chunk_size);
        let page = (self.fetch)(self.next_offset, page_limit).await?;
        let received = page.items.len() as i64;

        // Enforce the caller's total-item cap exactly, even if the API returns more than the
        // requested page limit.
        let mut items = page.items;
        if let Some(rem) = self.remaining {
            if received > rem {
                items.truncate(rem.max(0) as usize);
            }
        }

        self.next_offset += received;
        if let Some(rem) = self.remaining.as_mut() {
            *rem -= received;
        }

        // Decide whether more pages remain.
        if self.single_page {
            // Non-paginated endpoint: everything came back in one response.
            self.exhausted = true;
        } else if received == 0 {
            // Empty page: nothing more to read. This is the primary backstop and matches the
            // reference client, whose loop stops as soon as a page returns no items.
            self.exhausted = true;
        } else if matches!(self.remaining, Some(r) if r <= 0) {
            // Reached the caller's total-item cap.
            self.exhausted = true;
        } else if page.total > 0 {
            // The endpoint reports a usable total, so drive termination by position, like the
            // reference `_listPaginatedFromCallback`. A short page is deliberately NOT treated as
            // terminal here: with dataset item filters (`skip_empty`/`clean`/`skip_hidden`) a full,
            // non-final window can return fewer items than requested while more remain at higher
            // offsets, so short-page detection would silently truncate. The empty-page backstop
            // above ends the walk instead.
            if self.next_offset >= page.total {
                self.exhausted = true;
            }
        } else {
            // No usable total (endpoint reports `total == 0`): fall back to short-page detection —
            // a page shorter than the size the API says it served (`page.limit`) is the last one.
            // A non-positive reported limit means the endpoint is not offset-paginated at all.
            if page.limit <= 0 || received < page.limit {
                self.exhausted = true;
            }
        }

        self.buffer.extend(items);
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

    /// Builds a fetcher that serves `all` as offset-paginated pages, honouring the per-page `limit`
    /// requested by the iterator (falling back to `default_page` when the iterator requests none).
    /// Reports `total = all.len()` only when `report_total`, and echoes the effective page size back
    /// as the response `limit`. Counts how many times it is called so tests can assert page-request
    /// behaviour.
    fn slicing_fetcher(
        all: Vec<i64>,
        default_page: i64,
        report_total: bool,
        calls: Arc<AtomicUsize>,
    ) -> PageFetcher<i64> {
        let all = Arc::new(all);
        Box::new(move |offset, limit| {
            calls.fetch_add(1, Ordering::SeqCst);
            let all = all.clone();
            let page_size = limit.filter(|&l| l > 0).unwrap_or(default_page);
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
        let iter = ListIterator::new(
            0,
            None,
            slicing_fetcher((0..5).collect(), 2, true, calls.clone()),
        )
        .with_chunk_size(2);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3, 4]);
        // Pages: [0,1] [2,3] [4]. next_offset reaches total (5) on the third page, so no extra
        // empty request.
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn walks_all_pages_when_total_is_zero() {
        // Emulates an endpoint that does not report a usable total: short-page detection ends it.
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(0, None, slicing_fetcher((0..5).collect(), 2, false, calls))
            .with_chunk_size(2);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn total_zero_exact_multiple_terminates_on_empty_page() {
        // 4 items, page size 2, no usable total: pages [0,1] [2,3] [] — the empty page ends it.
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(
            0,
            None,
            slicing_fetcher((0..4).collect(), 2, false, calls.clone()),
        )
        .with_chunk_size(2);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn reported_total_avoids_extra_request_on_exact_multiple() {
        // 4 items, page size 2, total known: stops after the second full page (no empty fetch).
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(
            0,
            None,
            slicing_fetcher((0..4).collect(), 2, true, calls.clone()),
        )
        .with_chunk_size(2);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn non_final_short_page_with_reported_total_does_not_truncate() {
        // Guards the termination logic, NOT filter de-duplication: every page is "short" (it
        // returns fewer items than the page size the API reports) while the endpoint reports a
        // large total. Total-driven termination must keep going and yield every item, ending
        // only on the empty page. (This fetcher advances offset by the page size and never
        // overlaps windows, so it does not model the sparse-window duplicate behaviour
        // documented on `iterate_items`; it only proves a short page is not treated as terminal.)
        let all: Vec<i64> = (0..6).collect();
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let all = Arc::new(all);
        let fetch: PageFetcher<i64> = Box::new(move |offset, _limit| {
            counter.fetch_add(1, Ordering::SeqCst);
            let all = all.clone();
            Box::pin(async move {
                // Serve at most 2 items per page but report a page limit of 4, so every non-final
                // page is strictly short (received 2 < reported limit 4). `total` is reported large
                // (100) so termination can only come from the empty page, never a short page.
                let start = (offset.max(0) as usize).min(all.len());
                let end = (start + 2).min(all.len());
                let items = all[start..end].to_vec();
                Ok(PaginationList {
                    total: 100,
                    offset,
                    limit: 4,
                    count: items.len() as i64,
                    desc: false,
                    items,
                })
            })
        });
        let iter = ListIterator::new(0, None, fetch).with_chunk_size(4);
        let got = iter.collect_all().await.unwrap();
        // Every item must be yielded despite each page being short.
        assert_eq!(got, vec![0, 1, 2, 3, 4, 5]);
        // Pages: [0,1] [2,3] [4,5] [] — four calls, none terminated early by short-page detection.
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn total_limit_caps_items_yielded() {
        // `limit` is a total-item cap: with 100 items available, iterate should yield exactly 3.
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(
            0,
            Some(3),
            slicing_fetcher((0..100).collect(), 1000, true, calls.clone()),
        );
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2]);
        // Requesting limit=3 up front means a single page suffices.
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn total_limit_with_smaller_chunk_size_pages_and_caps() {
        // limit=5 total cap, chunk_size=2 page size: pages of 2 until 5 items are yielded.
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(
            0,
            Some(5),
            slicing_fetcher((0..100).collect(), 1000, true, calls.clone()),
        )
        .with_chunk_size(2);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3, 4]);
        // Pages: [0,1] [2,3] [4] — the last page is trimmed to the remaining budget of 1.
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn cap_truncates_page_that_exceeds_remaining_budget() {
        // Exercises the cap-truncation branch (`received > rem`): the endpoint ignores the
        // requested page limit and returns MORE items than the caller's total cap allows. The
        // iterator must trim the over-long page to the remaining budget and yield exactly `limit`
        // items, fetching only once.
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let fetch: PageFetcher<i64> = Box::new(move |offset, _limit| {
            counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                // Always return 5 items regardless of the requested limit.
                Ok(PaginationList {
                    total: 100,
                    offset,
                    limit: 5,
                    count: 5,
                    desc: false,
                    items: vec![0, 1, 2, 3, 4],
                })
            })
        });
        // Total cap of 3, but the page delivers 5 → must be truncated to [0, 1, 2].
        let iter = ListIterator::new(0, Some(3), fetch);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2]);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "cap reached on the first (over-long) page, so no second fetch"
        );
    }

    #[tokio::test]
    async fn honours_caller_start_offset() {
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(2, None, slicing_fetcher((0..5).collect(), 10, true, calls));
        assert_eq!(iter.collect_all().await.unwrap(), vec![2, 3, 4]);
    }

    #[tokio::test]
    async fn single_page_fetches_once_and_stops() {
        // A non-paginated endpoint: the fetcher ignores offset and returns everything every call,
        // reporting limit == count. Multi-page mode would loop forever here; single-page must not.
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let fetch: PageFetcher<i64> = Box::new(move |offset, _limit| {
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
        let mut iter = ListIterator::new(0, None, slicing_fetcher(vec![], 5, true, calls));
        assert!(iter.next().await.unwrap().is_none());
    }
}
