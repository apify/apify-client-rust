//! A scripted [`HttpBackend`] for the offline tests: it returns a queued sequence of responses and
//! records what the client sent, so the tests need no network access or `APIFY_TOKEN`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use apify_client::http_client::{HttpBackend, HttpRequest, HttpResponse};
use apify_client::{ApifyClient, ApifyClientError};
use async_trait::async_trait;

/// A scripted backend that returns a queued sequence of responses and counts calls.
#[derive(Debug)]
pub struct MockBackend {
    responses: Mutex<Vec<MockOutcome>>,
    calls: AtomicUsize,
    last_url: Mutex<Option<String>>,
    /// Every request URL in call order, so tests can assert per-page pagination wiring.
    urls: Mutex<Vec<String>>,
    last_headers: Mutex<std::collections::HashMap<String, String>>,
    last_body: Mutex<Option<Vec<u8>>>,
    /// The `timeout` of every request in call order, so tests can assert the timeout tiers.
    timeouts: Mutex<Vec<Duration>>,
}

#[derive(Debug, Clone)]
pub enum MockOutcome {
    Status(u16, Vec<u8>),
    NetworkError,
    /// The transport gave up on the request because it outlived its timeout.
    Timeout,
}

impl MockBackend {
    pub fn new(responses: Vec<MockOutcome>) -> Arc<Self> {
        Arc::new(Self {
            responses: Mutex::new(responses),
            calls: AtomicUsize::new(0),
            last_url: Mutex::new(None),
            urls: Mutex::new(Vec::new()),
            last_headers: Mutex::new(std::collections::HashMap::new()),
            last_body: Mutex::new(None),
            timeouts: Mutex::new(Vec::new()),
        })
    }

    /// The timeout of every request in call order.
    pub fn timeouts(&self) -> Vec<Duration> {
        self.timeouts.lock().unwrap().clone()
    }

    /// The timeout of the last request.
    pub fn last_timeout(&self) -> Duration {
        *self.timeouts().last().expect("a request was sent")
    }

    pub fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn last_url(&self) -> Option<String> {
        self.last_url.lock().unwrap().clone()
    }

    /// All request URLs in call order.
    pub fn urls(&self) -> Vec<String> {
        self.urls.lock().unwrap().clone()
    }

    /// Case-insensitive lookup of the last request's header value.
    pub fn last_header(&self, name: &str) -> Option<String> {
        let headers = self.last_headers.lock().unwrap();
        headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    }

    pub fn last_body(&self) -> Option<Vec<u8>> {
        self.last_body.lock().unwrap().clone()
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
        self.timeouts.lock().unwrap().push(request.timeout);
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
            MockOutcome::Timeout => Err(ApifyClientError::Timeout),
        }
    }
}

pub fn client_with(backend: Arc<MockBackend>, max_retries: u32) -> ApifyClient {
    ApifyClient::builder()
        .token("test-token")
        .max_retries(max_retries)
        // Keep backoff tiny so tests run fast.
        .min_delay_between_retries(Duration::from_millis(1))
        .http_backend(backend)
        .build()
}
