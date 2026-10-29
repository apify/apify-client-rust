//! Offline unit tests for the timeout tiers, the default headers and the request-body compressors.
//!
//! These use a mock [`HttpBackend`](apify_client::http_client::HttpBackend) and require no network
//! access or `APIFY_TOKEN`.

mod common;

use std::sync::Arc;
use std::time::Duration;

use apify_client::http_client::{HttpBackend, HttpMethod, HttpRequest, ReqwestBackend};
use apify_client::timeouts::NO_TIMEOUT;
use apify_client::{
    ActorStartOptions, ApifyClient, ApifyClientError, BrotliHttpCompressor, GzipHttpCompressor,
    HttpCompressor, Timeout,
};
use common::mock_backend::{client_with, MockBackend, MockOutcome};
use serde_json::{json, Value};

/// Minimum request-body size that gets compressed.
const COMPRESSIBLE_BODY_BYTES: usize = 4096;

fn ok_data(data: Value) -> MockOutcome {
    MockOutcome::Status(200, json!({ "data": data }).to_string().into_bytes())
}

/// A builder preconfigured with the mock backend, the way `client_with` does it.
fn builder_with(backend: Arc<MockBackend>) -> apify_client::ApifyClientBuilder {
    ApifyClient::builder()
        .token("test-token")
        .max_retries(0)
        .min_delay_between_retries(Duration::from_millis(1))
        .http_backend(backend)
}

#[tokio::test]
async fn simple_metadata_reads_use_the_short_tier() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "a" }))]);
    let client = client_with(backend.clone(), 0);
    client.actor("a").get().await.expect("get actor");
    assert_eq!(backend.last_timeout(), Duration::from_secs(5));
}

#[tokio::test]
async fn listing_uses_the_medium_tier() {
    let backend = MockBackend::new(vec![ok_data(json!({ "items": [] }))]);
    let client = client_with(backend.clone(), 0);
    client
        .actors()
        .list(Default::default())
        .await
        .expect("list actors");
    assert_eq!(backend.last_timeout(), Duration::from_secs(30));
}

#[tokio::test]
async fn dataset_items_use_the_long_tier() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"[]".to_vec())]);
    let client = client_with(backend.clone(), 0);
    client
        .dataset("d")
        .list_items::<Value>(Default::default())
        .await
        .expect("list items");
    assert_eq!(backend.last_timeout(), Duration::from_secs(360));
}

#[tokio::test]
async fn builder_sets_the_duration_of_each_tier() {
    let backend = MockBackend::new(vec![
        ok_data(json!({ "id": "a" })),
        ok_data(json!({ "items": [] })),
        MockOutcome::Status(200, b"[]".to_vec()),
    ]);
    let client = builder_with(backend.clone())
        .timeout_short(Duration::from_secs(1))
        .timeout_medium(Duration::from_secs(2))
        .timeout_long(Duration::from_secs(3))
        .build();
    client.actor("a").get().await.unwrap();
    client.actors().list(Default::default()).await.unwrap();
    client
        .dataset("d")
        .list_items::<Value>(Default::default())
        .await
        .unwrap();
    assert_eq!(
        backend.timeouts(),
        vec![
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(3)
        ]
    );
}

#[tokio::test]
async fn timeout_max_caps_a_tier() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "a" }))]);
    let client = builder_with(backend.clone())
        .timeout_max(Duration::from_secs(2))
        .build();
    client.actor("a").get().await.unwrap();
    assert_eq!(backend.last_timeout(), Duration::from_secs(2));
}

#[tokio::test]
async fn the_original_timeout_option_is_the_cap_on_an_attempt() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "a" }))]);
    let client = builder_with(backend.clone())
        .timeout(Duration::from_secs(3))
        .build();
    client.actor("a").get().await.unwrap();
    assert_eq!(backend.last_timeout(), Duration::from_secs(3));
}

#[tokio::test]
async fn with_timeout_replaces_the_default_tier() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "a" }))]);
    let client = client_with(backend.clone(), 0);
    client
        .actor("a")
        .with_timeout(Timeout::Long)
        .get()
        .await
        .unwrap();
    client
        .actor("a")
        .with_timeout(Timeout::Custom(Duration::from_secs(7)))
        .get()
        .await
        .unwrap();
    assert_eq!(
        backend.timeouts(),
        vec![Duration::from_secs(360), Duration::from_secs(7)]
    );
}

