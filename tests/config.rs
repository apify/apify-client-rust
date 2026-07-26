//! Offline tests for the integration-test harness configuration (no network needed).

mod common;

use common::resolve_base_url;

/// When `APIFY_API_URL` is unset, the base URL falls back to the documented default.
#[test]
fn base_url_falls_back_to_default() {
    assert_eq!(resolve_base_url(None), "https://api.apify.com");
}

/// `APIFY_API_URL` includes `/v2`; the harness strips it (the client re-adds it).
#[test]
fn base_url_strips_v2_suffix() {
    assert_eq!(
        resolve_base_url(Some("https://api.example.com/v2")),
        "https://api.example.com"
    );
    assert_eq!(
        resolve_base_url(Some("https://api.example.com/v2/")),
        "https://api.example.com"
    );
}

/// `make_client` honors the `APIFY_API_URL` -> `base_url` resolution end-to-end: the resolved
/// client's `api_base_url()` reflects the given value (with the harness `/v2` round-trip).
///
/// Exercises the full `make_client_from` path (the env-free core `make_client` delegates to),
/// not just the pure `resolve_base_url` helper. This deliberately does NOT mutate the real
/// `APIFY_TOKEN`/`APIFY_API_URL` process environment variables: those are read by every other
/// test in the suite via `require_client!`, and since `#[tokio::test]`s run concurrently within
/// one process, doing so would race any test that calls `make_client` during the mutation
/// window. Passing values straight to `make_client_from` exercises the identical resolution
/// logic with no process-global state and no race.
#[test]
fn make_client_honors_apify_api_url_env() {
    let client = common::make_client_from(
        Some("dummy-token-for-config-test".to_string()),
        Some("https://api.example.test/v2".to_string()),
    )
    .expect("make_client_from with a token set");
    assert_eq!(client.api_base_url(), "https://api.example.test/v2");
}
