//! Iteration over the default dataset of a run that is still producing items.

use std::collections::VecDeque;
use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::clients::dataset::{DatasetClient, DatasetListItemsOptions};
use crate::clients::run::RunClient;
use crate::error::ApifyClientResult;
use crate::models::ActorRun;
use crate::timeouts::Timeout;

/// Maximum number of dataset rows requested per API call unless
/// [`with_chunk_size`](RunDatasetItemsIterator::with_chunk_size) says otherwise.
const DEFAULT_CHUNK_SIZE: i64 = 1000;
/// The shortest wait between polls, in seconds: the API holds a poll for whole seconds, and a
/// poll that does not wait would repeat at once.
const MIN_POLL_SECS: u64 = 1;
/// How long to wait for the run to finish between polls unless
/// [`with_poll_interval`](RunDatasetItemsIterator::with_poll_interval) says otherwise.
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Where [`RunDatasetItemsIterator`] is in its walk over the dataset.
enum Phase {
    /// Nothing is fetched yet: the run and its dataset still have to be resolved.
    Start,
    /// The run may still be producing items: rows are read up to the dataset's `itemCount`, then
    /// the iterator waits for the run to make progress.
    Live,
    /// The run is finished, but `itemCount` lags behind the pushed items: rows are read a page at
    /// a time until none are left.
    Tail,
    /// Every item was yielded.
    Done,
}

/// One round of the [`Phase::Live`] walk: the rows to read before the next poll.
struct Round {
    /// The dataset's `itemCount` when the round started, capped at the end of the requested range.
    item_count: i64,
    /// Whether the run had reached a terminal status when the round started.
    run_finished: bool,
}

/// A lazy iterator over the items of a run's default dataset that keeps going while the run is
/// still producing them. Created by
/// [`RunClient::iterate_dataset_items`](crate::clients::run::RunClient::iterate_dataset_items).
///
/// While the run has not finished, each round yields the rows below the dataset's `itemCount` and
/// then waits up to the poll interval for the run to finish, so the last rows are read as soon as
/// it does. Each page is requested with a `limit` that ends at `itemCount`, so it covers exactly
/// the rows it asks for, whatever the filters or `unwind` do to the items. `itemCount` lags a few
/// seconds behind the pushed items, so once the run reaches a terminal status, the rows past it
/// are read a page at a time until none are left, and the iterator ends.
///
/// `offset` of the [`DatasetListItemsOptions`] is the number of rows to skip, and `limit` the
/// maximum number of dataset rows to scan: fewer items are yielded when filters drop some, more
/// when `unwind` splits a row into several. On a `last_run()` client, the iterator sticks to the
/// run that its first request resolves to.
///
/// Every request uses the [`Timeout::Long`] tier, unless the run client was given another with
/// `with_timeout`.
pub struct RunDatasetItemsIterator<T> {
    run_client: RunClient,
    options: DatasetListItemsOptions,
    chunk_size: i64,
    poll_interval: Duration,
    /// The run client and dataset client the walk sticks to, resolved by the first request.
    resolved: Option<(RunClient, DatasetClient)>,
    run: Option<ActorRun>,
    /// Index of the next dataset row to read.
    position: i64,
    /// Index of the row the requested range ends before, if it is bounded.
    end: Option<i64>,
    round: Option<Round>,
    phase: Phase,
    buffer: VecDeque<T>,
}

impl<T: DeserializeOwned> RunDatasetItemsIterator<T> {
    pub(crate) fn new(run_client: RunClient, options: DatasetListItemsOptions) -> Self {
        let position = options.offset.unwrap_or(0);
        let end = options
            .limit
            .filter(|&limit| limit > 0)
            .map(|limit| position + limit);
        Self {
            run_client,
            options,
            chunk_size: DEFAULT_CHUNK_SIZE,
            poll_interval: DEFAULT_POLL_INTERVAL,
            resolved: None,
            run: None,
            position,
            end,
            round: None,
            phase: Phase::Start,
            buffer: VecDeque::new(),
        }
    }

    /// Sets the maximum number of dataset rows requested per API call (default 1000). A
    /// non-positive value keeps the default.
    #[must_use]
    pub fn with_chunk_size(mut self, chunk_size: i64) -> Self {
        if chunk_size > 0 {
            self.chunk_size = chunk_size;
        }
        self
    }

    /// Sets how long to wait for the run to finish between polls (default 5 seconds). The API
    /// holds a poll for whole seconds, so the interval is rounded down to them, to at least one.
    #[must_use]
    pub fn with_poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    /// Returns the next item, or `None` when the run is finished and every item was read. Fetches
    /// more rows, or waits for the run to make progress, when the local buffer is empty.
    pub async fn next(&mut self) -> ApifyClientResult<Option<T>> {
        loop {
            if let Some(item) = self.buffer.pop_front() {
                return Ok(Some(item));
            }
            match self.phase {
                Phase::Start => self.resolve().await?,
                Phase::Live => self.read_live_rows().await?,
                Phase::Tail => self.read_tail_rows().await?,
                Phase::Done => return Ok(None),
            }
        }
    }

