//! Offline unit tests for `call`, the retry of non-idempotent starts, the polling timeouts, the
//! `WaitForResources` limits and the log redirection.
//!
//! These use scripted [`HttpBackend`]s and require no network access or `APIFY_TOKEN`.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use apify_client::clients::actor::WaitForResources;
use apify_client::http_client::{HttpBackend, HttpMethod, HttpRequest, HttpResponse};
use apify_client::timeouts::NO_TIMEOUT;
use apify_client::{
    ActorBuildOptions, ActorCallOptions, ActorStartOptions, ApifyClient, ApifyClientError,
    LogRedirect, StreamedLogOptions, TaskStartOptions, Timeout,
};
use async_trait::async_trait;
use common::mock_backend::{client_with, MockBackend, MockOutcome};
use serde_json::{json, Value};

const RUN_READY: &str = r#"{"data":{"id":"r1","actId":"a1","status":"READY"}}"#;
const RUN_DONE: &str = r#"{"data":{"id":"r1","actId":"a1","status":"SUCCEEDED"}}"#;

fn ok(body: &str) -> MockOutcome {
    MockOutcome::Status(200, body.as_bytes().to_vec())
}

fn call_options(wait_secs: Option<i64>) -> ActorCallOptions {
    ActorCallOptions {
        wait_secs,
        log: LogRedirect::Disabled,
        ..Default::default()
    }
}

type Route = dyn Fn(&HttpRequest, usize) -> Result<HttpResponse, ApifyClientError> + Send + Sync;

/// A backend that answers by request, and records every request.
struct RouteBackend {
    requests: Mutex<Vec<HttpRequest>>,
    route: Box<Route>,
}

impl std::fmt::Debug for RouteBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RouteBackend")
    }
}

impl RouteBackend {
    fn new(
        route: impl Fn(&HttpRequest, usize) -> Result<HttpResponse, ApifyClientError>
            + Send
            + Sync
            + 'static,
    ) -> Arc<Self> {
        Arc::new(Self {
            requests: Mutex::new(Vec::new()),
            route: Box::new(route),
        })
    }

    fn requests(&self) -> Vec<HttpRequest> {
        self.requests.lock().unwrap().clone()
    }

    fn count(&self, method: HttpMethod, url_part: &str) -> usize {
        self.requests()
            .iter()
            .filter(|r| r.method == method && r.url.contains(url_part))
            .count()
    }
}

#[async_trait]
impl HttpBackend for RouteBackend {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, ApifyClientError> {
        let index = {
            let mut requests = self.requests.lock().unwrap();
            requests.push(request.clone());
            requests.len() - 1
        };
        (self.route)(&request, index)
    }
}

fn response(status: u16, body: &str) -> Result<HttpResponse, ApifyClientError> {
    Ok(HttpResponse {
        status,
        headers: Default::default(),
        body: body.as_bytes().to_vec(),
    })
}

fn client_on(backend: Arc<RouteBackend>, max_retries: u32) -> ApifyClient {
    ApifyClient::builder()
        .token("test-token")
        .max_retries(max_retries)
        .min_delay_between_retries(Duration::from_millis(1))
        .timeout_medium(Duration::from_secs(1))
        .http_backend(backend)
        .build()
}

/// A start the API holds for longer than the `Medium` tier must not be aborted and retried by
/// `call`: that would POST a second run. The start has no timeout, so the POST happens once.
#[tokio::test]
async fn call_does_not_abort_and_retry_a_slow_start() {
    let backend = RouteBackend::new(|request, _| {
        if request.method == HttpMethod::Post {
            // The API holds the start for longer than the tier the client would give it.
            if request.timeout < Duration::from_millis(1500) {
                return Err(ApifyClientError::Timeout);
            }
            return response(201, RUN_READY);
        }
        response(200, RUN_DONE)
    });
    let client = client_on(backend.clone(), 3);

    let run = client
        .actor("a")
        .call::<Value>(None, call_options(None))
        .await
        .unwrap();
    assert_eq!(run.status.as_deref(), Some("SUCCEEDED"));
    assert_eq!(backend.count(HttpMethod::Post, "/runs"), 1);

    let run = client
        .task("t")
        .call::<Value>(None, TaskStartOptions::default(), None)
        .await
        .unwrap();
    assert_eq!(run.status.as_deref(), Some("SUCCEEDED"));
    assert_eq!(backend.count(HttpMethod::Post, "/runs"), 2);
}