#[tokio::test]
async fn an_explicit_timeout_above_the_cap_is_lowered_to_it() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "a" }))]);
    let client = client_with(backend.clone(), 0);
    client
        .actor("a")
        .with_timeout(Timeout::Custom(Duration::from_secs(10_000)))
        .get()
        .await
        .unwrap();
    assert_eq!(backend.last_timeout(), Duration::from_secs(360));
}

#[tokio::test]
async fn no_timeout_is_passed_to_the_backend_as_such() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "a" }))]);
    let client = client_with(backend.clone(), 0);
    client
        .actor("a")
        .with_timeout(Timeout::NoTimeout)
        .get()
        .await
        .unwrap();
    assert_eq!(backend.last_timeout(), NO_TIMEOUT);
}

#[tokio::test]
async fn the_timeout_doubles_with_every_retry_up_to_the_cap() {
    let backend = MockBackend::new(vec![MockOutcome::Status(500, b"{}".to_vec())]);
    let client = builder_with(backend.clone())
        .max_retries(5)
        .timeout_max(Duration::from_secs(30))
        .build();
    client.actor("a").get().await.expect_err("always fails");
    let seconds: Vec<u64> = backend.timeouts().iter().map(Duration::as_secs).collect();
    assert_eq!(seconds, vec![5, 10, 20, 30, 30, 30]);
}

#[tokio::test]
async fn a_request_without_a_timeout_never_gets_one_on_retry() {
    let backend = MockBackend::new(vec![MockOutcome::Status(500, b"{}".to_vec())]);
    let client = builder_with(backend.clone()).max_retries(2).build();
    client
        .actor("a")
        .with_timeout(Timeout::NoTimeout)
        .get()
        .await
        .expect_err("always fails");
    assert_eq!(backend.timeouts(), vec![NO_TIMEOUT; 3]);
}

#[tokio::test]
async fn starting_a_run_adds_the_server_side_wait_to_the_tier() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "r" }))]);
    let client = client_with(backend.clone(), 0);
    let actor = client.actor("a");
    for wait_for_finish in [None, Some(10), Some(999)] {
        actor
            .start::<Value>(
                None,
                ActorStartOptions {
                    wait_for_finish,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }
    // The API holds a `waitForFinish` response for at most a minute, so the client allows the
    // hold the caller asked for (capped there) on top of the `Medium` tier.
    assert_eq!(
        backend.timeouts(),
        vec![
            Duration::from_secs(30),
            Duration::from_secs(40),
            Duration::from_secs(90)
        ]
    );
}

#[tokio::test]
async fn an_explicit_timeout_is_used_as_given_with_a_server_side_wait() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "r" }))]);
    let client = client_with(backend.clone(), 0);
    client
        .actor("a")
        .with_timeout(Timeout::Custom(Duration::from_secs(7)))
        .start::<Value>(
            None,
            ActorStartOptions {
                wait_for_finish: Some(30),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(backend.last_timeout(), Duration::from_secs(7));
}

#[tokio::test(start_paused = true)]
async fn polling_for_a_run_to_finish_has_no_client_side_timeout() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "r", "status": "SUCCEEDED" }))]);
    let client = client_with(backend.clone(), 0);
    client.run("r").wait_for_finish(None).await.unwrap();
    assert_eq!(backend.last_timeout(), NO_TIMEOUT);
}

/// The default backend must not apply a timeout of its own to a request without one.
#[tokio::test]
async fn reqwest_backend_sends_a_request_without_a_timeout() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = [0u8; 1024];
        let _ = socket.read(&mut buffer).await.unwrap();
        socket
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\n{}")
            .await
            .unwrap();
    });

    let response = ReqwestBackend::new()
        .send(HttpRequest {
            method: HttpMethod::Get,
            url: format!("http://{address}/"),
            headers: Default::default(),
            body: None,
            timeout: NO_TIMEOUT,
        })
        .await
        .expect("a request without a timeout is sent");
    assert_eq!(response.status, 200);
}

/// The default backend aborts a request that outlives its timeout.
#[tokio::test]
async fn reqwest_backend_aborts_a_request_that_outlives_its_timeout() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    // Accept the connection, but never answer it.
    let _silent_server = tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_secs(30)).await;
    });

    let error = ReqwestBackend::new()
        .send(HttpRequest {
            method: HttpMethod::Get,
            url: format!("http://{address}/"),
            headers: Default::default(),
            body: None,
            timeout: Duration::from_millis(50),
        })
        .await
        .expect_err("the request must time out");
    assert!(matches!(error, ApifyClientError::Timeout), "{error:?}");
}

