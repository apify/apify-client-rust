//! Tests for client-level metadata that do not require network access.

mod common;

use apify_client::{ApifyClient, API_SPEC_VERSION, CLIENT_VERSION};

/// The exported version constants follow the expected formats.
#[test]
fn version_constants() {
    // Client version is semver-ish (starts with a digit).
    assert!(CLIENT_VERSION.chars().next().unwrap().is_ascii_digit());
    // Spec version matches the Apify `v2-...Z` format.
    assert!(API_SPEC_VERSION.starts_with("v2-"));
    assert!(API_SPEC_VERSION.ends_with('Z'));
}

/// The `User-Agent` header follows the mandated format.
#[test]
fn user_agent_format() {
    let client = ApifyClient::new("dummy-token");
    let ua = client.user_agent();
    assert!(ua.starts_with(&format!("ApifyClient/{CLIENT_VERSION} (")));
    assert!(ua.contains("Rust/"));
    assert!(ua.contains("isAtHome/"));
}
