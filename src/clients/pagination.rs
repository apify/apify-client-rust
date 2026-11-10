//! Generic lazy pagination shared by every collection client.
//!
//! A collection client's `iterate()` method returns a [`ListIterator`], which walks all pages
//! without the caller tracking offsets. The paging logic lives here once for every client.

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
/// the listings that are not paginated (an Actor's versions and a version's env vars) use
/// `single_page_iterator!`.
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

/// Generates the body of the `iterate()` method of a collection whose endpoint is not
/// offset-paginated: it has a no-argument `list()` method and returns every item in one response.
/// The resulting [`ListIterator`] calls `list` exactly once and then completes.
macro_rules! single_page_iterator {
    ($self:expr) => {{
        let client = $self.clone();
        $crate::clients::pagination::ListIterator::new_single_page(Box::new(
            move |_offset, _page_limit| {
                let client = client.clone();
                Box::pin(async move { client.list().await })
            },
        ))
    }};
}
pub(crate) use single_page_iterator;

/// Returns the smaller of two optional positive limits, treating a non-positive value as "no
/// limit" (`None`), since the API treats `0` as an absent limit.
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
/// returns the next item and fetches the following page when the buffer is empty, until the
/// listing is exhausted or the caller's cap is reached.
///
/// # `limit` and page size
///
/// The `limit` of the list options is a cap on the total number of items the iterator yields.
/// Leaving it unset (or `0`) iterates the whole listing. The page size is separate: set it with
/// [`with_chunk_size`](Self::with_chunk_size), or each request gets the API's default page size,
/// capped by the remaining `limit`. So `iterate(opts{ limit: 10 })` yields at most 10 items, and
/// `iterate(opts).with_chunk_size(50)` fetches 50 per request and yields everything. Dataset items
/// have no default limit, so
/// [`DatasetClient::iterate_items`](crate::clients::dataset::DatasetClient::iterate_items) without
/// a page size loads the whole dataset, or all `limit` items, in one response.
///
/// **Large caps.** With a cap but no page size, the first page requests `limit == cap`. If the cap
/// is larger than the endpoint's maximum `limit`, also call
/// [`with_chunk_size`](Self::with_chunk_size) with a value at or below that maximum.
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
    /// Page size to request per fetch; `None` = let the API choose its default page size.
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

    /// Sets the page size, the number of items requested per API call. It never changes how many
    /// items the iterator yields in total, which is the caller's `limit`. A non-positive value lets
    /// the API choose its default page size.
    pub fn with_chunk_size(mut self, chunk_size: i64) -> Self {
        self.chunk_size = (chunk_size > 0).then_some(chunk_size);
        self
    }

    /// Returns the next item, or `None` when the listing is exhausted. Fetches another page from
    /// the API when the local buffer is empty, and keeps fetching while the pages come back empty
    /// but scanned rows, as a filter can leave a page with no items in front of rows that match.
    pub async fn next(&mut self) -> ApifyClientResult<Option<T>> {
        loop {
            if let Some(item) = self.buffer.pop_front() {
                return Ok(Some(item));
            }
            if self.exhausted {
                return Ok(None);
            }
            self.fetch_next_page().await?;
        }
    }

    /// Fetches the page at the current offset into the buffer and advances the offset.
    async fn fetch_next_page(&mut self) -> ApifyClientResult<()> {
        // Request the smaller of the items still allowed (`remaining`) and the page size
        // (`chunk_size`); `None` lets the API pick its default page size.
        let page_limit = min_positive_limit(self.remaining, self.chunk_size);
        let page = (self.fetch)(self.next_offset, page_limit).await?;
        let received = page.items.len() as i64;

        // The number of rows this page covered, used to advance the offset and decide when the
        // listing is exhausted: the larger of `count` and the items returned, capped at the page
        // size actually requested. Dataset items report the rows scanned in `count` (see
        // `PaginationList::count`), since a filter can scan more rows than it returns and
        // `unwind` can return more items than rows scanned; the cap keeps an unwound page from
        // advancing past rows the next request would then skip. Other listings report the items
        // returned, except the store, which drops entries after counting them: stepping by
        // `count` keeps the next page from reading them again.
        let scanned = page.count.max(received);
        let scanned = match page_limit {
            Some(limit) if limit > 0 => scanned.min(limit),
            _ => scanned,
        };

        // Enforce the caller's total-item cap exactly, even if the API returns more than the
        // requested page limit.
        let mut items = page.items;
        if let Some(rem) = self.remaining {
            if received > rem {
                items.truncate(rem.max(0) as usize);
            }
        }

        self.next_offset += scanned;
        if let Some(rem) = self.remaining.as_mut() {
            *rem -= scanned;
        }

        // `total` is not used: it grows with the resource, so stopping at it could miss rows added
        // later. A short page is not terminal either, since a filter can shorten a non-final page.
        // Only a page that scanned nothing, or the caller's cap, ends the walk, which costs one
        // extra request at the end.
        if self.single_page {
            // Non-paginated endpoint: everything came back in one response.
            self.exhausted = true;
        } else if scanned == 0 {
            self.exhausted = true;
        } else if matches!(self.remaining, Some(r) if r <= 0) {
            // Reached the caller's total-item cap.
            self.exhausted = true;
        }

        self.buffer.extend(items);
        Ok(())
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
    async fn ignores_reported_total_and_pages_to_an_empty_page() {
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(
            0,
            None,
            slicing_fetcher((0..5).collect(), 2, true, calls.clone()),
        )
        .with_chunk_size(2);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3, 4]);
        // Pages: [0,1] [2,3] [4] [] — the third page is short (1 < 2) but not terminal, and
        // `total` (reported as 5, reached by offset after the third page) is not consulted
        // either, so a fourth, empty request is what actually ends the walk.
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn walks_all_pages_when_total_is_zero() {
        // Emulates an endpoint that does not report a usable total: the trailing empty page ends it.
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
    async fn reported_total_does_not_skip_the_trailing_empty_request() {
        // 4 items, page size 2, total known and an exact multiple of the page size: even though
        // offset reaches `total` right after the second (full) page, that is not consulted, so a
        // third, empty request is still made before the walk ends.
        let calls = Arc::new(AtomicUsize::new(0));
        let iter = ListIterator::new(
            0,
            None,
            slicing_fetcher((0..4).collect(), 2, true, calls.clone()),
        )
        .with_chunk_size(2);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn keeps_paging_past_the_first_pages_total_as_the_listing_grows() {
        // The listing holds 2 items when the first page is read and 4 from then on (e.g. a
        // dataset a still-running Actor keeps pushing to). Each response reports the *current*
        // total, which would wrongly look "complete" at the end of page one if total were
        // consulted; ignoring it lets the walk pick up the rows added afterwards.
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let fetch: PageFetcher<i64> = Box::new(move |offset, limit| {
            let call_number = counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                let total_rows: i64 = if call_number == 0 { 2 } else { 4 };
                let page_size = limit.unwrap_or(2);
                let start = (offset.max(0)).min(total_rows);
                let end = (start + page_size).min(total_rows);
                let items: Vec<i64> = (start..end).collect();
                Ok(PaginationList {
                    total: total_rows,
                    offset,
                    limit: page_size,
                    count: items.len() as i64,
                    desc: false,
                    items,
                })
            })
        });
        let iter = ListIterator::new(0, None, fetch).with_chunk_size(2);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3]);
        // Pages: [0,1] (total looked like 2) [2,3] (total now 4) [] — three requests.
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn advances_by_count_when_it_exceeds_items_returned() {
        // Models a dataset `listItems` filter: each page scans 3 rows (reported in `count`)
        // but a `skip_empty`-style filter leaves only 1 item per page. Advancing by `items.len()`
        // alone would re-request the same rows forever; advancing by `count` moves past
        // them.
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let fetch: PageFetcher<i64> = Box::new(move |offset, _limit| {
            counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                // 9 raw rows in windows of 3, only the first of each window "survives" the filter.
                let window_start = (offset.max(0) / 3) * 3;
                let items = if window_start < 9 {
                    vec![window_start]
                } else {
                    vec![]
                };
                let scanned = if window_start < 9 { 3 } else { 0 };
                Ok(PaginationList {
                    total: 0,
                    offset,
                    limit: 3,
                    count: scanned,
                    desc: false,
                    items,
                })
            })
        });
        let iter = ListIterator::new(0, None, fetch).with_chunk_size(3);
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 3, 6]);
        // Offsets requested: 0, 3, 6, 9 (empty) — four calls, each advancing by the 3 rows counted
        // rather than the 1 surviving item.
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn count_is_capped_at_the_requested_page_size() {
        // A lagging `count` can occasionally overshoot what was asked
        // for (e.g. counting rows added by a concurrent push); the cap keeps the offset from
        // jumping past rows the next request would then never read.
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let fetch: PageFetcher<i64> = Box::new(move |offset, limit| {
            counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                let page_size = limit.unwrap_or(2);
                let start = offset.max(0);
                let end = (start + page_size).min(6);
                let items: Vec<i64> = (start..end).collect();
                // Reports scanning more rows than were actually requested/returned, but only
                // while there is anything left to scan (a genuinely empty page reports 0, same
                // as `items.len()`, so the walk can still terminate).
                let scanned = if items.is_empty() { 0 } else { page_size + 5 };
                Ok(PaginationList {
                    total: 0,
                    offset,
                    limit: page_size,
                    count: scanned,
                    desc: false,
                    items,
                })
            })
        });
        let iter = ListIterator::new(0, None, fetch).with_chunk_size(2);
        // Without the cap this would skip straight past offset 6 and lose items [2..6).
        assert_eq!(iter.collect_all().await.unwrap(), vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(calls.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn non_final_short_page_with_reported_total_does_not_truncate() {
        // Guards the termination logic, NOT filter de-duplication: every page is "short" (it
        // returns fewer items than the page size the API reports), and the endpoint also reports
        // a large `total`. Neither is consulted, so the walk must keep going and yield every item,
        // ending only on the empty page. (This fetcher advances offset by the page size and never
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
                // page is strictly short (received 2 < reported limit 4), and `total` is reported
                // large (100) — neither is consulted, so termination can only come from the empty
                // page, never a short page or a reported total.
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
