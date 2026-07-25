//! Offline unit tests for the HTTP layer (retries, backoff, error parsing, 404 handling).
//!
//! These use a mock [`HttpBackend`] and require no network access or `APIFY_TOKEN`, so the
//! most bug-prone logic is always exercised in CI.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use apify_client::http_client::{HttpBackend, HttpMethod, HttpRequest, HttpResponse};
use apify_client::models::RequestQueueRequest;
use apify_client::{
    ApifyClient, ApifyClientError, BatchAddRequestsOptions, LastRunOptions, RequestCompression,
    RunChargeOptions,
};
use async_trait::async_trait;

/// A scripted backend that returns a queued sequence of responses and counts calls.
#[derive(Debug)]
struct MockBackend {
    responses: Mutex<Vec<MockOutcome>>,
    calls: AtomicUsize,
    last_url: Mutex<Option<String>>,
    /// Every request URL in call order, so tests can assert per-page pagination wiring.
    urls: Mutex<Vec<String>>,
    last_headers: Mutex<std::collections::HashMap<String, String>>,
    last_body: Mutex<Option<Vec<u8>>>,
    last_method: Mutex<Option<HttpMethod>>,
}

#[derive(Debug, Clone)]
enum MockOutcome {
    Status(u16, Vec<u8>),
    NetworkError,
}

impl MockBackend {
    fn new(responses: Vec<MockOutcome>) -> Arc<Self> {
        Arc::new(Self {
            responses: Mutex::new(responses),
            calls: AtomicUsize::new(0),
            last_url: Mutex::new(None),
            urls: Mutex::new(Vec::new()),
            last_headers: Mutex::new(std::collections::HashMap::new()),
            last_body: Mutex::new(None),
            last_method: Mutex::new(None),
        })
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn last_url(&self) -> Option<String> {
        self.last_url.lock().unwrap().clone()
    }

    /// All request URLs in call order.
    fn urls(&self) -> Vec<String> {
        self.urls.lock().unwrap().clone()
    }

    /// Case-insensitive lookup of the last request's header value.
    fn last_header(&self, name: &str) -> Option<String> {
        let headers = self.last_headers.lock().unwrap();
        headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    }

    fn last_body(&self) -> Option<Vec<u8>> {
        self.last_body.lock().unwrap().clone()
    }

    fn last_method(&self) -> Option<HttpMethod> {
        *self.last_method.lock().unwrap()
    }
}

#[async_trait]
impl HttpBackend for MockBackend {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, ApifyClientError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_url.lock().unwrap() = Some(request.url.clone());
        self.urls.lock().unwrap().push(request.url.clone());
        *self.last_headers.lock().unwrap() = request.headers.clone();
        *self.last_body.lock().unwrap() = request.body.clone();
        *self.last_method.lock().unwrap() = Some(request.method);
        let mut queue = self.responses.lock().unwrap();
        let outcome = if queue.len() > 1 {
            queue.remove(0)
        } else {
            // Repeat the last scripted outcome for any further calls.
            queue
                .first()
                .cloned()
                .expect("at least one scripted response")
        };
        match outcome {
            MockOutcome::Status(status, body) => Ok(HttpResponse {
                status,
                headers: Default::default(),
                body,
            }),
            MockOutcome::NetworkError => Err(ApifyClientError::Http("boom".to_string())),
        }
    }
}

fn client_with(backend: Arc<MockBackend>, max_retries: u32) -> ApifyClient {
    ApifyClient::builder()
        .token("test-token")
        .max_retries(max_retries)
        // Keep backoff tiny so tests run fast.
        .min_delay_between_retries(Duration::from_millis(1))
        .http_backend(backend)
        .build()
}

/// Builds a client with an explicit request-compression algorithm and no retries.
fn client_with_compression(
    backend: Arc<MockBackend>,
    compression: RequestCompression,
) -> ApifyClient {
    ApifyClient::builder()
        .token("test-token")
        .max_retries(0)
        .min_delay_between_retries(Duration::from_millis(1))
        .request_compression(compression)
        .http_backend(backend)
        .build()
}

/// A successful GET unwraps the `data` envelope and makes exactly one call.
#[tokio::test]
async fn success_single_call() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        200,
        br#"{"data":{"id":"abc","username":"u"}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 3);
    let user = client.me().get().await.expect("ok").expect("some");
    assert_eq!(user.id, "abc");
    assert_eq!(backend.call_count(), 1);
}

/// A `429` is retried up to `max_retries + 1` total attempts, then surfaces the API error.
#[tokio::test]
async fn rate_limit_is_retried() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        429,
        br#"{"error":{"type":"rate-limit-exceeded","message":"slow down"}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 2);
    let err = client.me().get().await.expect_err("should fail");
    assert_eq!(backend.call_count(), 3, "1 initial + 2 retries");
    let api = err.as_api_error().expect("api error");
    assert_eq!(api.status_code, 429);
    assert_eq!(api.error_type.as_deref(), Some("rate-limit-exceeded"));
    assert_eq!(api.attempt, 3);
}

/// A `5xx` is retried.
#[tokio::test]
async fn server_error_is_retried() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        503,
        br#"{"error":{"type":"server-error","message":"oops"}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 1);
    let _ = client.me().get().await.expect_err("should fail");
    assert_eq!(backend.call_count(), 2, "1 initial + 1 retry");
}

