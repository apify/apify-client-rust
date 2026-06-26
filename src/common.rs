//! Shared building blocks used across resource clients: the response envelope,
//! query-parameter helpers, pagination types and the `User-Agent` builder.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::ApifyClientResult;
use crate::version::CLIENT_VERSION;

/// Status code returned when a resource is not found.
const NOT_FOUND_STATUS_CODE: u16 = 404;
const RECORD_NOT_FOUND_TYPE: &str = "record-not-found";
const RECORD_OR_TOKEN_NOT_FOUND_TYPE: &str = "record-or-token-not-found";

/// Most Apify endpoints wrap their payload in a top-level `data` property.
/// This envelope unwraps `{ "data": ... }` into the inner type.
#[derive(Debug, Deserialize)]
pub(crate) struct DataEnvelope<T> {
    pub data: T,
}

/// Parses a JSON response body that is wrapped in a `data` envelope.
pub(crate) fn parse_data_envelope<T: DeserializeOwned>(body: &[u8]) -> ApifyClientResult<T> {
    let envelope: DataEnvelope<T> = serde_json::from_slice(body)?;
    Ok(envelope.data)
}

/// Translates a "not found" API error into `Ok(None)`, re-raising any other error.
///
/// This mirrors `catchNotFoundOrThrow` in the reference clients: a `get`/`delete` on a
/// missing resource resolves to `None` rather than raising.
pub(crate) fn catch_not_found<T>(result: ApifyClientResult<T>) -> ApifyClientResult<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(err) => {
            if let Some(api_error) = err.as_api_error() {
                let is_not_found_status = api_error.status_code == NOT_FOUND_STATUS_CODE;
                let is_not_found_type = matches!(
                    api_error.error_type.as_deref(),
                    Some(RECORD_NOT_FOUND_TYPE) | Some(RECORD_OR_TOKEN_NOT_FOUND_TYPE)
                ) || api_error.http_method.as_deref() == Some("HEAD");
                if is_not_found_status && is_not_found_type {
                    return Ok(None);
                }
            }
            Err(err)
        }
    }
}

/// A mutable collection of query parameters that serializes booleans as `0`/`1` and
/// omits `None` values, matching the Apify API conventions.
#[derive(Debug, Default, Clone)]
pub struct QueryParams {
    pairs: Vec<(String, String)>,
}

impl QueryParams {
    /// Creates an empty set of query parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a string parameter if `value` is `Some`.
    pub fn add_str(&mut self, key: &str, value: Option<impl Into<String>>) -> &mut Self {
        if let Some(value) = value {
            self.pairs.push((key.to_string(), value.into()));
        }
        self
    }

    /// Adds an integer parameter if `value` is `Some`.
    pub fn add_int(&mut self, key: &str, value: Option<i64>) -> &mut Self {
        if let Some(value) = value {
            self.pairs.push((key.to_string(), value.to_string()));
        }
        self
    }

    /// Adds a floating-point parameter if `value` is `Some`.
    pub fn add_float(&mut self, key: &str, value: Option<f64>) -> &mut Self {
        if let Some(value) = value {
            self.pairs.push((key.to_string(), value.to_string()));
        }
        self
    }

    /// Adds a boolean parameter, encoded as `1`/`0`, if `value` is `Some`.
    pub fn add_bool(&mut self, key: &str, value: Option<bool>) -> &mut Self {
        if let Some(value) = value {
            self.pairs
                .push((key.to_string(), if value { "1" } else { "0" }.to_string()));
        }
        self
    }

    /// Adds a comma-joined list parameter if `value` is `Some` and non-empty.
    pub fn add_csv(&mut self, key: &str, value: Option<&[String]>) -> &mut Self {
        if let Some(values) = value {
            if !values.is_empty() {
                self.pairs.push((key.to_string(), values.join(",")));
            }
        }
        self
    }

    /// Returns `true` if no parameters were added.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Internal: read access to the raw pairs (used when merging parent params).
    pub(crate) fn pairs_ref(&self) -> &[(String, String)] {
        &self.pairs
    }

    /// Internal: push an already-stringified key/value pair.
    pub(crate) fn push_raw(&mut self, key: String, value: String) {
        self.pairs.push((key, value));
    }

