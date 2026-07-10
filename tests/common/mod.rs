//! Shared helpers for the integration test suite.
//!
//! All integration tests require a valid `APIFY_TOKEN` for the test account. The API
//! base URL is taken from `APIFY_API_URL` (which includes the `/v2` suffix) and falls
//! back to `https://api.apify.com/v2`.
//!
//! Tests are designed to run concurrently — including against the same test account from
//! several language clients at once — so every test creates uniquely-named resources and
//! cleans them up afterwards.

#![allow(dead_code)]

use apify_client::ApifyClient;

/// The API URL the tests target, mirroring the integration-test contract.
const DEFAULT_API_URL: &str = "https://api.apify.com/v2";

/// Builds an [`ApifyClient`] configured from the environment.
///
/// Returns `None` (so the caller can skip) if `APIFY_TOKEN` is not set.
pub fn make_client() -> Option<ApifyClient> {
    let token = std::env::var("APIFY_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())?;
    let api_url = std::env::var("APIFY_API_URL")
        .ok()
        .filter(|u| !u.is_empty());
    let base_url = resolve_base_url(api_url.as_deref());
    Some(
        ApifyClient::builder()
            .token(token)
            .base_url(base_url)
            .build(),
    )
}

/// Resolves the client `base_url` from an optional `APIFY_API_URL`.
///
/// `APIFY_API_URL` includes the `/v2` suffix (per the integration-test contract) and falls
/// back to `https://api.apify.com/v2`. Since the client appends `/v2` itself, the suffix is
/// stripped here.
pub fn resolve_base_url(api_url: Option<&str>) -> String {
    let url = api_url.unwrap_or(DEFAULT_API_URL);
    url.trim_end_matches('/')
        .trim_end_matches("/v2")
        .to_string()
}

/// Returns a client or prints a skip notice and returns early from the test.
///
/// Usage: `let client = require_client!();`
#[macro_export]
macro_rules! require_client {
    () => {{
        match $crate::common::make_client() {
            Some(client) => client,
            None => {
                eprintln!("Skipping: APIFY_TOKEN is not set");
                return;
            }
        }
    }};
}

/// A panic-safe cleanup guard.
///
/// Holds a deferred cleanup action that runs when the guard is dropped — including when a
/// test panics partway through (a failed `assert!`/`expect`), which would otherwise leak the
/// created resource on the shared test account. The action is run to completion on the test's
/// own (multi-thread) Tokio runtime, so it works inside `#[tokio::test]` bodies.
///
/// Usage:
/// ```ignore
/// let store = client.key_value_stores().get_or_create(Some(&name)).await.unwrap();
/// let client2 = client.clone();
/// let id = store.id.clone();
/// let _guard = Cleanup::new(move || async move {
///     let _ = client2.key_value_store(&id).delete().await;
/// });
/// // ... test body; even if it panics, the store is deleted.
/// ```
pub struct Cleanup {
    action: Option<Box<dyn FnOnce() + Send>>,
}

impl Cleanup {
    /// Creates a guard from an async cleanup closure (a `FnOnce` returning a future).
    pub fn new<F, Fut>(action: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + 'static,
    {
        // Capture a handle to the test's own runtime. The cleanup must run on this runtime,
        // not a freshly-built one: the API client's reqwest connection pool is bound to this
        // runtime's reactor, and driving it from a foreign runtime deadlocks (the I/O never
        // gets polled). Re-entering the original (multi-thread) runtime via the handle lets the
        // deferred delete actually make progress. Tests therefore use the multi-thread flavor.
        let handle = tokio::runtime::Handle::current();
        Cleanup {
            action: Some(Box::new(move || {
                handle.block_on(action());
            })),
        }
    }
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Some(action) = self.action.take() {
            // Run on a separate OS thread: `Handle::block_on` cannot be called from within an
            // active runtime worker thread, which is where Drop runs during a `#[tokio::test]`.
            // The closure re-enters the original multi-thread runtime via the captured handle,
            // whose background workers poll the reqwest I/O the delete depends on.
            let _ = std::thread::spawn(action).join();
        }
    }
}

/// Upper bound on how many items an iteration test pulls while searching for a specific
/// just-created resource. Iteration tests sort newest-first, so the target is normally in the
/// first page; the cap only guards against an unbounded scan on a busy shared account.
pub const ITER_SEARCH_CAP: usize = 1000;

/// Drives a lazy [`ListIterator`](apify_client::ListIterator) looking for an item matching
/// `pred`, pulling at most [`ITER_SEARCH_CAP`] items. Returns `true` as soon as a match is
/// found. Used by the per-collection iteration tests to confirm a just-created resource is
/// reachable through the iterator (exercising the transparent page-fetching path).
pub async fn iter_contains<T, F>(mut iter: apify_client::ListIterator<T>, mut pred: F) -> bool
where
    F: FnMut(&T) -> bool,
{
    let mut pulled = 0usize;
    while let Some(item) = iter.next().await.expect("iteration should not error") {
        if pred(&item) {
            return true;
        }
        pulled += 1;
        if pulled >= ITER_SEARCH_CAP {
            break;
        }
    }
    false
}

/// Generates a unique, collision-resistant resource name for test isolation.
///
/// The name embeds the test-specific `prefix`, a random UUID fragment, and is kept short
/// enough for Apify's naming limits. Using a random component lets the same test run in
/// parallel (across processes and languages) without clobbering shared state.
pub fn unique_name(prefix: &str) -> String {
    let uuid = uuid::Uuid::new_v4().simple().to_string();
    format!("rust-test-{prefix}-{}", &uuid[..12])
}
