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
/// created resource on the shared test account. The action is run to completion on a fresh
/// current-thread Tokio runtime, so it works inside `#[tokio::test]` bodies.
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
        Cleanup {
            action: Some(Box::new(move || {
                // Run the async cleanup to completion on a dedicated current-thread runtime.
                // A new runtime avoids interfering with the test's own runtime during unwind.
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("build cleanup runtime");
                rt.block_on(action());
            })),
        }
    }
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Some(action) = self.action.take() {
            // Run on a separate OS thread: `block_on` cannot be called from within an active
            // runtime worker thread, which is where Drop runs during a `#[tokio::test]`.
            let _ = std::thread::spawn(action).join();
        }
    }
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
