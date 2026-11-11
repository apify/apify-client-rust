//! Client for the server interface of an Actor (`/v2/actors/{actorId}` and the Actor's `standbyUrl`).
//!
//! A Server Actor (formerly Actor Standby) runs a web server. It is called over HTTP, and the
//! platform keeps a run ready to answer, so each call skips the run startup.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;
use tokio::sync::OnceCell;

use crate::clients::base::ResourceContext;
use crate::clients::build::BuildClient;
use crate::clients::build_collection::BuildCollectionClient;
use crate::common::{parse_data_envelope, ListOptions, QueryParams};
use crate::error::{ApifyClientError, ApifyClientResult};
use crate::http_client::{
    AuthorizationHeader, BodyCompression, CallOptions, HttpClient, HttpMethod, HttpResponse,
    StreamingHttpResponse,
};
use crate::http_compressors::{HttpCompressor, RequestCompression};
use crate::models::Actor;
use crate::timeouts::Timeout;

/// Statuses a server request is retried on. The platform answers with them when it can't pass a
/// request to a server run, so a retry usually doesn't run the Actor's code again. An Actor that
/// returns one itself gets the request again.
const SERVER_RETRY_STATUS_CODES: &[u16] = &[429, 503];

/// Header that carries the ID of a request the platform forwards to a Server Actor.
const ACTOR_REQUEST_ID_HEADER: &str = "X-Actor-Request-ID";

/// The tag of the build that serves the server interface when the Actor's `actorStandby.build`
/// names none.
const DEFAULT_SERVER_BUILD: &str = "latest";

/// The body of a request to an Actor's server.
#[derive(Debug, Clone)]
pub enum ActorServerBody {
    /// A JSON value, sent serialized with `Content-Type: application/json` unless the request
    /// sets a content type.
    Json(Value),
    /// Text, sent as it is. Set the `Content-Type` header of the request to describe it.
    Text(String),
    /// Binary data, sent as it is. Set the `Content-Type` header of the request to describe it.
    Bytes(Vec<u8>),
}

impl ActorServerBody {
    /// Serializes `value` into a [`Json`](Self::Json) body.
    pub fn json<T: Serialize>(value: &T) -> ApifyClientResult<Self> {
        Ok(Self::Json(serde_json::to_value(value)?))
    }
}

/// How the body of a request to an Actor's server is compressed.
#[derive(Debug, Clone)]
pub enum ActorServerCompression {
    /// One of the built-in algorithms.
    Algorithm(RequestCompression),
    /// A custom compressor.
    Custom(Arc<dyn HttpCompressor>),
}

/// Options of [`ActorServerClient::request`] and [`ActorServerClient::request_stream`].
#[derive(Debug, Default, Clone)]
pub struct ActorServerRequestOptions {
    /// HTTP method of the request. Default is `GET`.
    pub method: Option<HttpMethod>,
    /// Query parameters of the request, in order. A key can repeat.
    pub params: Vec<(String, String)>,
    /// Additional headers of the request. They reach the Actor's server as they are,
    /// `Authorization` included.
    pub headers: HashMap<String, String>,
    /// The request body.
    pub body: Option<ActorServerBody>,
    /// The `X-Actor-Request-ID` header of the request your Server Actor is handling. It is sent on
    /// as the same header, replacing one in `headers`, so the called Actor charges at that
    /// request's pricing tier.
    pub parent_request_id: Option<String>,
    /// Compression of the request body. The body is sent uncompressed when it's `None`, since the
    /// Actor's server may not decode a compressed body. Set it only if the Actor's server decodes
    /// the resulting `Content-Encoding`. The body is compressed under the same conditions as for
    /// API calls: when it is large enough, no `Content-Encoding` header is set and its content
    /// type isn't already compressed.
    pub compression: Option<ActorServerCompression>,
}

/// A request to the server, ready to be sent.
struct PreparedServerRequest {
    method: HttpMethod,
    url: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    timeout: std::time::Duration,
    call_options: CallOptions,
}