#[tokio::test]
async fn call_starts_without_a_timeout_and_polls_as_wait_secs_asks() {
    let backend = MockBackend::new(vec![ok(RUN_READY), ok(RUN_DONE)]);
    let client = client_with(backend.clone(), 0);
    client
        .actor("a")
        .call::<Value>(None, call_options(None))
        .await
        .unwrap();
    assert_eq!(backend.timeouts(), vec![NO_TIMEOUT, NO_TIMEOUT]);

    // A bounded wait gives each poll the `Medium` tier plus the hold it asks the API for, so a
    // stalled connection cannot outlast the budget.
    let backend = MockBackend::new(vec![ok(RUN_READY), ok(RUN_DONE)]);
    let client = client_with(backend.clone(), 0);
    client
        .actor("a")
        .call_raw(b"{}", call_options(Some(10)))
        .await
        .unwrap();
    let timeouts = backend.timeouts();
    assert_eq!(timeouts[0], NO_TIMEOUT);
    assert!(
        timeouts[1] > Duration::from_secs(30) && timeouts[1] <= Duration::from_secs(40),
        "{timeouts:?}"
    );
}

/// A timeout set on the client applies to the start and to every poll.
#[tokio::test]
async fn call_applies_the_timeout_of_the_client_to_the_start_and_the_polls() {
    let backend = MockBackend::new(vec![
        ok(RUN_READY),
        ok(RUN_DONE),
        ok(RUN_READY),
        ok(RUN_DONE),
    ]);
    let client = client_with(backend.clone(), 0);
    let seven = Timeout::Custom(Duration::from_secs(7));
    client
        .actor("a")
        .with_timeout(seven)
        .call::<Value>(None, call_options(None))
        .await
        .unwrap();
    client
        .task("t")
        .with_timeout(seven)
        .call::<Value>(None, TaskStartOptions::default(), Some(5))
        .await
        .unwrap();
    assert_eq!(backend.timeouts(), vec![Duration::from_secs(7); 4]);
}

/// A start or a build that timed out may have reached the API, so it is not retried.
#[tokio::test]
async fn a_start_or_build_that_times_out_is_not_retried() {
    let backend = MockBackend::new(vec![MockOutcome::Timeout]);
    let client = client_with(backend.clone(), 3);
    let wait = ActorStartOptions {
        wait_for_finish: Some(2),
        ..Default::default()
    };

    let result = client.actor("a").start::<Value>(None, wait.clone()).await;
    assert!(matches!(result, Err(ApifyClientError::Timeout)));
    assert_eq!(backend.call_count(), 1);

    let result = client.actor("a").start_raw(b"{}", wait).await;
    assert!(matches!(result, Err(ApifyClientError::Timeout)));
    assert_eq!(backend.call_count(), 2);

    let result = client
        .actor("a")
        .build(
            "0.1",
            ActorBuildOptions {
                wait_for_finish: Some(2),
                ..Default::default()
            },
        )
        .await;
    assert!(matches!(result, Err(ApifyClientError::Timeout)));
    assert_eq!(backend.call_count(), 3);

    let result = client
        .task("t")
        .start::<Value>(None, TaskStartOptions::default())
        .await;
    assert!(matches!(result, Err(ApifyClientError::Timeout)));
    assert_eq!(backend.call_count(), 4);

    // Other errors are still retried.
    let backend = MockBackend::new(vec![MockOutcome::NetworkError]);
    let client = client_with(backend.clone(), 2);
    assert!(client
        .actor("a")
        .start::<Value>(None, Default::default())
        .await
        .is_err());
    assert_eq!(backend.call_count(), 3);
}

