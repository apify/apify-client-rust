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