/// A non-429 `4xx` is NOT retried (caller error).
#[tokio::test]
async fn client_error_not_retried() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        400,
        br#"{"error":{"type":"invalid-request","message":"bad"}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 5);
    let err = client.me().get().await.expect_err("should fail");
    assert_eq!(backend.call_count(), 1, "4xx must not be retried");
    assert_eq!(err.status_code(), Some(400));
}

/// Network errors are retried, then surface.
#[tokio::test]
async fn network_error_is_retried() {
    let backend = MockBackend::new(vec![MockOutcome::NetworkError]);
    let client = client_with(backend.clone(), 3);
    let err = client.me().get().await.expect_err("should fail");
    assert_eq!(backend.call_count(), 4, "1 initial + 3 retries");
    assert!(matches!(err, ApifyClientError::Http(_)));
}

/// Retries eventually succeed when a later attempt returns 200.
#[tokio::test]
async fn retry_then_success() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(500, b"{}".to_vec()),
        MockOutcome::Status(500, b"{}".to_vec()),
        MockOutcome::Status(200, br#"{"data":{"id":"ok"}}"#.to_vec()),
    ]);
    let client = client_with(backend.clone(), 5);
    let user = client.me().get().await.expect("ok").expect("some");
    assert_eq!(user.id, "ok");
    assert_eq!(backend.call_count(), 3);
}

/// A `404 record-not-found` on a `get` maps to `Ok(None)`, not an error.
#[tokio::test]
async fn not_found_maps_to_none() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        404,
        br#"{"error":{"type":"record-not-found","message":"missing"}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 3);
    let actor = client.actor("nope").get().await.expect("ok");
    assert!(actor.is_none(), "404 record-not-found should be Ok(None)");
    assert_eq!(backend.call_count(), 1, "404 is not retried");
}

/// The error body is parsed into the structured `ApiError` fields.
#[tokio::test]
async fn error_body_is_parsed() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        400,
        br#"{"error":{"type":"invalid-value","message":"Comments required"}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 0);
    let err = client.me().get().await.expect_err("should fail");
    let api = err.as_api_error().expect("api error");
    assert_eq!(api.status_code, 400);
    assert_eq!(api.error_type.as_deref(), Some("invalid-value"));
    assert_eq!(api.message, "Comments required");
    assert_eq!(api.http_method.as_deref(), Some("GET"));
    assert!(api.path.as_deref().unwrap().contains("/users/me"));
}

/// `max_retries = 0` means exactly one attempt.
#[tokio::test]
async fn zero_retries_single_attempt() {
    let backend = MockBackend::new(vec![MockOutcome::Status(500, b"{}".to_vec())]);
    let client = client_with(backend.clone(), 0);
    let _ = client.me().get().await.expect_err("should fail");
    assert_eq!(backend.call_count(), 1);
}

/// Regression guard for the validate-input envelope-skip fix: the `validate-input` endpoint
/// returns a **bare** `{"valid":...}` object with no `{"data":...}` envelope, so it must NOT be
/// routed through `parse_data_envelope` (which would fail with `missing field 'data'`). This is
/// token-free, so a future refactor that re-introduces envelope unwrapping is caught even in a
/// run without `APIFY_TOKEN`.
#[tokio::test]
async fn validate_input_does_not_unwrap_data_envelope() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        200,
        br#"{"valid":true}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 3);
    let result = client
        .actor("me~some-actor")
        .validate_input(&serde_json::json!({}))
        .await
        .expect("validate_input should parse a bare {valid} body");
    assert_eq!(
        result.get("valid").and_then(|v| v.as_bool()),
        Some(true),
        "the bare body must be returned verbatim, not unwrapped from a `data` envelope"
    );
    assert_eq!(backend.call_count(), 1);
}

/// `validate_input_for_build` sends the spec's optional `build` query parameter, and the
/// no-arg `validate_input` omits it.
#[tokio::test]
async fn validate_input_sends_build_query_param() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        200,
        br#"{"valid":true}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 0);

    // With a build tag -> `build=latest` present in the URL.
    client
        .actor("me~some-actor")
        .validate_input_for_build(&serde_json::json!({}), Some("latest"))
        .await
        .expect("ok");
    let url = backend.last_url().expect("a request was sent");
    assert!(
        url.contains("/validate-input") && url.contains("build=latest"),
        "expected build=latest in {url}"
    );

    // Without a build -> no `build` param.
    client
        .actor("me~some-actor")
        .validate_input(&serde_json::json!({}))
        .await
        .expect("ok");
    let url = backend.last_url().expect("a request was sent");
    assert!(
        !url.contains("build="),
        "default validate_input must not send a build param, got {url}"
    );
}

/// `LogClient::get_with_options` sends the spec's optional `raw` query parameter, and the
/// no-arg `get` omits it.
#[tokio::test]
async fn log_get_sends_raw_query_param() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"some log output".to_vec())]);
    let client = client_with(backend.clone(), 0);

    // With raw -> `raw=1` present in the run log URL.
    client
        .run("some-run-id")
        .log()
        .get_with_options(apify_client::LogOptions { raw: Some(true) })
        .await
        .expect("ok");
    let url = backend.last_url().expect("a request was sent");
    assert!(
        url.contains("/log") && url.contains("raw=1"),
        "expected raw=1 on the run log request, got {url}"
    );

    // Without options -> no `raw` param.
    client.run("some-run-id").log().get().await.expect("ok");
    let url = backend.last_url().expect("a request was sent");
    assert!(
        !url.contains("raw="),
        "default log get must not send a raw param, got {url}"
    );
}