/// Client for the server interface of a specific Actor.
///
/// Provides methods to work with the web server of a Server Actor, such as sending it requests or
/// getting its OpenAPI specification. Get an instance with
/// [`ActorClient::server`](crate::clients::actor::ActorClient::server).
///
/// ```no_run
/// use apify_client::clients::actor_server::ActorServerRequestOptions;
/// use apify_client::ApifyClient;
///
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ApifyClient::new("my-api-token");
/// let server = client.actor("username/actor-name").server();
///
/// let response = server
///     .request(
///         "/search",
///         ActorServerRequestOptions {
///             params: vec![("q".to_string(), "apify".to_string())],
///             ..Default::default()
///         },
///     )
///     .await?;
/// println!("{}", response.text());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ActorServerClient {
    ctx: ResourceContext,
    base_url: String,
    id: String,
    /// The server URL. Getting it takes an API call for the Actor, so it is cached for
    /// [`get_url`](Self::get_url) and [`request`](Self::request). Concurrent first calls share one
    /// API request, and a failed lookup is not kept.
    server_url: Arc<OnceCell<String>>,
}

crate::clients::base::impl_with_timeout!(ActorServerClient);

impl ActorServerClient {
    pub(crate) fn new(
        http: HttpClient,
        base_url: &str,
        id: &str,
        server_url: Arc<OnceCell<String>>,
    ) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "actors", id),
            base_url: base_url.to_string(),
            id: id.to_string(),
            server_url,
        }
    }

    /// Returns the URL of the Actor's server.
    ///
    /// The URL is the Actor's `standbyUrl`. The first call of this method or of
    /// [`get_open_api_spec`](Self::get_open_api_spec) fetches it, and the clients of the same
    /// [`ActorClient`](crate::clients::actor::ActorClient) reuse it. Use it to call the server with
    /// another HTTP client, for example one generated from the server's OpenAPI specification.
    ///
    /// Fails with [`ApifyClientError::InvalidArgument`] when the Actor is not a Server Actor, and
    /// with a `404` [`ApiError`](crate::ApiError) when the Actor does not exist.
    pub async fn get_url(&self) -> ApifyClientResult<String> {
        self.server_url
            .get_or_try_init(|| async { Ok(self.fetch_server_actor().await?.1) })
            .await
            .cloned()
    }

    /// Returns the OpenAPI specification of the Actor's server, or `None` if the Actor's
    /// definition doesn't include one.
    ///
    /// It is the web server schema in the definition of the build that serves the server
    /// interface: the one set in `actorStandby.build`, or `latest`. Use it to generate an HTTP
    /// client for the server, and point that client at [`get_url`](Self::get_url).
    ///
    /// Fails with [`ApifyClientError::InvalidArgument`] when the Actor is not a Server Actor, or
    /// the build that serves its server interface doesn't exist, and with a `404`
    /// [`ApiError`](crate::ApiError) when the Actor does not exist.
    pub async fn get_open_api_spec(&self) -> ApifyClientResult<Option<Value>> {
        let (actor, server_url) = self.fetch_server_actor().await?;
        // Saves a later `get_url` and `request` from fetching the Actor again.
        let _ = self.server_url.set(server_url);
        let build_ref = actor
            .actor_standby
            .as_ref()
            .and_then(|standby| standby.build.as_deref())
            .filter(|build| !build.is_empty())
            .unwrap_or(DEFAULT_SERVER_BUILD);

        let mut build_id = actor
            .tagged_builds
            .as_ref()
            .and_then(|tags| tags.get(build_ref))
            .and_then(|info| info.as_ref())
            .and_then(|info| info.build_id.clone());
        if build_id.is_none() {
            if let Some(target) = parse_build_number(build_ref) {
                // Not a tag, so look `build_ref` up as a build number, which the API can't
                // resolve directly.
                build_id = self.find_build_by_number(build_ref, target).await?;
            }
        }

        let build = match build_id {
            Some(id) => self.build_client(&id).get().await?,
            None => None,
        };
        let build = build.ok_or_else(|| {
            ApifyClientError::InvalidArgument(format!(
                "Build {build_ref} of Actor {} does not exist.",
                self.id
            ))
        })?;

        Ok(build
            .extra
            .get("actorDefinition")
            .and_then(|definition| definition.get("webServerSchema"))
            .filter(|schema| !schema.is_null())
            .cloned())
    }

    /// Sends an HTTP request to the Actor's server and buffers the response.
    ///
    /// The request goes to the URL of [`get_url`](Self::get_url). The client's token is sent in the
    /// `X-Apify-Authorization` header, which the platform consumes, so it never reaches the Actor's
    /// code. Headers in the options, `Authorization` included, reach the server as they are. The
    /// first request to an idle Actor waits for a server run to start, so the timeout defaults to
    /// the `Long` tier.
    ///
    /// `path` is the path on the server, for example `/search`. It is resolved against the server
    /// URL, so the request and the token can't go to another host. Pass the query in
    /// [`ActorServerRequestOptions::params`]. A fragment (`#...`) is dropped.
    ///
    /// The method returns any response of the server, error statuses included, so check
    /// [`HttpResponse::status`] before you use the body. `429` and `503` responses are retried with
    /// the client's retry settings, and the last one is returned when the retries run out. The
    /// Actor's code can return these statuses too, and then it gets the request again. Network
    /// errors and timeouts are not retried, since the Actor may have handled the request already. A
    /// redirect is returned as it is, since following it would send the token to a host the
    /// Actor's code picked.
    ///
    /// It fails only when it gets no response: with [`ApifyClientError::InvalidArgument`] when the
    /// Actor is not a Server Actor, with a `404` [`ApiError`](crate::ApiError) when the Actor does
    /// not exist, and with the transport error when the request fails or times out.
    pub async fn request(
        &self,
        path: &str,
        options: ActorServerRequestOptions,
    ) -> ApifyClientResult<HttpResponse> {
        let request = self.prepare_request(path, options).await?;
        self.ctx
            .send_with_options(
                request.method,
                request.url,
                request.headers,
                request.body,
                request.timeout,
                request.call_options,
            )
            .await
    }

    /// Like [`request`](Self::request), but returns the response body as a stream instead of
    /// buffering it.
    ///
    /// The response has whatever status the server answers, and is retried like
    /// [`request`](Self::request) until it starts, but not once the body is being read. The stream
    /// holds its connection until you read it to the end or drop it. The timeout bounds only the
    /// wait for the response to start, so a server that stays silent between chunks, such as one
    /// that sends events, is not cut off.
    pub async fn request_stream(
        &self,
        path: &str,
        options: ActorServerRequestOptions,
    ) -> ApifyClientResult<StreamingHttpResponse> {
        let request = self.prepare_request(path, options).await?;
        self.ctx
            .send_streaming_with_options(
                request.method,
                request.url,
                request.headers,
                request.body,
                request.timeout,
                &request.call_options,
            )
            .await
    }

    /// Resolves the URL and serializes the body and the query of a request to the server.
    async fn prepare_request(
        &self,
        path: &str,
        options: ActorServerRequestOptions,
    ) -> ApifyClientResult<PreparedServerRequest> {
        let server_url = self.get_url().await?;
        // A fragment is never sent to the server, and a leading `/` would only be doubled.
        let server_path = path
            .split('#')
            .next()
            .unwrap_or_default()
            .trim_start_matches('/');
        let mut params = QueryParams::new();
        for (key, value) in options.params {
            params.push_raw(key, value);
        }
        let url = params.apply_to_url(&format!(
            "{}/{server_path}",
            server_url.trim_end_matches('/')
        ));

        let mut headers = options.headers;
        if let Some(parent_request_id) = options.parent_request_id {
            headers.retain(|name, _| !name.eq_ignore_ascii_case(ACTOR_REQUEST_ID_HEADER));
            headers.insert(ACTOR_REQUEST_ID_HEADER.to_string(), parent_request_id);
        }
        let body = match options.body {
            None => None,
            Some(ActorServerBody::Json(value)) => {
                if !headers
                    .keys()
                    .any(|name| name.eq_ignore_ascii_case("Content-Type"))
                {
                    headers.insert("Content-Type".to_string(), "application/json".to_string());
                }
                Some(serde_json::to_vec(&value)?)
            }
            Some(ActorServerBody::Text(text)) => Some(text.into_bytes()),
            Some(ActorServerBody::Bytes(bytes)) => Some(bytes),
        };

        let compression = match options.compression {
            None => BodyCompression::Disabled,
            Some(ActorServerCompression::Algorithm(algorithm)) => {
                BodyCompression::With(algorithm.into_compressor())
            }
            Some(ActorServerCompression::Custom(compressor)) => BodyCompression::With(compressor),
        };
        Ok(PreparedServerRequest {
            method: options.method.unwrap_or(HttpMethod::Get),
            url,
            headers,
            body,
            timeout: self.ctx.timeout(Timeout::Long),
            call_options: CallOptions {
                compression,
                authorization_header: AuthorizationHeader::XApifyAuthorization,
                return_error_responses: true,
                retry_status_codes: Some(SERVER_RETRY_STATUS_CODES),
                do_not_retry_transport_errors: true,
                do_not_follow_redirects: true,
                no_stream_inactivity_timeout: true,
                ..CallOptions::default()
            },
        })
    }

    /// Fetches the Actor and checks that it is a Server Actor. Returns the Actor and its server
    /// URL.
    async fn fetch_server_actor(&self) -> ApifyClientResult<(Actor, String)> {
        let response = self
            .ctx
            .send(
                HttpMethod::Get,
                self.ctx.url(None),
                self.ctx.timeout(Timeout::Short),
            )
            .await?;
        let actor: Actor = parse_data_envelope(&response.body)?;
        match actor.standby_url.clone().filter(|url| !url.is_empty()) {
            Some(url) => Ok((actor, url)),
            None => Err(ApifyClientError::InvalidArgument(format!(
                "Actor {} is not a Server Actor.",
                self.id
            ))),
        }
    }

    /// Walks the Actor's builds, newest first, for the one with `build_number`.
    async fn find_build_by_number(
        &self,
        build_number: &str,
        target: [u64; 3],
    ) -> ApifyClientResult<Option<String>> {
        let mut builds = self.builds().iterate(ListOptions {
            desc: Some(true),
            ..ListOptions::default()
        });
        while let Some(build) = builds.next().await? {
            if build.build_number.as_deref() == Some(build_number) {
                return Ok(Some(build.id));
            }
            if is_older_build(build.build_number.as_deref(), target) {
                break;
            }
        }
        Ok(None)
    }

    /// The build collection of the Actor, with this client's timeout. Without one, it lists with
    /// the `Short` tier, like every other API request of this client, where the collection client
    /// defaults to `Medium`.
    fn builds(&self) -> BuildCollectionClient {
        BuildCollectionClient::with_base(self.ctx.http.clone(), &self.ctx.url(None), "builds")
            .with_timeout(self.ctx.timeout.unwrap_or(Timeout::Short))
    }

    /// A client for the build `id`, with this client's timeout.
    fn build_client(&self, id: &str) -> BuildClient {
        let build = BuildClient::new(self.ctx.http.clone(), &self.base_url, id);
        match self.ctx.timeout {
            Some(timeout) => build.with_timeout(timeout),
            None => build,
        }
    }
}

