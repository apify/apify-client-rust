//! Shared helpers for the integration test suite.
//!
//! The integration tests need an `APIFY_TOKEN`. The API URL is `APIFY_API_URL`, including the `/v2`
//! suffix, and defaults to `https://api.apify.com/v2`.
//!
//! The tests can run concurrently with each other and with other clients' tests on the same
//! account, so each one creates uniquely named resources and cleans them up.

#![allow(dead_code)]

pub mod mock_backend;

use apify_client::{ApifyClient, ApifyClientBuilder};

/// The API URL the tests target, mirroring the integration-test contract.
const DEFAULT_API_URL: &str = "https://api.apify.com/v2";

/// Builds an [`ApifyClientBuilder`] configured from the environment, for tests that customize the
/// client further.
///
/// Returns `None` (so the caller can skip) if `APIFY_TOKEN` is not set.
pub fn make_client_builder() -> Option<ApifyClientBuilder> {
    let token = std::env::var("APIFY_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())?;
    let api_url = std::env::var("APIFY_API_URL")
        .ok()
        .filter(|u| !u.is_empty());
    let base_url = resolve_base_url(api_url.as_deref());
    Some(ApifyClient::builder().token(token).base_url(base_url))
}

/// Builds an [`ApifyClient`] configured from the environment.
///
/// Returns `None` (so the caller can skip) if `APIFY_TOKEN` is not set.
pub fn make_client() -> Option<ApifyClient> {
    make_client_builder().map(ApifyClientBuilder::build)
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

/// Runs a cleanup action when dropped, also when the test panics, so a failed test doesn't leak
/// its resources. It runs to completion on the test's multi-thread Tokio runtime.
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

/// Number of times [`iter_contains_eventually`] rebuilds the iterator and re-scans while waiting
/// for a just-created resource to become visible in its collection LIST endpoint.
pub const ITER_RETRY_ATTEMPTS: usize = 32;

/// Delay between the attempts made by [`iter_contains_eventually`].
pub const ITER_RETRY_BACKOFF: std::time::Duration = std::time::Duration::from_secs(3);

/// Like [`iter_contains`], but tolerates replica lag. The list endpoints read from secondaries that
/// can trail the primary by up to 90 seconds, so a just-created resource may be missing from the
/// first scan.
///
/// It scans a fresh iterator from `make_iter` up to [`ITER_RETRY_ATTEMPTS`] times, sleeping
/// [`ITER_RETRY_BACKOFF`] between attempts, and returns `true` as soon as `pred` matches. The
/// retries add up to 93 seconds, which covers the staleness window.
pub async fn iter_contains_eventually<T, F, Mk>(mut make_iter: Mk, mut pred: F) -> bool
where
    Mk: FnMut() -> apify_client::ListIterator<T>,
    F: FnMut(&T) -> bool,
{
    for attempt in 0..ITER_RETRY_ATTEMPTS {
        if iter_contains(make_iter(), &mut pred).await {
            return true;
        }
        if attempt + 1 < ITER_RETRY_ATTEMPTS {
            tokio::time::sleep(ITER_RETRY_BACKOFF).await;
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

/// Length of the random suffix [`unique_actor_name`] appends, in hex characters.
const ACTOR_NAME_SUFFIX_LEN: usize = 12;

/// Longest human-readable head [`unique_actor_name`] keeps from its (sanitized) `prefix` arg.
const ACTOR_NAME_HEAD_CAP: usize = 8;

/// Generates a unique name that satisfies Apify's (stricter) Actor naming rules: starts with a
/// letter and contains no hyphens, unlike the names [`unique_name`] produces for other resources.
///
/// It is the alphanumeric part of `prefix`, capped at [`ACTOR_NAME_HEAD_CAP`] characters, plus a
/// random suffix of [`ACTOR_NAME_SUFFIX_LEN`] hex characters. Truncating a `unique_name` instead
/// could cut into its fixed `rust-test-` head and drop the `prefix`.
pub fn unique_actor_name(prefix: &str) -> String {
    let sanitized: String = prefix.chars().filter(char::is_ascii_alphanumeric).collect();
    let head = &sanitized[..sanitized.len().min(ACTOR_NAME_HEAD_CAP)];
    let uuid = uuid::Uuid::new_v4().simple().to_string();
    format!("a{head}{}", &uuid[..ACTOR_NAME_SUFFIX_LEN])
}