    /// Appends these parameters as a query string to `url`.
    pub fn apply_to_url(&self, url: &str) -> String {
        if self.pairs.is_empty() {
            return url.to_string();
        }
        let encoded: Vec<String> = self
            .pairs
            .iter()
            .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
            .collect();
        let separator = if url.contains('?') { '&' } else { '?' };
        format!("{url}{separator}{}", encoded.join("&"))
    }
}

/// Percent-encodes a query-string component.
fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Percent-encodes a single URL *path segment*, so that values interpolated into the path
/// (such as key-value-store record keys or request IDs) cannot break out of the segment or
/// inject a query string.
///
/// Encodes everything except the RFC 3986 "unreserved" characters. Notably `/`, `?`, `#`,
/// space and any non-ASCII bytes are escaped — unlike a raw `format!(".../{key}")`, which
/// would leave them intact and produce a malformed or wrong-endpoint URL.
pub fn encode_path_segment(input: &str) -> String {
    // Path segments and query components share the same unreserved set here, so reuse the
    // query encoder. (`url_encode` already escapes `/`, `?`, `#`, space and non-ASCII.)
    url_encode(input)
}

/// Standard offset/limit pagination options shared by most list endpoints.
#[derive(Debug, Default, Clone)]
pub struct ListOptions {
    /// Number of items to skip from the beginning of the list.
    pub offset: Option<i64>,
    /// Maximum number of items to return.
    pub limit: Option<i64>,
    /// If `true`, items are returned newest-first.
    pub desc: Option<bool>,
}

/// Options shared by the storage collection list endpoints (`GET /v2/datasets`,
/// `/v2/key-value-stores`, `/v2/request-queues`), which add `unnamed` and `ownership`
/// filters on top of the standard offset/limit pagination.
#[derive(Debug, Default, Clone)]
pub struct StorageListOptions {
    /// Number of items to skip from the beginning of the list.
    pub offset: Option<i64>,
    /// Maximum number of items to return.
    pub limit: Option<i64>,
    /// If `true`, items are returned newest-first.
    pub desc: Option<bool>,
    /// If `true`, include unnamed storages in the result.
    pub unnamed: Option<bool>,
    /// Filter by ownership (e.g. `OWNED` / `ACCESSIBLE`).
    pub ownership: Option<String>,
}

impl StorageListOptions {
    /// Serializes these options into query parameters.
    pub(crate) fn apply(&self, params: &mut QueryParams) {
        params
            .add_int("offset", self.offset)
            .add_int("limit", self.limit)
            .add_bool("desc", self.desc)
            .add_bool("unnamed", self.unnamed)
            .add_str("ownership", self.ownership.clone());
    }
}

/// A single page of an offset/limit-paginated list.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationList<T> {
    /// Total number of items available across all pages.
    #[serde(default)]
    pub total: i64,
    /// Number of items skipped at the start.
    #[serde(default)]
    pub offset: i64,
    /// Maximum number of items the API would return for this request.
    #[serde(default)]
    pub limit: i64,
    /// Number of items actually returned in this page.
    #[serde(default)]
    pub count: i64,
    /// Whether the items are in descending order.
    #[serde(default)]
    pub desc: bool,
    /// The items of this page.
    #[serde(default = "Vec::new")]
    pub items: Vec<T>,
}

/// Reports whether the environment variable `name` is set to a non-empty value.
fn env_var_set(name: &str) -> bool {
    matches!(std::env::var(name), Ok(value) if !value.is_empty())
}

