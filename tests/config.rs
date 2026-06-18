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