/// A request body at or above the 1024-byte threshold is brotli-compressed: the backend sees a
/// `Content-Encoding: br` header and a body that is smaller than (and different from) the original
/// and decodes back to the original bytes.
#[tokio::test]
async fn large_request_body_is_brotli_compressed() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"".to_vec())]);
    let client = client_with(backend.clone(), 0);

    // A highly compressible 4 KiB payload, comfortably over the 1024-byte threshold.
    let original = vec![b'a'; 4096];
    client
        .key_value_store("me~store")
        .set_record_raw("key", original.clone(), "text/plain")
        .await
        .expect("ok");

    assert_eq!(
        backend.last_header("Content-Encoding").as_deref(),
        Some("br"),
        "large bodies must be sent with Content-Encoding: br"
    );
    let sent = backend.last_body().expect("a body was sent");
    assert!(
        sent.len() < original.len(),
        "compressed body ({}) must be smaller than the original ({})",
        sent.len(),
        original.len()
    );
    assert_ne!(sent, original, "the sent body must actually be encoded");

    // The sent bytes must decode back to the original via brotli.
    let mut decompressed = Vec::new();
    let mut decoder = brotli::DecompressorWriter::new(&mut decompressed, 4096);
    std::io::Write::write_all(&mut decoder, &sent).expect("decode");
    drop(decoder);
    assert_eq!(decompressed, original, "brotli round-trip must be lossless");
}

/// With the gzip encoding selected, a large request body is gzip-compressed: the backend sees a
/// `Content-Encoding: gzip` header and a body that is smaller than (and different from) the original
/// and decodes back to the original bytes. This exercises the gzip path end-to-end.
#[tokio::test]
async fn large_request_body_is_gzip_compressed_when_selected() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"".to_vec())]);
    let client = client_with_compression(backend.clone(), RequestCompression::Gzip);

    // A highly compressible 4 KiB payload, comfortably over the 1024-byte threshold.
    let original = vec![b'a'; 4096];
    client
        .key_value_store("me~store")
        .set_record_raw("key", original.clone(), "text/plain")
        .await
        .expect("ok");

    assert_eq!(
        backend.last_header("Content-Encoding").as_deref(),
        Some("gzip"),
        "large bodies must be sent with Content-Encoding: gzip when gzip is selected"
    );
    let sent = backend.last_body().expect("a body was sent");
    assert!(
        sent.len() < original.len(),
        "compressed body ({}) must be smaller than the original ({})",
        sent.len(),
        original.len()
    );
    assert_ne!(sent, original, "the sent body must actually be encoded");

    // The sent bytes must decode back to the original via gzip.
    let mut decoder = flate2::read::GzDecoder::new(&sent[..]);
    let mut decompressed = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut decompressed).expect("decode");
    assert_eq!(decompressed, original, "gzip round-trip must be lossless");
}

/// When `request_compression()` is never called on the builder, large bodies are brotli-compressed:
/// the builder's default encoding is brotli, so the backend sees `Content-Encoding: br`. This
/// exercises the builder default via the public builder path (not the `Default` derive), guarding
/// against the default silently changing.
#[tokio::test]
async fn builder_default_request_compression_is_brotli() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"".to_vec())]);
    // `client_with` builds a client without ever calling `.request_compression(...)`.
    let client = client_with(backend.clone(), 0);

    let original = vec![b'a'; 4096];
    client
        .key_value_store("me~store")
        .set_record_raw("key", original.clone(), "text/plain")
        .await
        .expect("ok");

    assert_eq!(
        backend.last_header("Content-Encoding").as_deref(),
        Some("br"),
        "the default compression must be brotli"
    );
}

/// A request body below the 1024-byte threshold is sent verbatim, with no `Content-Encoding`.
#[tokio::test]
async fn small_request_body_is_not_compressed() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"".to_vec())]);
    let client = client_with(backend.clone(), 0);

    let original = vec![b'a'; 100];
    client
        .key_value_store("me~store")
        .set_record_raw("key", original.clone(), "text/plain")
        .await
        .expect("ok");

    assert!(
        backend.last_header("Content-Encoding").is_none(),
        "small bodies must not be compressed"
    );
    assert_eq!(
        backend.last_body().expect("a body was sent"),
        original,
        "small bodies must be sent verbatim"
    );
}