#[tokio::test]
async fn default_headers_win_over_the_built_in_ones_and_are_case_insensitive() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "a" }))]);
    let client = builder_with(backend.clone())
        .default_header("X-Custom", "first")
        .default_header("x-custom", "second")
        .default_header("user-agent", "my-agent/1.0")
        .build();
    client.actor("a").get().await.unwrap();
    assert_eq!(backend.last_header("X-Custom").as_deref(), Some("second"));
    assert_eq!(
        backend.last_header("User-Agent").as_deref(),
        Some("my-agent/1.0")
    );
    assert_eq!(
        backend.last_header("Authorization").as_deref(),
        Some("Bearer test-token"),
        "an unrelated built-in header is kept"
    );
}

#[tokio::test]
async fn a_request_header_wins_over_a_default_header() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, Vec::new())]);
    let client = builder_with(backend.clone())
        .default_header("content-type", "text/csv")
        .build();
    client
        .key_value_store("s")
        .set_record_raw("k", b"x".to_vec(), "text/plain")
        .await
        .unwrap();
    assert_eq!(
        backend.last_header("Content-Type").as_deref(),
        Some("text/plain")
    );
}

/// A compressor that leaves the body as it is, to see the custom compressor reach the wire.
#[derive(Debug)]
struct IdentityCompressor;

impl HttpCompressor for IdentityCompressor {
    fn content_encoding(&self) -> &str {
        "identity"
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, ApifyClientError> {
        Ok(data.to_vec())
    }
}

/// Uploads a compressible record through a client using `compressor`, and returns the backend that
/// saw the request.
async fn upload_with(compressor: Arc<dyn HttpCompressor>) -> Arc<MockBackend> {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, Vec::new())]);
    let client = builder_with(backend.clone())
        .http_compressor(compressor)
        .build();
    client
        .key_value_store("s")
        .set_record_raw("k", vec![b'a'; COMPRESSIBLE_BODY_BYTES], "text/plain")
        .await
        .unwrap();
    backend
}

#[tokio::test]
async fn a_custom_compressor_sets_the_content_encoding_and_the_body() {
    let backend = upload_with(Arc::new(IdentityCompressor)).await;
    assert_eq!(
        backend.last_header("Content-Encoding").as_deref(),
        Some("identity")
    );
    assert_eq!(
        backend.last_body().unwrap(),
        vec![b'a'; COMPRESSIBLE_BODY_BYTES]
    );
}

#[tokio::test]
async fn brotli_quality_is_configurable() {
    let compressor = BrotliHttpCompressor::new(11).expect("11 is a valid brotli quality");
    let backend = upload_with(Arc::new(compressor)).await;
    assert_eq!(
        backend.last_header("Content-Encoding").as_deref(),
        Some("br")
    );
    let sent = backend.last_body().unwrap();
    let mut decompressed = Vec::new();
    let mut decoder = brotli::DecompressorWriter::new(&mut decompressed, 4096);
    std::io::Write::write_all(&mut decoder, &sent).expect("decode");
    drop(decoder);
    assert_eq!(decompressed, vec![b'a'; COMPRESSIBLE_BODY_BYTES]);
}

#[tokio::test]
async fn gzip_quality_is_configurable() {
    let compressor = GzipHttpCompressor::new(1).expect("1 is a valid gzip quality");
    let backend = upload_with(Arc::new(compressor)).await;
    assert_eq!(
        backend.last_header("Content-Encoding").as_deref(),
        Some("gzip")
    );
    let sent = backend.last_body().unwrap();
    let mut decompressed = Vec::new();
    std::io::Read::read_to_end(
        &mut flate2::read::GzDecoder::new(&sent[..]),
        &mut decompressed,
    )
    .expect("decode");
    assert_eq!(decompressed, vec![b'a'; COMPRESSIBLE_BODY_BYTES]);
}

#[test]
fn a_quality_outside_the_range_of_the_algorithm_is_rejected() {
    assert!(matches!(
        BrotliHttpCompressor::new(12),
        Err(ApifyClientError::InvalidArgument(_))
    ));
    assert!(matches!(
        GzipHttpCompressor::new(0),
        Err(ApifyClientError::InvalidArgument(_))
    ));
}
