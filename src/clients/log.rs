//! Client for accessing a run's or build's log.
//!
//! Logs are accessible at the top level (`/v2/logs/{buildOrRunId}`) and nested under a
//! run or build (`.../log`). The [`LogClient`] supports fetching the whole log as text
//! and streaming it for real-time redirection (the "log redirection" feature).

use futures_util::Stream;

use crate::clients::base::{get_raw, ResourceContext};
use crate::common::QueryParams;
use crate::error::{ApifyClientError, ApifyClientResult};
use crate::http_client::HttpClient;

/// Client for an Actor run or build log.
#[derive(Debug, Clone)]
pub struct LogClient {
    ctx: ResourceContext,
    /// The URL used for streaming; we keep it so streaming can bypass the buffered path.
    stream_url: String,
    token: Option<String>,
    user_agent: String,
}

impl LogClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, resource_path: &str, id: &str) -> Self {
        let ctx = ResourceContext::single(http, base_url, resource_path, id);
        let stream_url = ctx.url(None);
        let (token, user_agent) = ctx.http.stream_credentials();
        Self {
            ctx,
            stream_url,
            token,
            user_agent,
        }
    }

    /// Creates a log client nested under a run or build (path `.../log`).
    pub(crate) fn nested(http: HttpClient, base_url: &str, sub_path: &str) -> Self {
        let ctx = ResourceContext::collection(http, base_url, sub_path);
        let stream_url = ctx.url(None);
        let (token, user_agent) = ctx.http.stream_credentials();
        Self {
            ctx,
            stream_url,
            token,
            user_agent,
        }
    }

    /// Fetches the entire log as a string, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<String>> {
        let response = get_raw(&self.ctx, None, &QueryParams::new()).await?;
        Ok(response.map(|r| String::from_utf8_lossy(&r.body).into_owned()))
    }

    /// Opens a streaming connection to the log, yielding chunks of bytes as they arrive.
    ///
    /// This powers real-time log redirection: callers can forward each chunk to their own
    /// logger/stdout while a run is still in progress. The stream completes when the log
    /// ends (i.e. the run finishes).
    pub async fn stream(
        &self,
    ) -> ApifyClientResult<impl Stream<Item = ApifyClientResult<Vec<u8>>>> {
        // Streaming needs a live connection, so we go through reqwest directly rather than
        // the buffered backend path. The retry policy does not apply to an open stream.
        let client = reqwest::Client::new();
        let mut params = QueryParams::new();
        params.push_raw("stream".to_string(), "1".to_string());
        let url = params.apply_to_url(&self.stream_url);

        let mut builder = client.get(&url).header("User-Agent", &self.user_agent);
        if let Some(token) = &self.token {
            builder = builder.header("Authorization", format!("Bearer {token}"));
        }

        let response = builder.send().await.map_err(ApifyClientError::from)?;
        if !response.status().is_success() {
            return Err(ApifyClientError::InvalidResponse(format!(
                "log stream returned status {}",
                response.status().as_u16()
            )));
        }

        let byte_stream = response.bytes_stream();
        Ok(futures_util::StreamExt::map(byte_stream, |chunk| {
            chunk.map(|b| b.to_vec()).map_err(ApifyClientError::from)
        }))
    }
}