/// The status-only `last_run(status)` and the options-based `last_run_with_options` thread the
/// optional `status` and `origin` filters into the last-run request as query parameters, matching
/// the JS reference's `lastRun({ status, origin })`. Leaving a filter as `None` omits it.
#[tokio::test]
async fn last_run_sends_status_and_origin_query_params() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        200,
        br#"{"data":{"id":"run1","status":"SUCCEEDED"}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 0);

    // The non-breaking status-only convenience sends just `status`.
    client
        .actor("me~some-actor")
        .last_run(Some("SUCCEEDED"))
        .get()
        .await
        .expect("ok");
    let url = backend.last_url().expect("a request was sent");
    assert!(
        url.contains("/runs/last") && url.contains("status=SUCCEEDED"),
        "expected status=SUCCEEDED on the actor last-run request, got {url}"
    );
    assert!(
        !url.contains("origin="),
        "last_run(Some(..)) must not send an origin param, got {url}"
    );

    // Both filters set via options on the actor last run -> both query params present.
    client
        .actor("me~some-actor")
        .last_run_with_options(LastRunOptions {
            status: Some("SUCCEEDED".to_owned()),
            origin: Some("API".to_owned()),
        })
        .get()
        .await
        .expect("ok");
    let url = backend.last_url().expect("a request was sent");
    assert!(
        url.contains("/runs/last")
            && url.contains("status=SUCCEEDED")
            && url.contains("origin=API"),
        "expected status=SUCCEEDED and origin=API on the actor last-run request, got {url}"
    );

    // Only `origin` set via options on the task last run -> origin present, status absent.
    client
        .task("me~some-task")
        .last_run_with_options(LastRunOptions {
            status: None,
            origin: Some("SCHEDULER".to_owned()),
        })
        .get()
        .await
        .expect("ok");
    let url = backend.last_url().expect("a request was sent");
    assert!(
        url.contains("/runs/last") && url.contains("origin=SCHEDULER"),
        "expected origin=SCHEDULER on the task last-run request, got {url}"
    );
    assert!(
        !url.contains("status="),
        "task last_run_with_options without status must not send a status param, got {url}"
    );

    // Neither filter set -> no `status`/`origin` params.
    client
        .actor("me~some-actor")
        .last_run(None)
        .get()
        .await
        .expect("ok");
    let url = backend.last_url().expect("a request was sent");
    assert!(
        !url.contains("status=") && !url.contains("origin="),
        "default last_run must not send status/origin params, got {url}"
    );
}

/// `iterate_items` must keep paging even when the dataset-items response omits the
/// `X-Apify-Pagination-Total` header. With the header absent, `list_items` reports `total = 0`
/// ("unknown"), so the iterator falls through to the short-page/empty-page backstop and walks
/// every page. The `MockBackend` returns no headers, reproducing the missing-total case. Before
/// the fix, `list_items` fell back to `total = count`, which the iterator read as a completed
/// total after the first full page (`next_offset == total`) and silently dropped every later
/// item — this test scripts three data pages then an empty one and asserts all items are yielded.
#[tokio::test]
async fn iterate_items_without_total_header_walks_all_pages() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(200, br#"[{"i":0},{"i":1}]"#.to_vec()),
        MockOutcome::Status(200, br#"[{"i":2},{"i":3}]"#.to_vec()),
        MockOutcome::Status(200, br#"[{"i":4}]"#.to_vec()),
        MockOutcome::Status(200, b"[]".to_vec()),
    ]);
    let client = client_with(backend.clone(), 0);

    let mut it = client
        .dataset("some-dataset-id")
        .iterate_items::<serde_json::Value>(Default::default());
    let mut seen = Vec::new();
    while let Some(item) = it.next().await.expect("ok") {
        seen.push(item["i"].as_i64().expect("i field"));
    }

    assert_eq!(
        seen,
        vec![0, 1, 2, 3, 4],
        "all items must be yielded even without a total header"
    );
    assert_eq!(
        backend.call_count(),
        4,
        "three data pages plus the terminating empty page"
    );
}

