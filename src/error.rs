//! Error types returned by the Apify client.

use serde::Deserialize;

/// The result type used throughout this crate.
pub type ApifyClientResult<T> = Result<T, ApifyClientError>;

/// Shape of the `error` object returned by the Apify API on failure.
///
/// The API encodes errors as `{ "error": { "type": "...", "message": "..." } }`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ApiErrorBody {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ApiErrorDetail {
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// An error response returned by the Apify API.
///
/// This is raised for HTTP requests that reach the API but return a non-success
/// status code. It mirrors the `ApifyApiError` of the reference clients and exposes
/// the parsed error `type`, the human-readable `message`, the HTTP `status_code`,
/// the number of the final `attempt`, and the request `http_method`/`path`.
#[derive(Debug, Clone)]
pub struct ApiError {
    /// HTTP status code of the error response.
    pub status_code: u16,
    /// The machine-readable error type returned by the API (e.g. `record-not-found`).
    pub error_type: Option<String>,
    /// Human-readable description of the error returned by the API.
    pub message: String,
    /// Number of the API call attempt that produced this error (1-based).
    pub attempt: u32,
    /// HTTP method of the API call (e.g. `GET`, `POST`).
    pub http_method: Option<String>,
    /// Full path of the API endpoint (URL excluding origin).
    pub path: Option<String>,
    /// Additional structured data provided by the API about the error, if any.
    pub data: Option<serde_json::Value>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Apify API error (status {}, type {}): {}",
            self.status_code,
            self.error_type.as_deref().unwrap_or("unknown"),
            self.message,
        )
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    /// `true` for HTTP 400 Bad Request, typically because the request failed validation.
    ///
    /// The reference clients throw a distinct `InvalidRequestError` subclass for this status;
    /// since Rust has no exception hierarchy to mirror, these `is_*` predicates are the
    /// idiomatic equivalent classification on the one [`ApiError`] type.
    pub fn is_invalid_request(&self) -> bool {
        self.status_code == 400
    }

    /// `true` for HTTP 401 Unauthorized: the token is missing or invalid.
    pub fn is_unauthorized(&self) -> bool {
        self.status_code == 401
    }

    /// `true` for HTTP 403 Forbidden: the token lacks permission for the operation.
    pub fn is_forbidden(&self) -> bool {
        self.status_code == 403
    }

    /// `true` for HTTP 404 Not Found.
    ///
    /// Most `get`-style methods already map a `404` to `None` rather than an error (see e.g.
    /// [`crate::clients::actor::ActorClient::get`]), so this is mainly useful for the methods
    /// that return other errors directly, e.g. [`crate::clients::actor::ActorClient::start`]
    /// with a nonexistent Actor ID.
    pub fn is_not_found(&self) -> bool {
        self.status_code == 404
    }

    /// `true` for HTTP 409 Conflict.
    pub fn is_conflict(&self) -> bool {
        self.status_code == 409
    }

    /// `true` for HTTP 429 Too Many Requests. The client already retries these internally (see
    /// [`crate::ApifyClientBuilder::max_retries`]), so this surfaces only once retries are
    /// exhausted.
    pub fn is_rate_limited(&self) -> bool {
        self.status_code == 429
    }

    /// `true` for an HTTP 5xx status. Like [`is_rate_limited`](Self::is_rate_limited), the
    /// client already retries these internally, so this surfaces only once retries are
    /// exhausted.
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500
    }
}

/// The top-level error type for all client operations.
#[derive(Debug, thiserror::Error)]
pub enum ApifyClientError {
    /// The API returned a non-success status code with a structured error body.
    ///
    /// Boxed to keep the overall `Result` size small (the error path is rare).
    #[error(transparent)]
    Api(Box<ApiError>),

    /// A network/transport-level error occurred (connection failure, timeout, etc.).
    #[error("HTTP transport error: {0}")]
    Http(String),

    /// The request timed out.
    #[error("Request timed out")]
    Timeout,

    /// Failed to serialize the request body or deserialize the response body.
    #[error("(De)serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// The response body could not be interpreted as expected.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// A required configuration value or argument was missing or invalid.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

impl From<ApiError> for ApifyClientError {
    fn from(err: ApiError) -> Self {
        ApifyClientError::Api(Box::new(err))
    }
}

impl ApifyClientError {
    /// Returns the underlying [`ApiError`] if this is an API error, otherwise `None`.
    pub fn as_api_error(&self) -> Option<&ApiError> {
        match self {
            ApifyClientError::Api(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the HTTP status code if this error originated from an API response.
    pub fn status_code(&self) -> Option<u16> {
        self.as_api_error().map(|e| e.status_code)
    }
}

impl From<reqwest::Error> for ApifyClientError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApifyClientError::Timeout
        } else {
            ApifyClientError::Http(err.to_string())
        }
    }
}
