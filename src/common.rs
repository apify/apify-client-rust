//! Building blocks shared by the resource clients: the response envelope, query parameters,
//! pagination types and the `User-Agent` builder.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::{ApifyClientError, ApifyClientResult, NOT_FOUND_STATUS_CODE};
use crate::version::CLIENT_VERSION;

/// The `{ "data": ... }` envelope around the payload of most endpoints.
#[derive(Debug, Deserialize)]
pub(crate) struct DataEnvelope<T> {
    pub data: T,
}

/// Parses a JSON response body that is wrapped in a `data` envelope.
pub(crate) fn parse_data_envelope<T: DeserializeOwned>(body: &[u8]) -> ApifyClientResult<T> {
    let envelope: DataEnvelope<T> = serde_json::from_slice(body)?;
    Ok(envelope.data)
}

/// Translates a "not found" API error into `Ok(None)` and returns any other error. A `get` or
/// `delete` of a missing resource thus resolves to `None`, whatever the error `type` of the `404`.
pub(crate) fn catch_not_found<T>(result: ApifyClientResult<T>) -> ApifyClientResult<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(err) if err.status_code() == Some(NOT_FOUND_STATUS_CODE) => Ok(None),
        Err(err) => Err(err),
    }
}

/// A collection of query parameters that serializes booleans as `0`/`1` and omits `None` values.
#[derive(Debug, Default, Clone)]
pub(crate) struct QueryParams {
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

    /// The raw key-value pairs.
    pub(crate) fn pairs_ref(&self) -> &[(String, String)] {
        &self.pairs
    }