/// Hermetic coverage of the `list_iterator!` macro wiring used by every offset/limit collection
/// `iterate()`. A total-item cap plus a smaller `with_chunk_size` must (a) yield exactly the cap
/// and (b) drive the per-page request window correctly: the first page requests
/// `min(remaining, chunk_size)` at `offset=0`, and the second advances `offset` by the items
/// received and requests only the remaining budget. Previously this path was exercised only by
/// `APIFY_TOKEN`-gated integration tests, so it was skipped offline.
#[tokio::test]
async fn iterate_macro_caps_and_pages_correctly() {
    let backend = MockBackend::new(vec![
        // Endpoint reports a large total (10) so termination is driven by the cap, not by total.
        MockOutcome::Status(
            200,
            br#"{"data":{"total":10,"items":[{"id":"a0"},{"id":"a1"}]}}"#.to_vec(),
        ),
        MockOutcome::Status(
            200,
            br#"{"data":{"total":10,"items":[{"id":"a2"}]}}"#.to_vec(),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    let mut it = client
        .actors()
        .iterate(apify_client::ActorListOptions {
            limit: Some(3),
            ..Default::default()
        })
        .with_chunk_size(2);
    let mut ids = Vec::new();
    while let Some(actor) = it.next().await.expect("ok") {
        ids.push(actor.id);
    }

    assert_eq!(ids, vec!["a0", "a1", "a2"], "cap of 3 must yield exactly 3");
    assert_eq!(backend.call_count(), 2, "two pages suffice under the cap");
    let urls = backend.urls();
    assert!(
        urls[0].contains("offset=0") && urls[0].contains("limit=2"),
        "first page requests min(remaining=3, chunk=2)=2 at offset 0, got {}",
        urls[0]
    );
    assert!(
        urls[1].contains("offset=2") && urls[1].contains("limit=1"),
        "second page advances to offset 2 and requests the remaining budget of 1, got {}",
        urls[1]
    );
}

/// Hermetic coverage of `RunCollectionClient::iterate`, which builds its iterator directly
/// (not via the `list_iterator!` macro) because it threads a separate `filter` argument. The
/// iterator must walk every page using the reported total for termination and must forward the
/// `status` filter on every page request. Offline-only; the integration suite gates this on a
/// live token.
#[tokio::test]
async fn run_collection_iterate_walks_pages_and_forwards_filter() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            br#"{"data":{"total":3,"limit":2,"items":[{"id":"r0"},{"id":"r1"}]}}"#.to_vec(),
        ),
        MockOutcome::Status(
            200,
            br#"{"data":{"total":3,"limit":2,"items":[{"id":"r2"}]}}"#.to_vec(),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    let mut it = client.runs().iterate(
        Default::default(),
        apify_client::RunListOptions {
            status: vec!["SUCCEEDED".to_owned()],
            ..Default::default()
        },
    );
    let mut ids = Vec::new();
    while let Some(run) = it.next().await.expect("ok") {
        ids.push(run.id);
    }

    assert_eq!(
        ids,
        vec!["r0", "r1", "r2"],
        "all runs across pages must be yielded"
    );
    assert_eq!(backend.call_count(), 2, "two pages then total-driven stop");
    for url in backend.urls() {
        assert!(
            url.contains("status=SUCCEEDED"),
            "the status filter must be forwarded on every page, got {url}"
        );
    }
}

/// Hermetic coverage of the cursor-based `KeyValueStoreClient::iterate_keys`. Key-value stores
/// paginate by `exclusiveStartKey`/`nextExclusiveStartKey` (not offset), so this walks two pages
/// and asserts: every key is yielded, the second request carries the previous page's
/// `nextExclusiveStartKey` as `exclusiveStartKey`, and iteration stops once the API returns a
/// null next cursor.
#[tokio::test]
async fn iterate_keys_walks_cursor_pages() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            br#"{"data":{"items":[{"key":"k0"},{"key":"k1"}],"isTruncated":true,"nextExclusiveStartKey":"k1"}}"#.to_vec(),
        ),
        MockOutcome::Status(
            200,
            br#"{"data":{"items":[{"key":"k2"}],"isTruncated":false,"nextExclusiveStartKey":null}}"#.to_vec(),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    let mut it = client
        .key_value_store("some-store")
        .iterate_keys(Default::default());
    let mut keys = Vec::new();
    while let Some(key) = it.next().await.expect("ok") {
        keys.push(key.key);
    }

    assert_eq!(
        keys,
        vec!["k0", "k1", "k2"],
        "all keys across pages must be yielded"
    );
    assert_eq!(backend.call_count(), 2, "two pages then null-cursor stop");
    let urls = backend.urls();
    assert!(
        !urls[0].contains("exclusiveStartKey="),
        "first page has no start cursor, got {}",
        urls[0]
    );
    assert!(
        urls[1].contains("exclusiveStartKey=k1"),
        "second page must carry the previous nextExclusiveStartKey as the cursor, got {}",
        urls[1]
    );
}

/// `iterate_keys` terminates on the model's `isTruncated == false` flag (the authoritative
/// end-of-data signal), not merely on a null cursor. If the API sends the final page with
/// `isTruncated:false` but still carries a non-null `nextExclusiveStartKey`, the walk must stop
/// and not perform a wasted extra fetch. (The mock repeats its last scripted page, so a regression
/// that followed the stale cursor would loop; the iteration guard below catches that.)
#[tokio::test]
async fn iterate_keys_stops_on_is_truncated_even_with_trailing_cursor() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            br#"{"data":{"items":[{"key":"k0"},{"key":"k1"}],"isTruncated":true,"nextExclusiveStartKey":"k1"}}"#.to_vec(),
        ),
        // Final page: no more data (isTruncated:false) yet a non-null trailing cursor.
        MockOutcome::Status(
            200,
            br#"{"data":{"items":[{"key":"k2"}],"isTruncated":false,"nextExclusiveStartKey":"k2"}}"#.to_vec(),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    let mut it = client
        .key_value_store("some-store")
        .iterate_keys(Default::default());
    let mut keys = Vec::new();
    let mut guard = 0;
    while let Some(key) = it.next().await.expect("ok") {
        keys.push(key.key);
        guard += 1;
        assert!(
            guard < 100,
            "iterate_keys did not terminate on isTruncated=false"
        );
    }

    assert_eq!(
        keys,
        vec!["k0", "k1", "k2"],
        "all keys across pages must be yielded"
    );
    assert_eq!(
        backend.call_count(),
        2,
        "isTruncated:false ends the walk on the second page; no wasted fetch despite the trailing cursor"
    );
}