/// Builds the `User-Agent` header value mandated by the client requirements:
/// `ApifyClient/{version} ({os}; {language version}); isAtHome/{isAtHome}`.
pub fn build_user_agent(suffix: Option<&str>) -> String {
    let os = std::env::consts::OS;
    // The `isAtHome` flag signals whether the client runs on the Apify platform. Per the
    // requirements it is `true`/`false` based solely on the `APIFY_IS_AT_HOME` environment
    // variable (`false` when the variable is missing), matching the JS reference, which reads
    // only `APIFY_IS_AT_HOME` and renders a lowercase boolean.
    let is_at_home = if env_var_set("APIFY_IS_AT_HOME") {
        "true"
    } else {
        "false"
    };
    // Rust has no stable runtime-version API, so report the compiler version captured at build
    // time by `build.rs` (the closest analogue to a "language/runtime version"). Falls back to
    // "unknown" only if the build script could not invoke rustc.
    let rust_version = option_env!("BUILD_RUSTC_VERSION").unwrap_or("unknown");
    let mut ua =
        format!("ApifyClient/{CLIENT_VERSION} ({os}; Rust/{rust_version}); isAtHome/{is_at_home}");
    if let Some(suffix) = suffix {
        if !suffix.is_empty() {
            ua.push_str("; ");
            ua.push_str(suffix);
        }
    }
    ua
}

/// Encodes a resource id so it is safe to embed in a URL path. Apify uses the
/// `username~resourcename` form, so the first `/` of an id is replaced with `~`.
pub fn to_safe_id(id: &str) -> String {
    id.replacen('/', "~", 1)
}

/// Version tag embedded in storage-content signatures. Matches the upstream library default
/// (`0`), which the reference clients rely on by not passing an explicit version.
const STORAGE_CONTENT_SIGNATURE_VERSION: &str = "0";
/// Number of leading hex characters of the HMAC digest used by `create_hmac_signature`.
const HMAC_SIGNATURE_HEX_LEN: usize = 30;
/// base62 alphabet (`0-9a-zA-Z`, lowercase first) used to encode the truncated HMAC. This
/// ordering matches upstream `@apify/utilities`.
const BASE62_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Computes an Apify URL-signing signature, byte-for-byte compatible with the platform's
/// `@apify/utilities` `createHmacSignature`.
///
/// The algorithm is: `HMAC-SHA256(secret_key, message)` as lowercase hex, take the first 30
/// hex characters, interpret them as a big integer, then base62-encode (alphabet `0-9a-zA-Z`).
/// Used to sign key-value-store record keys for public access.
pub fn create_hmac_signature(secret_key: &str, message: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes())
        .expect("HMAC accepts a key of any length");
    mac.update(message.as_bytes());
    let digest = mac.finalize().into_bytes();

    // Hex-encode and keep the leading 30 hex chars (120 bits → fits in u128).
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        hex.push_str(&format!("{byte:02x}"));
    }
    let truncated = &hex[..HMAC_SIGNATURE_HEX_LEN];
    let value = u128::from_str_radix(truncated, 16).expect("30 hex chars always parse into a u128");
    to_base62(value)
}

/// Encodes a non-negative integer in base62 using the `0-9a-zA-Z` alphabet.
fn to_base62(mut value: u128) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let base = BASE62_ALPHABET.len() as u128;
    let mut digits = Vec::new();
    while value > 0 {
        let rem = (value % base) as usize;
        digits.push(BASE62_ALPHABET[rem]);
        value /= base;
    }
    digits.reverse();
    String::from_utf8(digits).expect("base62 alphabet is valid ASCII")
}

/// Builds a storage-content signature for a resource's public URL, byte-for-byte compatible
/// with the platform's `@apify/utilities` `createStorageContentSignature`.
///
/// It signs the message `"{version}.{expiresAtMillis}.{resourceId}"` (where `expiresAtMillis`
/// is the absolute expiry in milliseconds, or `0` for a non-expiring URL) with
/// [`create_hmac_signature`], then returns the base64url (no padding) encoding of
/// `"{version}.{expiresAtMillis}.{hmac}"`. Used for dataset-items and key-list public URLs.
pub fn sign_storage_content(
    secret_key: &str,
    resource_id: &str,
    expires_in_secs: Option<i64>,
) -> String {
    use base64::Engine;

    let expires_at_millis = match expires_in_secs {
        Some(secs) => chrono::Utc::now().timestamp_millis() + secs * 1000,
        None => 0,
    };
    let version = STORAGE_CONTENT_SIGNATURE_VERSION;
    let message = format!("{version}.{expires_at_millis}.{resource_id}");
    let hmac = create_hmac_signature(secret_key, &message);
    let envelope = format!("{version}.{expires_at_millis}.{hmac}");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(envelope.as_bytes())
}

