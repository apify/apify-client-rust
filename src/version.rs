//! Version constants exposed by the client.

/// Semantic version of this client crate.
///
/// This is sourced from the crate's `Cargo.toml` `version` field at compile time
/// and follows [Semantic Versioning](https://semver.org/).
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The version of the Apify OpenAPI specification that this client was generated
/// and verified against.
///
/// This corresponds to the `info.version` field of the Apify OpenAPI document.
pub const API_SPEC_VERSION: &str = "v2-2026-06-30T091455Z";