/// The `iterate_keys` `limit` is a total cap that also bounds the first page's request size
/// (reference parity): with `limit=2` the first request asks for `limit=2` and, once two keys
/// are consumed, the walk stops without a second fetch even though the API advertised more keys.
#[tokio::test]
async fn iterate_keys_limit_caps_the_walk() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        200,
        br#"{"data":{"items":[{"key":"k0"},{"key":"k1"}],"isTruncated":true,"nextExclusiveStartKey":"k1"}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 0);

    let mut it = client
        .key_value_store("some-store")
        .iterate_keys(apify_client::ListKeysOptions {
            limit: Some(2),
            ..Default::default()
        });
    let mut keys = Vec::new();
    while let Some(key) = it.next().await.expect("ok") {
        keys.push(key.key);
    }

    assert_eq!(
        keys,
        vec!["k0", "k1"],
        "yields exactly the first (capped) page"
    );
    assert_eq!(
        backend.call_count(),
        1,
        "the cap is reached on the first page, so no second fetch despite isTruncated"
    );
    assert!(
        backend.urls()[0].contains("limit=2"),
        "the cap bounds the first page's request size, got {}",
        backend.urls()[0]
    );
}

/// `iterate_keys` with `limit = Some(0)` means "iterate everything" — the first request must NOT
/// send `limit=0` (out of the endpoint's `minimum: 1` range); the `0` is normalized to no limit.
#[tokio::test]
async fn iterate_keys_zero_limit_sends_no_limit_and_walks_all() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            br#"{"data":{"items":[{"key":"k0"}],"isTruncated":true,"nextExclusiveStartKey":"k0"}}"#.to_vec(),
        ),
        MockOutcome::Status(
            200,
            br#"{"data":{"items":[{"key":"k1"}],"isTruncated":false,"nextExclusiveStartKey":null}}"#.to_vec(),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    let mut it = client
        .key_value_store("some-store")
        .iterate_keys(apify_client::ListKeysOptions {
            limit: Some(0),
            ..Default::default()
        });
    let mut keys = Vec::new();
    while let Some(key) = it.next().await.expect("ok") {
        keys.push(key.key);
    }

    assert_eq!(keys, vec!["k0", "k1"], "0 must iterate the whole store");
    assert!(
        !backend.urls()[0].contains("limit="),
        "limit=0 must be normalized to no limit param, got {}",
        backend.urls()[0]
    );
}

/// A finite `iterate_keys` cap larger than one page must be clamped to the endpoint maximum per
/// request (so it does not 400) while still yielding across pages until the cap or the store is
/// exhausted.
#[tokio::test]
async fn iterate_keys_large_cap_clamps_page_size() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            br#"{"data":{"items":[{"key":"k0"},{"key":"k1"}],"isTruncated":true,"nextExclusiveStartKey":"k1"}}"#.to_vec(),
        ),
        MockOutcome::Status(
            200,
            br#"{"data":{"items":[{"key":"k2"}],"isTruncated":false,"nextExclusiveStartKey":null}}"#.to_vec(),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    // Cap of 1500 > the endpoint max (1000): each request must ask for at most 1000.
    let mut it = client
        .key_value_store("some-store")
        .iterate_keys(apify_client::ListKeysOptions {
            limit: Some(1500),
            ..Default::default()
        });
    let mut keys = Vec::new();
    while let Some(key) = it.next().await.expect("ok") {
        keys.push(key.key);
    }

    assert_eq!(
        keys,
        vec!["k0", "k1", "k2"],
        "walk continues under the large cap"
    );
    let urls = backend.urls();
    assert!(
        urls[0].contains("limit=1000") && urls[1].contains("limit=1000"),
        "each request must be clamped to the endpoint max of 1000, got {urls:?}"
    );
    assert!(
        urls[1].contains("exclusiveStartKey=k1"),
        "the cursor must still advance across the clamped pages, got {}",
        urls[1]
    );
}

/// Builds a minimal request for the `batch_add_requests` tests below.
fn rq_request(unique_key: &str) -> RequestQueueRequest {
    RequestQueueRequest {
        id: None,
        url: format!("https://example.com/{unique_key}"),
        unique_key: Some(unique_key.to_string()),
        method: Some("GET".to_string()),
        user_data: None,
        extra: Default::default(),
    }
}

/// `batch_add_requests` count-chunks input larger than the 25-per-call API limit and merges
/// each chunk's `processedRequests`/`unprocessedRequests` into one result. With the default
/// `max_parallel` (5) comfortably above the 2 chunks a 30-item input produces, both chunk calls
/// are dispatched without the caller having to do anything — this asserts the merge is complete
/// and correct regardless of how the two concurrent calls interleave.
#[tokio::test]
async fn batch_add_requests_chunks_and_merges_across_calls() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            br#"{"data":{"processedRequests":[{"uniqueKey":"a","requestId":"r-a","wasAlreadyPresent":false,"wasAlreadyHandled":false}],"unprocessedRequests":[]}}"#.to_vec(),
        ),
        MockOutcome::Status(
            200,
            br#"{"data":{"processedRequests":[{"uniqueKey":"b","requestId":"r-b","wasAlreadyPresent":false,"wasAlreadyHandled":false}],"unprocessedRequests":[]}}"#.to_vec(),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    let requests: Vec<RequestQueueRequest> =
        (0..30).map(|i| rq_request(&format!("k{i}"))).collect();
    let result = client
        .request_queue("some-queue")
        .batch_add_requests(&requests, false)
        .await
        .expect("batch add should succeed");

    assert_eq!(
        backend.call_count(),
        2,
        "30 requests must be split into exactly 2 chunks (25 + 5)"
    );
    let processed = result["processedRequests"]
        .as_array()
        .expect("processedRequests array");
    assert_eq!(
        processed.len(),
        2,
        "results from both chunk calls must be merged into one array"
    );
    assert!(result["unprocessedRequests"]
        .as_array()
        .expect("unprocessedRequests array")
        .is_empty());
}