/// Splits a build number like `0.1.23` into its version and sequence number, or returns `None`
/// for anything else.
fn parse_build_number(build_number: &str) -> Option<[u64; 3]> {
    let parts: Vec<u64> = build_number
        .split('.')
        .map(parse_digits)
        .collect::<Option<_>>()?;
    parts.try_into().ok()
}

/// Parses a non-empty run of ASCII digits. `str::parse` alone would also accept a leading `+`.
fn parse_digits(part: &str) -> Option<u64> {
    if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

/// Returns whether `build_number` belongs to the same version as `target` and comes before it.
/// Builds are listed newest first and the sequence numbers of a version grow over time, so
/// meeting such a build means the target doesn't exist.
fn is_older_build(build_number: Option<&str>, target: [u64; 3]) -> bool {
    build_number
        .and_then(parse_build_number)
        .is_some_and(|parsed| {
            parsed[0] == target[0] && parsed[1] == target[1] && parsed[2] < target[2]
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_numbers_are_parsed_strictly() {
        assert_eq!(parse_build_number("0.1.23"), Some([0, 1, 23]));
        assert_eq!(parse_build_number("latest"), None);
        assert_eq!(parse_build_number("0.1"), None);
        assert_eq!(parse_build_number("0.1.2.3"), None);
        assert_eq!(parse_build_number("0.+1.2"), None);
        assert_eq!(parse_build_number("0..2"), None);
    }

    #[test]
    fn only_an_earlier_build_of_the_same_version_ends_the_search() {
        let target = [0, 1, 5];
        assert!(is_older_build(Some("0.1.4"), target));
        assert!(!is_older_build(Some("0.1.5"), target));
        assert!(!is_older_build(Some("0.1.6"), target));
        assert!(!is_older_build(Some("0.2.1"), target));
        assert!(!is_older_build(None, target));
    }
}
