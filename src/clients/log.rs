//! Client for accessing a run's or build's log.
//!
//! Logs are accessible at the top level (`/v2/logs/{buildOrRunId}`) and nested under a
//! run or build (`.../log`). The [`LogClient`] supports fetching the whole log as text
//! and streaming it for real-time redirection (the "log redirection" feature).

use futures_util::Stream;

use crate::clients::base::{
    catch_not_found_for, get_byte_stream, get_raw_for_resource, ResourceContext,
};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::timeouts::Timeout;

/// Options for retrieving or streaming a log ([`LogClient::get_with_options`] /
/// [`LogClient::stream_with_options`]).
///
/// Covers the spec's optional `raw` and `download` query parameters on the log endpoints
/// (`GET /v2/logs/{buildOrRunId}`, `GET /v2/actor-runs/{runId}/log`, and the last-run log
/// variants), matching the reference client's `LogOptions`.
#[derive(Debug, Default, Clone)]
pub struct LogOptions {
    /// If `true`, return the raw log content without any server-side processing (e.g. without
    /// the per-line timestamps the API adds by default). Defaults to `false` when unset.
    pub raw: Option<bool>,
    /// If `true`, the API asks a web browser to download the log as a file rather than open it in
    /// a tab (it adds a `Content-Disposition: attachment` header). Defaults to `false` when unset.
    pub download: Option<bool>,
}

/// Client for an Actor run or build log.
#[derive(Debug, Clone)]
pub struct LogClient {
    ctx: ResourceContext,
}

crate::clients::base::impl_with_timeout!(LogClient);

impl LogClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, resource_path: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, resource_path, id),
        }
    }

    /// Creates a log client nested under a run or build (path `.../log`).
    pub(crate) fn nested(http: HttpClient, base_url: &str, sub_path: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, sub_path),
        }
    }

    /// Fetches the entire log as a string, or `None` if it does not exist. A log client chained
    /// from a run or build, such as `run.log()`, has no ID of its own, so it returns the
    /// `404` as an error: the response cannot tell a missing run from a missing log.
    ///
    /// Uses the default (processed) log format. To request the raw log, use
    /// [`LogClient::get_with_options`]. To get the log as raw bytes instead of a `String`
    /// (e.g. if it is not valid UTF-8), use [`LogClient::get_bytes`].
    pub async fn get(&self) -> ApifyClientResult<Option<String>> {
        self.get_with_options(LogOptions::default()).await
    }

    /// Fetches the entire log as a string, or `None` if it does not exist, applying the given
    /// [`LogOptions`] (e.g. [`LogOptions::raw`]).
    pub async fn get_with_options(&self, options: LogOptions) -> ApifyClientResult<Option<String>> {
        Ok(self
            .get_bytes_with_options(options)
            .await?
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
    }

    /// Fetches the entire log as raw bytes, or `None` if it does not exist.
    ///
    /// The counterpart of [`LogClient::get`] for a log that is not valid UTF-8, or when the
    /// caller wants to avoid the lossy-UTF-8 conversion `get()` applies. Uses the default
    /// (processed) log format; see [`LogClient::get_bytes_with_options`] for the raw format.
    pub async fn get_bytes(&self) -> ApifyClientResult<Option<Vec<u8>>> {
        self.get_bytes_with_options(LogOptions::default()).await
    }

    /// Fetches the entire log as raw bytes, or `None` if it does not exist, applying the given
    /// [`LogOptions`] (e.g. [`LogOptions::raw`]). The byte-returning counterpart of
    /// [`LogClient::get_with_options`].
    pub async fn get_bytes_with_options(
        &self,
        options: LogOptions,
    ) -> ApifyClientResult<Option<Vec<u8>>> {
        let mut params = QueryParams::new();
        params
            .add_bool("raw", options.raw)
            .add_bool("download", options.download);
        let response = get_raw_for_resource(&self.ctx, None, &params, Timeout::Long).await?;
        Ok(response.map(|r| r.body))
    }

    /// Opens a streaming connection to the log, yielding chunks of bytes as they arrive, or
    /// `None` if the log does not exist.
    ///
    /// This powers real-time log redirection: callers can forward each chunk to their own
    /// logger/stdout while a run is still in progress. The stream completes when the log ends
    /// (i.e. the run finishes) — as long as a new chunk arrives at least every 6 minutes (the
    /// [`Timeout::Long`] tier, see [`with_timeout`](Self::with_timeout)); a run that legitimately
    /// logs nothing for that long errors with
    /// [`ApifyClientError::Timeout`](crate::error::ApifyClientError::Timeout) instead.
    ///
    /// A log client chained from a run or build, such as `run.log()`, has no ID of its own, so it
    /// returns the `404` as an error: the response cannot tell a missing run from a missing log.
    ///
    /// Uses the default (processed) log format. To stream the raw log, use
    /// [`LogClient::stream_with_options`].
    pub async fn stream(
        &self,
    ) -> ApifyClientResult<Option<impl Stream<Item = ApifyClientResult<Vec<u8>>>>> {
        self.stream_with_options(LogOptions::default()).await
    }

    /// Opens a streaming connection to the log applying the given [`LogOptions`], yielding
    /// chunks of bytes as they arrive, or `None` if the log does not exist.
    ///
    /// Like [`LogClient::stream`], but lets the caller request the raw log via
    /// [`LogOptions::raw`] (as the reference client's log redirection does, which streams
    /// `{ raw: true }`).
    pub async fn stream_with_options(
        &self,
        options: LogOptions,
    ) -> ApifyClientResult<Option<impl Stream<Item = ApifyClientResult<Vec<u8>>>>> {
        catch_not_found_for(&self.ctx, self.stream_required(options).await)
    }

    /// Like [`stream_with_options`](Self::stream_with_options), but a `404` is an error whatever
    /// the client's path. Used where the caller already knows the run exists.
    pub(crate) async fn stream_required(
        &self,
        options: LogOptions,
    ) -> ApifyClientResult<impl Stream<Item = ApifyClientResult<Vec<u8>>>> {
        // Goes through `get_byte_stream` (the configured `HttpBackend`'s streaming path) instead
        // of `HttpClient::call`, whose buffered backend fully reads a response before returning
        // it. The retry policy does not apply to an open stream.
        let mut params = QueryParams::new();
        params.push_raw("stream".to_string(), "1".to_string());
        params
            .add_bool("raw", options.raw)
            .add_bool("download", options.download);
        let url = params.apply_to_url(&self.ctx.url(None));
        get_byte_stream(&self.ctx, url, Timeout::Long).await
    }
}
