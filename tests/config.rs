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

/// `make_client` honors the `APIFY_API_URL` environment variable end-to-end: the resolved
/// client's `api_base_url()` reflects the env value (with the harness `/v2` round-trip).
///
/// Exercises the actual env-var read path in `make_client`, not just the pure helper. Env
/// vars are process-global, so this test owns `APIFY_TOKEN`/`APIFY_API_URL` for its duration
/// and restores them afterwards.
#[test]
fn make_client_honors_apify_api_url_env() {
    let prev_token = std::env::var("APIFY_TOKEN").ok();
    let prev_url = std::env::var("APIFY_API_URL").ok();

    std::env::set_var("APIFY_TOKEN", "dummy-token-for-config-test");
    std::env::set_var("APIFY_API_URL", "https://api.example.test/v2");

    let client = common::make_client().expect("make_client with a token set");
    assert_eq!(client.api_base_url(), "https://api.example.test/v2");

    match prev_token {
        Some(v) => std::env::set_var("APIFY_TOKEN", v),
        None => std::env::remove_var("APIFY_TOKEN"),
    }
    match prev_url {
        Some(v) => std::env::set_var("APIFY_API_URL", v),
        None => std::env::remove_var("APIFY_API_URL"),
    }
}