    /// Eagerly drains the iterator into a single `Vec`, waiting for the run to finish.
    pub async fn collect_all(mut self) -> ApifyClientResult<Vec<T>> {
        let mut items = Vec::new();
        while let Some(item) = self.next().await? {
            items.push(item);
        }
        Ok(items)
    }

    /// Fetches the run, and pins the walk to it and to its default dataset.
    async fn resolve(&mut self) -> ApifyClientResult<()> {
        let timeout = self.run_client.timeout().unwrap_or(Timeout::Long);
        let run_client = self.run_client.clone().with_timeout(timeout);
        let run = run_client.get().await?;
        // A `last_run()` client resolves `runs/last` on each request, so a newer run would swap
        // the dataset mid-iteration.
        let run_client = match &run {
            Some(run) if run.id != self.run_client.id() => {
                self.run_client.pinned_to(&run.id).with_timeout(timeout)
            }
            _ => run_client,
        };
        let dataset = run_client.dataset().with_timeout(timeout);
        self.run = run;
        self.resolved = Some((run_client, dataset));
        self.phase = Phase::Live;
        Ok(())
    }

    fn clients(&self) -> &(RunClient, DatasetClient) {
        self.resolved
            .as_ref()
            .expect("the run is resolved before any row is read")
    }

    /// One step of the [`Phase::Live`] walk: starts a round, reads a page of it, or ends it.
    async fn read_live_rows(&mut self) -> ApifyClientResult<()> {
        let item_count = match &self.round {
            Some(round) => round.item_count,
            None => {
                let run_finished = self.run.as_ref().is_none_or(ActorRun::is_terminal);
                let dataset = self.clients().1.get().await?;
                let mut item_count = dataset.and_then(|dataset| dataset.item_count).unwrap_or(0);
                if let Some(end) = self.end {
                    item_count = item_count.min(end);
                }
                self.round = Some(Round {
                    item_count,
                    run_finished,
                });
                item_count
            }
        };

        if self.position < item_count {
            let page_limit = self.chunk_size.min(item_count - self.position);
            let page = self.list_page(self.position, page_limit).await?;
            self.buffer.extend(page.items);
            self.position += page_limit;
            return Ok(());
        }

        // The round is read: the run is polled for more, unless this was the last round.
        let run_finished = self.round.take().is_some_and(|round| round.run_finished);
        if self.end.is_some_and(|end| self.position >= end) {
            self.phase = Phase::Done;
        } else if run_finished {
            self.phase = Phase::Tail;
        } else {
            let wait_secs =
                i64::try_from(self.poll_interval.as_secs().max(MIN_POLL_SECS)).unwrap_or(i64::MAX);
            self.run = Some(self.clients().0.wait_for_finish(Some(wait_secs)).await?);
        }
        Ok(())
    }

    /// One step of the [`Phase::Tail`] walk: reads a page past `itemCount`.
    async fn read_tail_rows(&mut self) -> ApifyClientResult<()> {
        let page_limit = match self.end {
            Some(end) => self.chunk_size.min(end - self.position),
            None => self.chunk_size,
        };
        let page = self.list_page(self.position, page_limit).await?;
        // Only an empty page marks the end, as filters can shorten a full one. A page that
        // `clean`, `skip_empty` or `unwind` emptied past a lagging `itemCount` reports no scanned
        // rows either, so a plain read checks.
        let is_empty = page.items.is_empty() && page.scanned.unwrap_or(0) == 0;
        self.buffer.extend(page.items);
        if is_empty && !self.filters_can_hide_rows() {
            self.phase = Phase::Done;
            return Ok(());
        }
        if is_empty && !self.rows_are_left().await? {
            self.phase = Phase::Done;
            return Ok(());
        }
        self.position += page_limit;
        if self.end.is_some_and(|end| self.position >= end) {
            self.phase = Phase::Done;
        }
        Ok(())
    }

    /// Whether a server-side filter can empty a page that still has rows behind it.
    fn filters_can_hide_rows(&self) -> bool {
        self.options.clean == Some(true)
            || self.options.skip_empty == Some(true)
            || self
                .options
                .unwind
                .as_ref()
                .is_some_and(|fields| !fields.is_empty())
    }

    /// Whether a plain, unfiltered read finds a row at the current position.
    async fn rows_are_left(&self) -> ApifyClientResult<bool> {
        let probe = DatasetListItemsOptions {
            offset: Some(self.position),
            limit: Some(1),
            ..Default::default()
        };
        let page = self
            .clients()
            .1
            .list_items::<serde_json::Value>(probe)
            .await?;
        Ok(!page.items.is_empty())
    }

    async fn list_page(
        &self,
        offset: i64,
        limit: i64,
    ) -> ApifyClientResult<crate::common::PaginationList<T>> {
        let options = DatasetListItemsOptions {
            offset: Some(offset),
            limit: Some(limit),
            ..self.options.clone()
        };
        self.clients().1.list_items::<T>(options).await
    }
}