/// A chunk whose response reports `unprocessedRequests` (typically caused by rate limiting) is
/// retried with the still-unprocessed subset, and the retry's results are merged into the final
/// response. This exercises `_batchAddRequestsWithRetries` parity: the first call reports 3
/// requests processed and 2 unprocessed; the retry (with only the 2 remaining) reports both
/// processed, so the final result has all 5 processed and none unprocessed.
#[tokio::test]
async fn batch_add_requests_retries_unprocessed_requests() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            br#"{"data":{"processedRequests":[{"uniqueKey":"u0","requestId":"r0","wasAlreadyPresent":false,"wasAlreadyHandled":false},{"uniqueKey":"u1","requestId":"r1","wasAlreadyPresent":false,"wasAlreadyHandled":false},{"uniqueKey":"u2","requestId":"r2","wasAlreadyPresent":false,"wasAlreadyHandled":false}],"unprocessedRequests":[{"uniqueKey":"u3","url":"https://example.com/u3","method":"GET"},{"uniqueKey":"u4","url":"https://example.com/u4","method":"GET"}]}}"#.to_vec(),
        ),
        MockOutcome::Status(
            200,
            br#"{"data":{"processedRequests":[{"uniqueKey":"u3","requestId":"r3","wasAlreadyPresent":false,"wasAlreadyHandled":false},{"uniqueKey":"u4","requestId":"r4","wasAlreadyPresent":false,"wasAlreadyHandled":false}],"unprocessedRequests":[]}}"#.to_vec(),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    let requests: Vec<RequestQueueRequest> = (0..5).map(|i| rq_request(&format!("u{i}"))).collect();
    let result = client
        .request_queue("some-queue")
        .batch_add_requests_with_options(
            &requests,
            BatchAddRequestsOptions {
                min_delay_between_unprocessed_requests_retries: Some(Duration::from_millis(1)),
                ..Default::default()
            },
        )
        .await
        .expect("batch add should not fail even before all retries complete");

    assert_eq!(
        backend.call_count(),
        2,
        "the unprocessed pair must be retried in a second call"
    );
    assert_eq!(
        result["processedRequests"]
            .as_array()
            .expect("processedRequests array")
            .len(),
        5,
        "all 5 requests must end up processed after the retry"
    );
    assert!(result["unprocessedRequests"]
        .as_array()
        .expect("unprocessedRequests array")
        .is_empty());
}

/// When a chunk call fails outright (not merely reporting `unprocessedRequests`, but erroring),
/// `batch_add_requests` must not propagate the error: the reference client's
/// `_batchAddRequestsWithRetries` guarantees this method's signature never throws for a partial
/// failure. The not-yet-processed requests must instead show up as `unprocessedRequests`.
#[tokio::test]
async fn batch_add_requests_chunk_error_becomes_unprocessed_not_a_failure() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        400,
        br#"{"error":{"type":"invalid-request","message":"bad batch"}}"#.to_vec(),
    )]);
    // No transport retries and no application-level retries, so exactly one call is made.
    let client = client_with(backend.clone(), 0);

    let requests = vec![rq_request("e0"), rq_request("e1")];
    let result = client
        .request_queue("some-queue")
        .batch_add_requests_with_options(
            &requests,
            BatchAddRequestsOptions {
                max_unprocessed_requests_retries: Some(0),
                ..Default::default()
            },
        )
        .await
        .expect("a failed chunk must not fail the overall call");

    assert_eq!(backend.call_count(), 1);
    assert!(result["processedRequests"]
        .as_array()
        .expect("processedRequests array")
        .is_empty());
    let unprocessed = result["unprocessedRequests"]
        .as_array()
        .expect("unprocessedRequests array");
    assert_eq!(
        unprocessed.len(),
        2,
        "both requests must be reported unprocessed after the chunk call failed"
    );
}

/// Requests whose combined serialized size exceeds the API's payload byte limit are sliced into
/// multiple `requests/batch` calls even when their count is well under the 25-per-call cap,
/// mirroring the reference client's `sliceArrayByByteLength`. Three ~4 MiB requests (~12 MiB
/// total) exceed the ~9 MiB limit, so they cannot all fit in one call.
#[tokio::test]
async fn batch_add_requests_slices_by_byte_length() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        200,
        br#"{"data":{"processedRequests":[],"unprocessedRequests":[]}}"#.to_vec(),
    )]);
    let client = client_with(backend.clone(), 0);

    let big_value = "a".repeat(4 * 1024 * 1024); // ~4 MiB
    let requests: Vec<RequestQueueRequest> = (0..3)
        .map(|i| {
            let mut r = rq_request(&format!("big{i}"));
            r.user_data = Some(serde_json::Value::String(big_value.clone()));
            r
        })
        .collect();

    client
        .request_queue("some-queue")
        .batch_add_requests(&requests, false)
        .await
        .expect("batch add should succeed");

    assert!(
        backend.call_count() > 1,
        "requests too large to fit in one payload must be split across multiple calls, got {} call(s)",
        backend.call_count()
    );
}

