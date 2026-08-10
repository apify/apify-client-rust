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
/// Returns `None` (so the caller can skip) if `APIFY_TOKEN` is not set. Reads the environment
/// once and delegates to [`make_client_from`], which takes no process state — see that function
/// for why this split exists.
pub fn make_client() -> Option<ApifyClient> {
    let token = std::env::var("APIFY_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())?;
    let api_url = std::env::var("APIFY_API_URL")
        .ok()
        .filter(|u| !u.is_empty());
    make_client_from(Some(token), api_url)
}

/// Builds an [`ApifyClient`] from an explicit token/API-URL pair, with no process-environment
/// reads at all.
///
/// This is the env-free core of [`make_client`], split out so tests that want to exercise the
/// `APIFY_API_URL` -> `base_url` resolution path don't have to mutate the real
/// `APIFY_TOKEN`/`APIFY_API_URL` process environment variables to do it. Those are read by
/// every other test in the suite via `require_client!`/`make_client`, and `#[tokio::test]`s run
/// concurrently within one process, so mutating them process-wide would race every other test
/// that happens to call `make_client` during the mutation window. Passing values in directly
/// sidesteps that race entirely rather than merely narrowing it.
///
/// Returns `None` if `token` is `None` or empty, mirroring `make_client`'s skip behavior.
pub fn make_client_from(token: Option<String>, api_url: Option<String>) -> Option<ApifyClient> {
    let token = token.filter(|t| !t.is_empty())?;
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

/// Number of times [`iter_contains_eventually`] rebuilds the iterator and re-scans while waiting
/// for a just-created resource to become visible in its collection LIST endpoint.
pub const ITER_RETRY_ATTEMPTS: usize = 16;

/// Delay between the attempts made by [`iter_contains_eventually`].
pub const ITER_RETRY_BACKOFF: std::time::Duration = std::time::Duration::from_millis(1000);

/// Like [`iter_contains`], but tolerant of eventual consistency in collection LIST endpoints.
///
/// A resource created through a write endpoint is not always immediately reflected in its
/// collection's LIST response — the write and the list index converge asynchronously on the
/// server. A create-then-iterate test that scans the collection exactly once therefore races that
/// convergence and flakes when the just-created entity has not yet propagated.
///
/// This helper rebuilds a fresh iterator via `make_iter` and re-scans it with [`iter_contains`] up
/// to [`ITER_RETRY_ATTEMPTS`] times, sleeping [`ITER_RETRY_BACKOFF`] between attempts, returning
/// `true` as soon as `pred` matches. When the entity is already visible it matches on the first
/// attempt and returns immediately with no sleeping — so it is a no-op in the common
/// already-consistent case and only pays the backoff on the rare lagging run.
///
/// Budget: `(ITER_RETRY_ATTEMPTS - 1) * ITER_RETRY_BACKOFF` = ~15s of retrying before giving up.
/// This is deliberately larger than a "couple of seconds": the only Apify propagation lag actually
/// measured in this suite is the dataset-items count settling at ~10s, and the collection LIST
/// index convergence time is not independently measured, so a ~2s budget could let the flake recur
/// at a lower (harder-to-diagnose) frequency. ~15s gives real headroom above the ~10s observation
/// while still failing fast enough on a genuinely-missing entity (a true bug). The cost lands only
/// on lagging or genuinely-failing runs; a consistent account never sleeps.
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

/// Number of trailing random hex characters kept by [`short_unique_name`]. Fixed so the random
/// component's collision-resistance doesn't shrink as callers pass longer prefixes.
const SHORT_NAME_RANDOM_LEN: usize = 10;

/// Generates a short, collision-resistant resource name for APIs with strict naming limits
/// (e.g. Actor/build names, which reject hyphens and cap length).
///
/// Naively taking `unique_name(prefix).replace('-', "")[..max_len]` truncates from the *end*,
/// which can cut off the random suffix entirely for long prefixes, leaving a constant name that
/// collides across concurrent test runs (see the `actor_webhooks_and_default_build`
/// regression this guards against). This instead truncates the (hyphen-stripped) `prefix`
/// *first*, so the trailing random fragment always survives.
///
/// `leading` is prepended as-is (used to satisfy naming rules that require a letter first);
/// `max_len` bounds the total length of `leading` + prefix + random suffix.
pub fn short_unique_name(leading: char, prefix: &str, max_len: usize) -> String {
    let uuid = uuid::Uuid::new_v4().simple().to_string();
    let random_suffix = &uuid[..SHORT_NAME_RANDOM_LEN];
    let clean_prefix: String = prefix.chars().filter(|c| c.is_alphanumeric()).collect();
    let prefix_budget = max_len.saturating_sub(1 + SHORT_NAME_RANDOM_LEN);
    let prefix_trunc = &clean_prefix[..clean_prefix.len().min(prefix_budget)];
    format!("{leading}{prefix_trunc}{random_suffix}")
}