#[cfg(test)]
mod user_agent_tests {
    use super::build_user_agent;

    // `build_user_agent` keys the flag solely on the `APIFY_IS_AT_HOME` env var (per the
    // requirements and the JS reference). A bare `isAtHome` env var must NOT affect it. The value
    // is the lowercase `true`/`false` the JS reference emits. Env vars are process-global, so this
    // test owns both names for its duration and restores them afterwards.
    #[test]
    fn is_at_home_reads_only_apify_is_at_home() {
        let prev_apify = std::env::var("APIFY_IS_AT_HOME").ok();
        let prev_literal = std::env::var("isAtHome").ok();

        // Neither set -> false.
        std::env::remove_var("APIFY_IS_AT_HOME");
        std::env::remove_var("isAtHome");
        assert!(
            build_user_agent(None).contains("isAtHome/false"),
            "no env var set must render isAtHome/false"
        );

        // A bare `isAtHome` env var must NOT flip the flag (it is not the mandated variable).
        std::env::set_var("isAtHome", "1");
        assert!(
            build_user_agent(None).contains("isAtHome/false"),
            "bare `isAtHome` env var must not affect the flag"
        );

        // Only `APIFY_IS_AT_HOME` drives the flag.
        std::env::remove_var("isAtHome");
        std::env::set_var("APIFY_IS_AT_HOME", "1");
        assert!(
            build_user_agent(None).contains("isAtHome/true"),
            "APIFY_IS_AT_HOME must drive the flag"
        );

        // Restore prior environment.
        match prev_apify {
            Some(v) => std::env::set_var("APIFY_IS_AT_HOME", v),
            None => std::env::remove_var("APIFY_IS_AT_HOME"),
        }
        match prev_literal {
            Some(v) => std::env::set_var("isAtHome", v),
            None => std::env::remove_var("isAtHome"),
        }
    }

    // The `{language version}` segment must carry a real compiler version captured by build.rs,
    // not the literal "unknown" the MSRV-based source used to produce. We assert the `Rust/`
    // token is followed by a digit (e.g. `Rust/1.94.1`).
    #[test]
    fn user_agent_reports_real_rust_version() {
        let ua = build_user_agent(None);
        let after = ua
            .split("Rust/")
            .nth(1)
            .expect("user agent must contain a Rust/ token");
        let version = after.split([')', ';']).next().unwrap_or("");
        assert!(
            version.chars().next().is_some_and(|c| c.is_ascii_digit()),
            "Rust version must be a real compiler version, got `Rust/{version}`"
        );
    }
}

#[cfg(test)]
mod signature_tests {
    use super::{create_hmac_signature, sign_storage_content, to_base62};

    // Known-answer test pinning the `createHmacSignature` algorithm against upstream
    // `@apify/utilities`: HMAC-SHA256 → hex → first 30 hex chars → big integer → base62 with
    // the lowercase-first alphabet `0-9a-zA-Z`. For key="secret", msg="message" the upstream
    // output is `11GYWmGxviysIBMtnQHBk`.
    #[test]
    fn hmac_signature_matches_upstream_scheme() {
        let sig = create_hmac_signature("secret", "message");
        assert_eq!(sig, "11GYWmGxviysIBMtnQHBk");
        // base62 output uses only `0-9a-zA-Z`.
        assert!(sig.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn base62_encoding() {
        assert_eq!(to_base62(0), "0");
        // With the lowercase-first alphabet, 61 maps to the last char `Z`.
        assert_eq!(to_base62(61), "Z");
        assert_eq!(to_base62(62), "10");
    }

    // A non-expiring storage-content signature uses version `0` and `expiresAt = 0`, and is
    // the base64url (no-pad) encoding of `"0.0.<hmac>"` where the hmac is over `"0.0.RESID"`.
    #[test]
    fn storage_content_signature_non_expiring_envelope() {
        use base64::Engine;
        let sig = sign_storage_content("secret", "RESID", None);
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(sig)
            .expect("valid base64url");
        let decoded = String::from_utf8(decoded).expect("utf8");
        let expected_hmac = create_hmac_signature("secret", "0.0.RESID");
        assert_eq!(decoded, format!("0.0.{expected_hmac}"));
    }
}