/// A task start takes neither `contentType` nor `forcePermissionLevel`.
#[tokio::test]
async fn a_task_start_sends_only_the_parameters_the_endpoint_documents() {
    let backend = MockBackend::new(vec![ok(RUN_READY)]);
    let client = client_with(backend.clone(), 0);
    client
        .task("t")
        .start::<Value>(
            Some(&json!({})),
            TaskStartOptions {
                build: Some("beta".to_string()),
                memory_mbytes: Some(512),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let url = backend.last_url().unwrap();
    assert!(
        url.contains("build=beta") && url.contains("memory=512"),
        "{url}"
    );
    assert!(!url.contains("forcePermissionLevel"), "{url}");
}

#[tokio::test]
async fn get_request_sends_no_client_key() {
    let backend = MockBackend::new(vec![ok(r#"{"data":{"id":"x","url":"u"}}"#)]);
    let client = client_with(backend.clone(), 0);
    client
        .request_queue("q")
        .with_client_key("key")
        .get_request("x")
        .await
        .unwrap();
    assert!(!backend.last_url().unwrap().contains("clientKey"));
}

#[tokio::test(start_paused = true)]
async fn wait_for_resources_accepts_an_unbounded_wait_and_rejects_nan() {
    let limit = MockOutcome::Status(
        402,
        br#"{"error":{"type":"actor-memory-limit-exceeded","message":"m"}}"#.to_vec(),
    );
    for secs in [f64::INFINITY, 1e30, f64::MAX] {
        let backend = MockBackend::new(vec![limit.clone(), ok(RUN_READY)]);
        let client = client_with(backend.clone(), 0);
        client
            .actor("a")
            .start::<Value>(
                None,
                ActorStartOptions {
                    wait_for_resources: Some(WaitForResources::ForSecs(secs)),
                    ..Default::default()
                },
            )
            .await
            .unwrap_or_else(|e| panic!("{secs}: {e}"));
        assert_eq!(backend.call_count(), 2);
    }

    let backend = MockBackend::new(vec![ok(RUN_READY)]);
    let client = client_with(backend.clone(), 0);
    let result = client
        .actor("a")
        .start::<Value>(
            None,
            ActorStartOptions {
                wait_for_resources: Some(WaitForResources::ForSecs(f64::NAN)),
                ..Default::default()
            },
        )
        .await;
    assert!(matches!(result, Err(ApifyClientError::InvalidArgument(_))));
    assert_eq!(backend.call_count(), 0);
}

const LOG: &str = "2024-01-01T00:00:00.000Z first\n  trace\n2024-01-01T00:00:01.000Z second\n";

/// Serves a run, its Actor and its log. The first `empty_streams` log streams are empty, like the
/// log of a run that has not logged yet.
fn log_backend(empty_streams: usize) -> Arc<RouteBackend> {
    let streams = Arc::new(Mutex::new(0usize));
    RouteBackend::new(move |request, _| {
        let url = &request.url;
        if url.contains("/log") {
            if url.contains("stream=1") {
                let mut seen = streams.lock().unwrap();
                *seen += 1;
                return response(200, if *seen <= empty_streams { "" } else { LOG });
            }
            return response(200, LOG);
        }
        if url.contains("/actors/a1") {
            return response(200, r#"{"data":{"id":"a1","name":"my-actor"}}"#);
        }
        if request.method == HttpMethod::Post {
            return response(201, RUN_READY);
        }
        response(200, RUN_DONE)
    })
}

fn collecting_sink() -> (Arc<Mutex<Vec<String>>>, LogRedirect) {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let sink = messages.clone();
    let redirect =
        LogRedirect::to_sink(move |line: &str| sink.lock().unwrap().push(line.to_string()));
    (messages, redirect)
}

#[tokio::test]
async fn a_streamed_log_redirects_the_messages_of_the_log_to_the_sink() {
    let backend = log_backend(0);
    let client = client_on(backend.clone(), 0);
    let (messages, to_log) = collecting_sink();
    let mut streamed = client
        .run("r1")
        .get_streamed_log(StreamedLogOptions {
            to_log,
            from_start: true,
        })
        .await
        .unwrap()
        .expect("a streamed log");
    streamed.start().unwrap();
    assert!(streamed.start().is_err(), "already started");
    tokio::time::sleep(Duration::from_millis(100)).await;
    streamed.stop().await.unwrap();
    assert!(streamed.stop().await.is_err(), "not started");

    assert_eq!(
        *messages.lock().unwrap(),
        vec![
            "2024-01-01T00:00:00.000Z first\n  trace".to_string(),
            "2024-01-01T00:00:01.000Z second".to_string()
        ]
    );
    // A custom sink needs no names, so neither the run nor its Actor is requested.
    assert_eq!(backend.count(HttpMethod::Get, "/actors/"), 0);
    assert!(backend
        .requests()
        .iter()
        .any(|r| r.url.contains("/actor-runs/r1/log?stream=1&raw=1")));
}

/// The default redirection names the messages after the Actor and the run.
#[tokio::test]
async fn the_default_redirection_prefixes_the_messages_with_the_actor_and_the_run() {
    let client = client_on(log_backend(0), 0);
    let streamed = client
        .run("r1")
        .get_streamed_log(Default::default())
        .await
        .unwrap()
        .expect("a streamed log");
    assert!(
        format!("{streamed:?}").contains("my-actor runId:r1 -> "),
        "{streamed:?}"
    );

    let disabled = client
        .run("r1")
        .get_streamed_log(StreamedLogOptions {
            to_log: LogRedirect::Disabled,
            from_start: true,
        })
        .await
        .unwrap();
    assert!(disabled.is_none());
}

/// The log of a run that has not logged yet is an empty stream, which is reopened until the first
/// bytes arrive.
#[tokio::test(start_paused = true)]
async fn an_empty_log_stream_is_reopened() {
    let backend = log_backend(2);
    let client = client_on(backend.clone(), 0);
    let (messages, to_log) = collecting_sink();
    let mut streamed = client
        .run("r1")
        .get_streamed_log(StreamedLogOptions {
            to_log,
            from_start: true,
        })
        .await
        .unwrap()
        .unwrap();
    streamed.start().unwrap();
    tokio::time::sleep(Duration::from_secs(10)).await;
    streamed.stop().await.unwrap();

    let streams = backend.count(HttpMethod::Get, "stream=1");
    assert_eq!(streams, 3);
    assert_eq!(messages.lock().unwrap().len(), 2);
}

/// Stopping before the stream delivered anything reads the whole log in one request.
#[tokio::test(start_paused = true)]
async fn stopping_before_the_first_bytes_reads_the_whole_log() {
    let backend = log_backend(usize::MAX);
    let client = client_on(backend.clone(), 0);
    let (messages, to_log) = collecting_sink();
    let mut streamed = client
        .run("r1")
        .get_streamed_log(StreamedLogOptions {
            to_log,
            from_start: true,
        })
        .await
        .unwrap()
        .unwrap();
    streamed.start().unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(messages.lock().unwrap().is_empty());
    streamed.stop().await.unwrap();
    assert_eq!(messages.lock().unwrap().len(), 2);
    assert!(backend
        .requests()
        .iter()
        .any(|r| r.url.contains("/log?raw=1") && !r.url.contains("stream=1")));
}

#[tokio::test]
async fn from_start_false_drops_the_messages_stamped_before_the_call() {
    let client = client_on(log_backend(0), 0);
    let (messages, to_log) = collecting_sink();
    let mut streamed = client
        .run("r1")
        .get_streamed_log(StreamedLogOptions {
            to_log,
            from_start: false,
        })
        .await
        .unwrap()
        .unwrap();
    streamed.start().unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    streamed.stop().await.unwrap();
    assert!(messages.lock().unwrap().is_empty());
}

#[tokio::test]
async fn call_redirects_the_log_of_the_run_to_the_sink() {
    let backend = log_backend(0);
    let client = client_on(backend.clone(), 0);
    let (messages, log) = collecting_sink();
    let run = client
        .actor("a1")
        .call::<Value>(
            None,
            ActorCallOptions {
                log,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(run.status.as_deref(), Some("SUCCEEDED"));
    assert_eq!(messages.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn a_streamed_log_of_a_missing_run_is_none() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        404,
        br#"{"error":{"type":"record-not-found","message":"m"}}"#.to_vec(),
    )]);
    let client = client_with(backend, 0);
    assert!(client
        .run("r1")
        .get_streamed_log(Default::default())
        .await
        .unwrap()
        .is_none());
}