/// `RunClient::charge` sends `POST .../actor-runs/{runId}/charge` with an auto-generated
/// `idempotency-key` header of the documented `{runId}-{eventName}-{millis}-{random}` shape, and
/// a JSON body carrying `eventName`/`count`. Hermetic coverage of the one thing
/// `post_raw_with_extra_header` exists for (the idempotency header) — `charge` itself is not
/// covered by a live integration test because it bills real money and requires a pay-per-event
/// Actor unavailable as a fixture here, so this offline test is the only guard on its request
/// shape.
#[tokio::test]
async fn charge_sends_idempotency_key_header_and_body() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"".to_vec())]);
    let client = client_with(backend.clone(), 0);

    client
        .run("run123")
        .charge(RunChargeOptions {
            event_name: "pageScraped".to_string(),
            count: Some(3),
            idempotency_key: None,
        })
        .await
        .expect("charge should succeed");

    assert_eq!(backend.last_method(), Some(HttpMethod::Post));
    let url = backend.last_url().expect("a request was sent");
    assert!(
        url.contains("/actor-runs/run123/charge"),
        "expected a POST to .../actor-runs/run123/charge, got {url}"
    );

    let key = backend
        .last_header("idempotency-key")
        .expect("charge must send an idempotency-key header");
    // Documented shape: `{runId}-{eventName}-{millis}-{random}`. `event_name` above has no
    // dashes, so splitting from the right on '-' unambiguously separates the trailing
    // `millis`/`random` numeric segments from the `runId-eventName` prefix.
    let parts: Vec<&str> = key.rsplitn(3, '-').collect();
    assert_eq!(
        parts.len(),
        3,
        "expected `runId-eventName-millis-random`, got {key}"
    );
    let (random_suffix, millis, prefix) = (parts[0], parts[1], parts[2]);
    assert_eq!(prefix, "run123-pageScraped");
    assert!(
        !millis.is_empty() && millis.chars().all(|c| c.is_ascii_digit()),
        "millis segment should be numeric, got {millis} in {key}"
    );
    assert!(
        !random_suffix.is_empty() && random_suffix.chars().all(|c| c.is_ascii_digit()),
        "random segment should be numeric, got {random_suffix} in {key}"
    );

    let body = backend.last_body().expect("a body was sent");
    let body: serde_json::Value = serde_json::from_slice(&body).expect("body should be JSON");
    assert_eq!(body["eventName"], "pageScraped");
    assert_eq!(body["count"], 3);
}

/// `RunClient::charge` with `idempotency_key: None` still sends a header when `count` is
/// omitted, defaulting the body's `count` to `1`.
#[tokio::test]
async fn charge_defaults_count_to_one() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"".to_vec())]);
    let client = client_with(backend.clone(), 0);

    client
        .run("run123")
        .charge(RunChargeOptions {
            event_name: "pageScraped".to_string(),
            count: None,
            idempotency_key: None,
        })
        .await
        .expect("charge should succeed");

    let body = backend.last_body().expect("a body was sent");
    let body: serde_json::Value = serde_json::from_slice(&body).expect("body should be JSON");
    assert_eq!(body["count"], 1, "count should default to 1 when omitted");
}

/// `UserClient::update_limits` sends `PUT .../users/me/limits` with the serialized body.
/// Hermetic coverage: the live variant would mutate the shared test account's real limits,
/// which is unsafe under the concurrent-test-account requirement, so this offline test is the
/// only guard on its request shape.
#[tokio::test]
async fn update_limits_sends_put_with_body() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"".to_vec())]);
    let client = client_with(backend.clone(), 0);

    client
        .me()
        .update_limits(&serde_json::json!({ "maxMonthlyUsageUsd": 500 }))
        .await
        .expect("update_limits should succeed");

    assert_eq!(backend.last_method(), Some(HttpMethod::Put));
    let url = backend.last_url().expect("a request was sent");
    assert!(
        url.contains("/users/me/limits"),
        "expected a PUT to .../users/me/limits, got {url}"
    );

    let body = backend.last_body().expect("a body was sent");
    let body: serde_json::Value = serde_json::from_slice(&body).expect("body should be JSON");
    assert_eq!(body["maxMonthlyUsageUsd"], 500);
}

/// The standalone `ApifyClient::log(build_or_run_id)` (`GET /v2/logs/{buildOrRunId}`) is
/// distinct from the nested `run().log()`/`build().log()` accessors, which hit
/// `.../actor-runs/{runId}/log` and `.../actor-builds/{buildId}/log` respectively. This is
/// hermetic coverage of the standalone accessor's URL, which — unlike the nested variants — was
/// previously exercised by no test at all.
#[tokio::test]
async fn standalone_log_hits_top_level_logs_endpoint() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"log output".to_vec())]);
    let client = client_with(backend.clone(), 0);

    let log = client
        .log("some-build-or-run-id")
        .get()
        .await
        .expect("get log");

    assert_eq!(log.as_deref(), Some("log output"));
    let url = backend.last_url().expect("a request was sent");
    assert!(
        url.contains("/logs/some-build-or-run-id"),
        "standalone log() must hit the top-level /logs/{{id}} endpoint, got {url}"
    );
    assert!(
        !url.contains("/actor-runs/") && !url.contains("/actor-builds/"),
        "standalone log() must not go through the nested run/build log path, got {url}"
    );
}