    /// Adds an already-stringified pair.
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

/// Percent-encodes a single URL path segment, such as a record key or a request ID, so it can't
/// leave the segment or add a query string. Everything except the RFC 3986 unreserved characters
/// is escaped.
///
/// It does not reject an empty segment or a dot segment (`.` or `..`), which a URL parser resolves
/// to another endpoint even when percent-encoded. Methods that take such a value use
/// `to_path_segment` instead.
pub(crate) fn encode_path_segment(input: &str) -> String {
    // Path segments and query components share the unreserved set.
    url_encode(input)
}

/// Validates and percent-encodes a caller-supplied path segment. An empty segment, which would
/// address the collection, and `.` or `..`, which URL parsers resolve even when percent-encoded,
/// fail with [`ApifyClientError::InvalidArgument`].
pub(crate) fn to_path_segment(input: &str) -> ApifyClientResult<String> {
    if input.is_empty() || input == "." || input == ".." {
        return Err(ApifyClientError::InvalidArgument(format!(
            "URL path segment must be non-empty and must not be a dot segment, got `{input}`"
        )));
    }
    Ok(encode_path_segment(input))
}

/// Offset/limit pagination options of most list endpoints.
#[derive(Debug, Default, Clone)]
pub struct ListOptions {
    /// Number of items to skip from the beginning of the list.
    pub offset: Option<i64>,
    /// Maximum number of items to return.
    pub limit: Option<i64>,
    /// If `true`, items are returned newest-first.
    pub desc: Option<bool>,
}

/// Options of the storage collection list endpoints (datasets, key-value stores and request
/// queues): the offset/limit pagination plus `unnamed` and `ownership` filters.
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
    /// Number of entries on this page. On a page from
    /// [`DatasetClient::list_items`](crate::clients::dataset::DatasetClient::list_items), the
    /// number of dataset rows the API scanned for it, or the number of items returned when that
    /// is larger. Filters (`clean`, `skip_empty`, `skip_hidden`) can leave it above `items.len()`.
    /// The next page starts at `offset + min(count, limit)`; the cap matters only with `unwind`,
    /// which can return more items than the rows scanned.
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
pub(crate) fn build_user_agent(suffix: Option<&str>) -> String {
    // The OS token is the `os.platform()` value of Node.js, as in every Apify client.
    // `std::env::consts::OS` spells a few differently.
    let os = map_os_to_node_platform(std::env::consts::OS);
    // `isAtHome` depends only on `APIFY_IS_AT_HOME` and is `false` when it is missing.
    let is_at_home = if env_var_set("APIFY_IS_AT_HOME") {
        "true"
    } else {
        "false"
    };
    // Rust has no runtime-version API, so this is the compiler version `build.rs` captured, or
    // "unknown" if it could not run rustc.
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

/// Maps a Rust `std::env::consts::OS` value to the corresponding Node.js `os.platform()` token.
///
/// Rust and Node.js agree on most tokens, which pass through unchanged. The four that differ
/// (`macos`, `windows`, `solaris`, `illumos`) are translated to `darwin`, `win32`, `sunos` and
/// `sunos`. An unknown value passes through as it is.
fn map_os_to_node_platform(os: &str) -> &str {
    match os {
        "macos" => "darwin",
        "windows" => "win32",
        "solaris" => "sunos",
        "illumos" => "sunos",
        other => other,
    }
}

/// Encodes a resource id so it is safe to embed in a URL path. Apify uses the
/// `username~resourcename` form, so every `/` of an id is replaced with `~`.
pub(crate) fn to_safe_id(id: &str) -> String {
    id.replace('/', "~")
}

/// Version tag embedded in storage-content signatures: the default of `@apify/utilities`.
const STORAGE_CONTENT_SIGNATURE_VERSION: &str = "0";
/// Number of leading hex characters of the HMAC digest used by `create_hmac_signature`.
const HMAC_SIGNATURE_HEX_LEN: usize = 30;
/// base62 alphabet (`0-9a-zA-Z`, lowercase first) used to encode the truncated HMAC, in the order
/// `@apify/utilities` uses.
const BASE62_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Computes an Apify URL-signing signature, compatible with `createHmacSignature` of
/// `@apify/utilities`: the first 30 hex characters of `HMAC-SHA256(secret_key, message)`,
/// base62-encoded. It signs the record keys of public key-value-store URLs.
pub(crate) fn create_hmac_signature(secret_key: &str, message: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes())
        .expect("HMAC accepts a key of any length");
    mac.update(message.as_bytes());
    let digest = mac.finalize().into_bytes();

    // 30 hex characters are 120 bits, which fit in a `u128`.
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

/// Milliseconds in a second.
const MILLIS_PER_SECOND: i64 = 1000;

/// The longest validity of a signed public URL, in seconds (about 146 million years): half of what
/// fits in milliseconds in an `i64`, so the current time plus the validity can't overflow. It is
/// the upper bound of the `expires_in_secs` of the public URL methods.
pub const MAX_EXPIRES_IN_SECS: i64 = i64::MAX / MILLIS_PER_SECOND / 2;

/// Checks the validity of a signed public URL before any request is sent. `None` never expires.
/// A validity must be positive, since `0` or less signs a URL that has already expired, and at
/// most [`MAX_EXPIRES_IN_SECS`]. Fails with [`ApifyClientError::InvalidArgument`] otherwise.
pub(crate) fn validate_expires_in_secs(expires_in_secs: Option<i64>) -> ApifyClientResult<()> {
    match expires_in_secs {
        Some(secs) if secs <= 0 => Err(ApifyClientError::InvalidArgument(format!(
            "`expires_in_secs` must be positive, got {secs}; leave it out for a URL that does not expire"
        ))),
        Some(secs) if secs > MAX_EXPIRES_IN_SECS => Err(ApifyClientError::InvalidArgument(format!(
            "`expires_in_secs` must be at most {MAX_EXPIRES_IN_SECS}, got {secs}"
        ))),
        _ => Ok(()),
    }
}

/// The expiry of a signed public URL as an absolute time in milliseconds: now plus the validity,
/// or `0` for a URL that does not expire. Call it when the URL is signed, so the time of earlier
/// requests doesn't come off the validity. The validity must have passed
/// [`validate_expires_in_secs`].
pub(crate) fn public_url_expiry_millis(expires_in_secs: Option<i64>) -> i64 {
    match expires_in_secs {
        Some(secs) => chrono::Utc::now().timestamp_millis() + secs * MILLIS_PER_SECOND,
        None => 0,
    }
}

/// Builds a storage-content signature for the public URL of dataset items or a key list,
/// compatible with `createStorageContentSignature` of `@apify/utilities`.
///
/// It signs `"{version}.{expiresAtMillis}.{resourceId}"` with [`create_hmac_signature`] and returns
/// the unpadded base64url encoding of `"{version}.{expiresAtMillis}.{hmac}"`. `expiresAtMillis` is
/// what [`public_url_expiry_millis`] returns.
pub(crate) fn sign_storage_content(
    secret_key: &str,
    resource_id: &str,
    expires_at_millis: i64,
) -> String {
    use base64::Engine;

    let version = STORAGE_CONTENT_SIGNATURE_VERSION;
    let message = format!("{version}.{expires_at_millis}.{resource_id}");
    let hmac = create_hmac_signature(secret_key, &message);
    let envelope = format!("{version}.{expires_at_millis}.{hmac}");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(envelope.as_bytes())
}

#[cfg(test)]
mod user_agent_tests {
    use super::{build_user_agent, map_os_to_node_platform};

    // Only `APIFY_IS_AT_HOME` sets the flag, not a bare `isAtHome` variable. Environment variables
    // are process-global, so this test owns both names and restores them afterwards.
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

    // The OS token must be a documented Node.js `os.platform()` value. The per-platform mapping is
    // covered by `os_token_maps_rust_spellings_to_node_platform`.
    #[test]
    fn user_agent_os_token_matches_node_platform() {
        // The values Node's `os.platform()` can return (independent of our mapping code).
        const NODE_PLATFORMS: &[&str] = &[
            "aix", "android", "darwin", "freebsd", "haiku", "linux", "netbsd", "openbsd", "sunos",
            "win32", "cygwin",
        ];

        let ua = build_user_agent(None);
        // Extract the OS token exactly as it appears in the produced header — the first
        // component inside the parentheses: `(<os>; Rust/...)`.
        let os = ua
            .split_once('(')
            .and_then(|(_, rest)| rest.split_once(';'))
            .map(|(token, _)| token.trim())
            .expect("user agent must contain an `(<os>; Rust/...` section");
        assert!(
            NODE_PLATFORMS.contains(&os),
            "OS token must be a Node `os.platform()` value, got `{os}` in `{ua}`"
        );
        assert_eq!(
            os,
            os.to_ascii_lowercase(),
            "OS token must be lowercase, got `{os}`"
        );
        // Must never emit the Rust-native spellings or uname-style names.
        for forbidden in [
            "macos",
            "windows",
            "solaris",
            "Linux",
            "Darwin",
            "Windows_NT",
        ] {
            assert_ne!(
                os, forbidden,
                "OS token must be the Node `os.platform()` token, not `{forbidden}`"
            );
        }
    }

    // The four differing tokens are translated and the shared ones are left untouched.
    #[test]
    fn os_token_maps_rust_spellings_to_node_platform() {
        assert_eq!(map_os_to_node_platform("macos"), "darwin");
        assert_eq!(map_os_to_node_platform("windows"), "win32");
        assert_eq!(map_os_to_node_platform("solaris"), "sunos");
        assert_eq!(map_os_to_node_platform("illumos"), "sunos");
        // Tokens Rust and Node already share must pass through unchanged.
        for shared in ["linux", "android", "freebsd", "openbsd", "netbsd", "aix"] {
            assert_eq!(map_os_to_node_platform(shared), shared);
        }
    }

    // The `Rust/` token must be followed by a compiler version such as `1.94.1`, not "unknown".
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

    // Known answer of `createHmacSignature` in `@apify/utilities` for key "secret" and message
    // "message".
    #[test]
    fn hmac_signature_matches_the_known_answer() {
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
        let sig = sign_storage_content("secret", "RESID", 0);
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(sig)
            .expect("valid base64url");
        let decoded = String::from_utf8(decoded).expect("utf8");
        let expected_hmac = create_hmac_signature("secret", "0.0.RESID");
        assert_eq!(decoded, format!("0.0.{expected_hmac}"));
    }

    // A validity is positive and at most the maximum: `None` never expires and a positive one is
    // in the future, while `0`, a negative one and one beyond the maximum are refused.
    #[test]
    fn public_url_validity_is_validated_and_turned_into_an_expiry() {
        use super::{public_url_expiry_millis, validate_expires_in_secs, MAX_EXPIRES_IN_SECS};
        // The documented value.
        assert_eq!(MAX_EXPIRES_IN_SECS, 4_611_686_018_427_387);
        let now = chrono::Utc::now().timestamp_millis();

        assert_eq!(public_url_expiry_millis(None), 0);
        let expiry = public_url_expiry_millis(Some(60));
        assert!((now + 60_000..=now + 70_000).contains(&expiry), "{expiry}");
        // The longest validity still has an expiry that fits.
        assert!(public_url_expiry_millis(Some(MAX_EXPIRES_IN_SECS)) > now);

        for secs in [None, Some(1), Some(MAX_EXPIRES_IN_SECS)] {
            validate_expires_in_secs(secs).expect("accepted");
        }
        for secs in [0, -1, i64::MIN, MAX_EXPIRES_IN_SECS + 1, i64::MAX] {
            let error = validate_expires_in_secs(Some(secs)).expect_err("refused");
            assert!(
                matches!(error, crate::ApifyClientError::InvalidArgument(_)),
                "{secs}: {error:?}"
            );
        }
    }
}
