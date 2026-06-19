//! Offline unit tests for the HTTP layer (retries, backoff, error parsing, 404 handling).
//!
//! These use a mock [`HttpBackend`] and require no network access or `APIFY_TOKEN`, so the
//! most bug-prone logic is always exercised in CI.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use apify_client::http_client::{HttpBackend, HttpRequest, HttpResponse};
use apify_client::{ApifyClient, ApifyClientError};
use async_trait::async_trait;

/// A scripted backend that returns a queued sequence of responses and counts calls.
#[derive(Debug)]
struct MockBackend {
    responses: Mutex<Vec<MockOutcome>>,
    calls: AtomicUsize,
    last_url: Mutex<Option<String>>,
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
        })
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn last_url(&self) -> Option<String> {
        self.last_url.lock().unwrap().clone()
    }
}

#[async_trait]
impl HttpBackend for MockBackend {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, ApifyClientError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_url.lock().unwrap() = Some(request.url.clone());
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
