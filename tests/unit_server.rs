//! Offline unit tests for the server interface of an Actor (`ActorClient::server`).
//!
//! These use a mock [`HttpBackend`](apify_client::http_client::HttpBackend) that answers the
//! requests in order, and require no network access or `APIFY_TOKEN`.

mod common;

use std::sync::Arc;

use apify_client::clients::actor_server::{
    ActorServerBody, ActorServerCompression, ActorServerRequestOptions,
};
use apify_client::http_client::{HttpMethod, HttpRequest, HttpResponse};
use apify_client::{ApifyClientError, RequestCompression};
use common::mock_backend::{client_with, MockBackend, MockOutcome};
use futures_util::StreamExt;
use serde_json::{json, Value};

const SERVER_URL: &str = "https://user--actor.apify.actor";

fn actor(extra: Value) -> MockOutcome {
    let mut data = json!({ "id": "a", "standbyUrl": SERVER_URL });
    data.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    MockOutcome::Status(200, json!({ "data": data }).to_string().into_bytes())
}

fn server_response(status: u16, body: &str) -> MockOutcome {
    MockOutcome::Status(status, body.as_bytes().to_vec())
}

fn build(extra: Value) -> MockOutcome {
    let mut data = json!({ "id": "b1" });
    data.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    MockOutcome::Status(200, json!({ "data": data }).to_string().into_bytes())
}

fn builds(items: Value) -> MockOutcome {
    MockOutcome::Status(
        200,
        json!({ "data": { "total": 0, "offset": 0, "limit": 1000, "count": 0, "desc": true, "items": items } })
            .to_string()
            .into_bytes(),
    )
}

fn options_with_params(params: &[(&str, &str)]) -> ActorServerRequestOptions {
    ActorServerRequestOptions {
        params: params
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
        ..Default::default()
    }
}

#[tokio::test]
async fn the_url_is_fetched_once_and_shared_by_the_clients_of_an_actor() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(200, "{}")]);
    let client = client_with(backend.clone(), 0);
    let actor_client = client.actor("user~actor");

    let url = actor_client.server().get_url().await.unwrap();
    assert_eq!(url, SERVER_URL);
    actor_client.server().get_url().await.unwrap();
    actor_client
        .server()
        .request("/", Default::default())
        .await
        .unwrap();

    assert_eq!(
        backend.urls(),
        vec![
            "https://api.apify.com/v2/actors/user~actor".to_string(),
            format!("{SERVER_URL}/"),
        ]
    );
}

#[tokio::test]
async fn an_actor_without_a_server_url_is_not_a_server_actor_and_the_failure_is_not_kept() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            json!({ "data": { "id": "a", "standbyUrl": null } })
                .to_string()
                .into_bytes(),
        ),
        actor(json!({})),
    ]);
    let client = client_with(backend.clone(), 0);
    let server = client.actor("a").server();

    let error = server.get_url().await.unwrap_err();
    assert!(
        matches!(&error, ApifyClientError::InvalidArgument(message) if message.contains("not a Server Actor")),
        "{error:?}"
    );
    assert_eq!(server.get_url().await.unwrap(), SERVER_URL);
    assert_eq!(backend.call_count(), 2);
}

#[tokio::test]
async fn a_missing_actor_is_an_error() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        404,
        br#"{"error":{"type":"record-not-found","message":"nope"}}"#.to_vec(),
    )]);
    let client = client_with(backend, 0);

    let error = client
        .actor("a")
        .server()
        .request("/", Default::default())
        .await
        .unwrap_err();
    assert_eq!(error.status_code(), Some(404), "{error:?}");
}

#[tokio::test]
async fn the_token_goes_in_the_x_apify_authorization_header_and_own_headers_reach_the_server() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(200, "ok")]);
    let client = client_with(backend.clone(), 0);

    let mut options = options_with_params(&[("q", "apify docs"), ("q", "b")]);
    options.headers.insert(
        "Authorization".to_string(),
        "Bearer actor-token".to_string(),
    );
    let response = client
        .actor("a")
        .server()
        .request("//search?lang=en#fragment", options)
        .await
        .unwrap();

    assert_eq!(response.text(), "ok");
    assert_eq!(
        backend.last_url().unwrap(),
        format!("{SERVER_URL}/search?lang=en&q=apify%20docs&q=b")
    );
    assert_eq!(
        backend.last_header("Authorization").as_deref(),
        Some("Bearer actor-token")
    );
    assert_eq!(
        backend.last_header("X-Apify-Authorization").as_deref(),
        Some("Bearer test-token")
    );
    assert!(backend.last_header("User-Agent").is_some());
    assert_eq!(backend.follow_redirects(), vec![true, false]);
}

#[tokio::test]
async fn the_token_does_not_travel_in_the_authorization_header_by_default() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(200, "ok")]);
    let client = client_with(backend.clone(), 0);

    client
        .actor("a")
        .server()
        .request("/", Default::default())
        .await
        .unwrap();

    assert_eq!(backend.last_header("Authorization"), None);
}

#[tokio::test]
async fn a_default_authorization_header_is_moved_too_and_a_request_header_of_either_name_wins() {
    let backend = MockBackend::new(vec![
        actor(json!({})),
        server_response(200, "ok"),
        server_response(200, "ok"),
    ]);
    let client = apify_client::ApifyClient::builder()
        .token("test-token")
        .max_retries(0)
        .default_header("authorization", "Bearer default")
        .http_backend(backend.clone())
        .build();
    let server = client.actor("a").server();

    server.request("/", Default::default()).await.unwrap();
    assert_eq!(backend.last_header("Authorization"), None);
    assert_eq!(
        backend.last_header("X-Apify-Authorization").as_deref(),
        Some("Bearer default")
    );

    let mut options = ActorServerRequestOptions::default();
    options.headers.insert(
        "x-apify-authorization".to_string(),
        "Bearer own".to_string(),
    );
    server.request("/", options).await.unwrap();
    assert_eq!(
        backend.last_header("X-Apify-Authorization").as_deref(),
        Some("Bearer own")
    );
}

#[tokio::test]
async fn a_json_body_is_sent_with_its_method_and_content_type() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(200, "{}")]);
    let client = client_with(backend.clone(), 0);

    client
        .actor("a")
        .server()
        .request(
            "/items",
            ActorServerRequestOptions {
                method: Some(HttpMethod::Patch),
                body: Some(ActorServerBody::json(&json!({ "limit": 5 })).unwrap()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(backend.methods(), vec![HttpMethod::Get, HttpMethod::Patch]);
    assert_eq!(
        backend.last_header("Content-Type").as_deref(),
        Some("application/json")
    );
    assert_eq!(backend.last_body().unwrap(), br#"{"limit":5}"#);
}

#[tokio::test]
async fn text_and_binary_bodies_are_sent_as_they_are() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(200, "{}")]);
    let client = client_with(backend.clone(), 0);
    let server = client.actor("a").server();

    let mut text = ActorServerRequestOptions {
        method: Some(HttpMethod::Post),
        body: Some(ActorServerBody::Text("a=1".to_string())),
        ..Default::default()
    };
    text.headers.insert(
        "Content-Type".to_string(),
        "application/x-www-form-urlencoded".to_string(),
    );
    server.request("/form", text).await.unwrap();
    assert_eq!(backend.last_body().unwrap(), b"a=1");
    assert_eq!(
        backend.last_header("Content-Type").as_deref(),
        Some("application/x-www-form-urlencoded")
    );

    server
        .request(
            "/bytes",
            ActorServerRequestOptions {
                method: Some(HttpMethod::Put),
                body: Some(ActorServerBody::Bytes(vec![0, 1, 2])),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(backend.last_body().unwrap(), vec![0, 1, 2]);
    assert_eq!(backend.last_header("Content-Type"), None);
}

#[tokio::test]
async fn an_error_status_is_returned_as_it_is_without_a_retry() {
    let backend = MockBackend::new(vec![
        actor(json!({})),
        server_response(404, r#"{"error":"not here"}"#),
    ]);
    let client = client_with(backend.clone(), 3);

    let response = client
        .actor("a")
        .server()
        .request("/missing", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 404);
    assert!(!response.is_success());
    assert_eq!(
        response.json::<Value>().unwrap(),
        json!({ "error": "not here" })
    );
    assert_eq!(backend.call_count(), 2, "the actor and one request");
}

#[tokio::test]
async fn a_server_error_other_than_503_is_not_retried() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(500, "boom")]);
    let client = client_with(backend.clone(), 3);

    let response = client
        .actor("a")
        .server()
        .request("/", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 500);
    assert_eq!(backend.call_count(), 2);
}

#[tokio::test]
async fn rate_limit_and_unavailable_responses_are_retried_and_the_last_one_is_returned() {
    let backend = MockBackend::new(vec![
        actor(json!({})),
        server_response(429, "slow down"),
        server_response(503, "unavailable"),
    ]);
    let client = client_with(backend.clone(), 2);

    let response = client
        .actor("a")
        .server()
        .request("/", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 503);
    assert_eq!(backend.call_count(), 4, "the actor and three attempts");
    assert_eq!(
        client.stats().rate_limit_errors().iter().sum::<u64>(),
        0,
        "a server's rate limit is not one of the API"
    );
}

#[tokio::test]
async fn a_503_that_clears_up_is_retried_into_a_success() {
    let backend = MockBackend::new(vec![
        actor(json!({})),
        server_response(503, "starting"),
        server_response(200, "ready"),
    ]);
    let client = client_with(backend.clone(), 2);

    let response = client
        .actor("a")
        .server()
        .request("/", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 200);
    assert_eq!(response.text(), "ready");
}

#[tokio::test]
async fn network_errors_and_timeouts_are_not_retried() {
    for outcome in [MockOutcome::NetworkError, MockOutcome::Timeout] {
        let backend = MockBackend::new(vec![actor(json!({})), outcome]);
        let client = client_with(backend.clone(), 3);

        let error = client
            .actor("a")
            .server()
            .request("/", Default::default())
            .await
            .unwrap_err();

        assert!(
            matches!(error, ApifyClientError::Http(_) | ApifyClientError::Timeout),
            "{error:?}"
        );
        assert_eq!(backend.call_count(), 2, "the actor and one attempt");
    }
}

#[tokio::test]
async fn a_redirect_is_returned_as_it_is() {
    let backend = MockBackend::new(vec![
        actor(json!({})),
        MockOutcome::StatusWithHeaders(
            302,
            vec![("Location", "https://elsewhere.example.com/".to_string())],
            Vec::new(),
        ),
    ]);
    let client = client_with(backend.clone(), 3);

    let response = client
        .actor("a")
        .server()
        .request("/", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 302);
    assert_eq!(
        response.header("location"),
        Some("https://elsewhere.example.com/")
    );
    assert_eq!(backend.call_count(), 2);
}

#[tokio::test]
async fn the_request_timeout_defaults_to_the_long_tier() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(200, "ok")]);
    let client = client_with(backend.clone(), 0);

    client
        .actor("a")
        .server()
        .request("/", Default::default())
        .await
        .unwrap();

    let timeouts = backend.timeouts();
    assert_eq!(timeouts[0], std::time::Duration::from_secs(5), "the actor");
    assert_eq!(
        timeouts[1],
        std::time::Duration::from_secs(360),
        "the server"
    );
}

#[tokio::test]
async fn a_request_body_is_sent_uncompressed_unless_compression_is_set() {
    let large = "a".repeat(4096);
    let backend = MockBackend::new(vec![
        actor(json!({})),
        server_response(200, "{}"),
        server_response(200, "{}"),
    ]);
    let client = client_with(backend.clone(), 0);
    let server = client.actor("a").server();
    let options = |compression| ActorServerRequestOptions {
        method: Some(HttpMethod::Post),
        body: Some(ActorServerBody::Text(large.clone())),
        compression,
        ..Default::default()
    };

    server.request("/", options(None)).await.unwrap();
    assert_eq!(backend.last_header("Content-Encoding"), None);
    assert_eq!(backend.last_body().unwrap().len(), large.len());

    server
        .request(
            "/",
            options(Some(ActorServerCompression::Algorithm(
                RequestCompression::Gzip,
            ))),
        )
        .await
        .unwrap();
    assert_eq!(
        backend.last_header("Content-Encoding").as_deref(),
        Some("gzip")
    );
    assert!(backend.last_body().unwrap().len() < large.len());
}

#[tokio::test]
async fn a_custom_compressor_compresses_the_body() {
    #[derive(Debug)]
    struct Marker;
    impl apify_client::HttpCompressor for Marker {
        fn content_encoding(&self) -> &str {
            "marker"
        }
        fn compress(&self, data: &[u8]) -> apify_client::ApifyClientResult<Vec<u8>> {
            Ok(data[..4].to_vec())
        }
    }

    let backend = MockBackend::new(vec![actor(json!({})), server_response(200, "{}")]);
    let client = client_with(backend.clone(), 0);

    client
        .actor("a")
        .server()
        .request(
            "/",
            ActorServerRequestOptions {
                method: Some(HttpMethod::Post),
                body: Some(ActorServerBody::Text("b".repeat(4096))),
                compression: Some(ActorServerCompression::Custom(Arc::new(Marker))),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(
        backend.last_header("Content-Encoding").as_deref(),
        Some("marker")
    );
    assert_eq!(backend.last_body().unwrap(), b"bbbb");
}

#[tokio::test]
async fn a_stream_response_is_returned_with_any_status() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(418, "teapot")]);
    let client = client_with(backend.clone(), 3);

    let mut response = client
        .actor("a")
        .server()
        .request_stream("/tea", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 418);
    let mut body = Vec::new();
    while let Some(chunk) = response.body.next().await {
        body.extend(chunk.unwrap());
    }
    assert_eq!(body, b"teapot");
    assert_eq!(backend.call_count(), 2);
    assert_eq!(backend.last_header("Authorization"), None);
    assert_eq!(
        backend.last_header("X-Apify-Authorization").as_deref(),
        Some("Bearer test-token")
    );
    assert_eq!(backend.follow_redirects(), vec![true, false]);
}

#[tokio::test]
async fn the_standby_alias_returns_a_server_client() {
    let backend = MockBackend::new(vec![actor(json!({}))]);
    let client = client_with(backend, 0);

    #[allow(deprecated)] // The alias is deprecated, and this test covers it.
    let url = client.actor("a").standby().get_url().await.unwrap();
    assert_eq!(url, SERVER_URL);
}

#[tokio::test]
async fn the_spec_is_the_web_server_schema_of_the_latest_build_by_default() {
    let schema = json!({ "openapi": "3.0.0", "paths": {} });
    let backend = MockBackend::new(vec![
        actor(json!({ "taggedBuilds": { "latest": { "buildId": "b1" } } })),
        build(json!({ "actorDefinition": { "webServerSchema": schema } })),
    ]);
    let client = client_with(backend.clone(), 0);
    let server = client.actor("a").server();

    assert_eq!(server.get_open_api_spec().await.unwrap(), Some(schema));
    assert_eq!(
        backend.last_url().unwrap(),
        "https://api.apify.com/v2/actor-builds/b1"
    );
    // The spec lookup cached the URL.
    assert_eq!(server.get_url().await.unwrap(), SERVER_URL);
    assert_eq!(backend.call_count(), 2);
}

#[tokio::test]
async fn the_spec_comes_from_the_build_tag_the_server_runs() {
    let backend = MockBackend::new(vec![
        actor(json!({
            "actorStandby": { "build": "beta" },
            "taggedBuilds": { "latest": { "buildId": "b1" }, "beta": { "buildId": "b2" } },
        })),
        build(
            json!({ "id": "b2", "actorDefinition": { "webServerSchema": { "openapi": "beta" } } }),
        ),
    ]);
    let client = client_with(backend.clone(), 0);

    let spec = client
        .actor("a")
        .server()
        .get_open_api_spec()
        .await
        .unwrap();

    assert_eq!(spec, Some(json!({ "openapi": "beta" })));
    assert_eq!(
        backend.last_url().unwrap(),
        "https://api.apify.com/v2/actor-builds/b2"
    );
}

#[tokio::test]
async fn the_spec_looks_up_a_build_number_in_the_builds_of_the_actor() {
    let backend = MockBackend::new(vec![
        actor(json!({ "actorStandby": { "build": "0.1.2" }, "taggedBuilds": {} })),
        builds(json!([
            { "id": "b3", "buildNumber": "0.1.3" },
            { "id": "b2", "buildNumber": "0.1.2" },
        ])),
        build(json!({ "id": "b2", "actorDefinition": { "webServerSchema": { "openapi": "2" } } })),
    ]);
    let client = client_with(backend.clone(), 0);

    let spec = client
        .actor("a")
        .server()
        .get_open_api_spec()
        .await
        .unwrap();

    assert_eq!(spec, Some(json!({ "openapi": "2" })));
    let urls = backend.urls();
    assert!(
        urls[1].starts_with("https://api.apify.com/v2/actors/a/builds?"),
        "{urls:?}"
    );
    assert!(urls[1].contains("desc=1"), "{urls:?}");
    assert_eq!(urls[2], "https://api.apify.com/v2/actor-builds/b2");
}

#[tokio::test]
async fn the_build_number_search_stops_at_an_earlier_build_of_the_same_version() {
    let backend = MockBackend::new(vec![
        actor(json!({ "actorStandby": { "build": "0.1.5" }, "taggedBuilds": {} })),
        builds(json!([
            { "id": "b6", "buildNumber": "0.1.6" },
            { "id": "b4", "buildNumber": "0.1.4" },
        ])),
    ]);
    let client = client_with(backend.clone(), 0);

    let error = client
        .actor("a")
        .server()
        .get_open_api_spec()
        .await
        .unwrap_err();

    assert!(
        matches!(&error, ApifyClientError::InvalidArgument(message) if message.contains("Build 0.1.5 of Actor a does not exist")),
        "{error:?}"
    );
    assert_eq!(backend.call_count(), 2, "the older build ended the search");
}

#[tokio::test]
async fn a_tag_without_a_build_is_an_error() {
    let backend = MockBackend::new(vec![actor(json!({ "taggedBuilds": { "latest": null } }))]);
    let client = client_with(backend.clone(), 0);

    let result = client.actor("a").server().get_open_api_spec().await;

    // `latest` is neither a tag with a build nor a build number, so no build is looked up.
    assert!(
        matches!(&result, Err(ApifyClientError::InvalidArgument(message)) if message.contains("Build latest of Actor a does not exist")),
        "{result:?}"
    );
    assert_eq!(backend.call_count(), 1);
}

#[tokio::test]
async fn a_build_the_api_does_not_return_is_an_error_even_with_a_known_id() {
    let backend = MockBackend::new(vec![
        actor(json!({ "taggedBuilds": { "latest": { "buildId": "gone" } } })),
        MockOutcome::Status(404, br#"{"error":{"message":"gone"}}"#.to_vec()),
    ]);
    let client = client_with(backend, 0);

    let result = client.actor("a").server().get_open_api_spec().await;

    assert!(
        matches!(&result, Err(ApifyClientError::InvalidArgument(_))),
        "{result:?}"
    );
}

#[tokio::test]
async fn the_spec_is_none_when_the_definition_has_no_web_server_schema() {
    for definition in [
        json!({}),
        json!({ "actorDefinition": null }),
        json!({ "actorDefinition": {} }),
    ] {
        let backend = MockBackend::new(vec![
            actor(json!({ "taggedBuilds": { "latest": { "buildId": "b1" } } })),
            build(definition),
        ]);
        let client = client_with(backend, 0);

        let spec = client
            .actor("a")
            .server()
            .get_open_api_spec()
            .await
            .unwrap();

        assert_eq!(spec, None);
    }
}

#[tokio::test]
async fn the_spec_of_an_actor_that_is_not_a_server_actor_is_an_error() {
    let backend = MockBackend::new(vec![MockOutcome::Status(
        200,
        json!({ "data": { "id": "a" } }).to_string().into_bytes(),
    )]);
    let client = client_with(backend, 0);

    let result = client.actor("a").server().get_open_api_spec().await;

    assert!(
        matches!(&result, Err(ApifyClientError::InvalidArgument(message)) if message.contains("not a Server Actor")),
        "{result:?}"
    );
}

#[tokio::test]
async fn with_timeout_sets_the_timeout_of_every_request() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(200, "ok")]);
    let client = client_with(backend.clone(), 0);

    client
        .actor("a")
        .server()
        .with_timeout(apify_client::Timeout::Short)
        .request("/", Default::default())
        .await
        .unwrap();

    let timeouts = backend.timeouts();
    assert_eq!(timeouts[0], std::time::Duration::from_secs(5));
    assert_eq!(timeouts[1], std::time::Duration::from_secs(5));
}

#[test]
fn http_response_helpers_describe_the_body() {
    let response = HttpResponse {
        status: 204,
        headers: Default::default(),
        body: b"{\"a\":1}".to_vec(),
    };
    assert!(response.is_success());
    assert_eq!(response.text(), "{\"a\":1}");
    assert_eq!(response.json::<Value>().unwrap(), json!({ "a": 1 }));
    assert!(!HttpResponse {
        status: 300,
        ..response
    }
    .is_success());
}

#[tokio::test]
async fn the_default_backend_does_not_follow_a_redirect_when_asked_not_to() {
    use apify_client::http_client::{HttpBackend, ReqwestBackend};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = [0u8; 1024];
        let _ = socket.read(&mut buffer).await.unwrap();
        socket
            .write_all(b"HTTP/1.1 302 Found\r\nlocation: http://127.0.0.1:1/\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
            .await
            .unwrap();
    });

    let response = ReqwestBackend::new()
        .send(HttpRequest {
            method: HttpMethod::Get,
            url: format!("http://{address}/"),
            headers: Default::default(),
            body: None,
            timeout: std::time::Duration::from_secs(5),
            follow_redirects: false,
        })
        .await
        .expect("the redirect is returned, not followed");

    assert_eq!(response.status, 302);
    assert_eq!(response.header("location"), Some("http://127.0.0.1:1/"));
}

#[tokio::test]
async fn a_stream_response_is_retried_on_503_until_it_starts() {
    let backend = MockBackend::new(vec![
        actor(json!({})),
        server_response(503, "starting"),
        server_response(200, "ready"),
    ]);
    let client = client_with(backend.clone(), 2);

    let mut response = client
        .actor("a")
        .server()
        .request_stream("/", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 200);
    let chunk = response.body.next().await.unwrap().unwrap();
    assert_eq!(chunk, b"ready");
    assert_eq!(backend.call_count(), 3, "the actor and two attempts");
}

#[tokio::test]
async fn a_stream_response_with_an_error_status_that_is_not_retried_is_returned_at_once() {
    let backend = MockBackend::new(vec![actor(json!({})), server_response(500, "boom")]);
    let client = client_with(backend.clone(), 3);

    let response = client
        .actor("a")
        .server()
        .request_stream("/", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 500);
    assert_eq!(backend.call_count(), 2);
}

/// A backend whose streaming responses start at once and then stay silent for a long while.
#[derive(Debug)]
struct SilentStreamBackend {
    actor: Vec<u8>,
}

/// How long the stream of [`SilentStreamBackend`] stays silent between its two chunks.
const SILENCE: std::time::Duration = std::time::Duration::from_secs(600);

#[async_trait::async_trait]
impl apify_client::http_client::HttpBackend for SilentStreamBackend {
    async fn send(&self, _request: HttpRequest) -> apify_client::ApifyClientResult<HttpResponse> {
        Ok(HttpResponse {
            status: 200,
            headers: Default::default(),
            body: self.actor.clone(),
        })
    }

    async fn send_streaming(
        &self,
        _request: HttpRequest,
    ) -> apify_client::ApifyClientResult<apify_client::http_client::StreamingHttpResponse> {
        let body = futures_util::stream::unfold(0u8, |step| async move {
            match step {
                0 => Some((Ok(b"first".to_vec()), 1)),
                1 => {
                    tokio::time::sleep(SILENCE).await;
                    Some((Ok(b"second".to_vec()), 2))
                }
                _ => None,
            }
        });
        Ok(apify_client::http_client::StreamingHttpResponse {
            status: 200,
            headers: Default::default(),
            body: Box::pin(body),
        })
    }
}

/// The timeout of a server stream bounds only the wait for the response to start: a server that
/// keeps the connection open and stays silent between two chunks is not cut off.
#[tokio::test(start_paused = true)]
async fn a_server_stream_is_not_cut_off_while_the_server_stays_silent() {
    let backend = Arc::new(SilentStreamBackend {
        actor: json!({ "data": { "id": "a", "standbyUrl": SERVER_URL } })
            .to_string()
            .into_bytes(),
    });
    let client = apify_client::ApifyClient::builder()
        .token("test-token")
        .http_backend(backend)
        .build();
    let server = client
        .actor("a")
        .server()
        .with_timeout(apify_client::Timeout::Short);

    let mut response = server
        .request_stream("/events", Default::default())
        .await
        .unwrap();
    let mut chunks = Vec::new();
    while let Some(chunk) = response.body.next().await {
        chunks.push(chunk.expect("no timeout"));
    }

    assert_eq!(chunks, vec![b"first".to_vec(), b"second".to_vec()]);
}

#[tokio::test]
async fn a_stream_response_is_not_retried_on_a_network_error_or_a_timeout() {
    for outcome in [MockOutcome::NetworkError, MockOutcome::Timeout] {
        let backend = MockBackend::new(vec![actor(json!({})), outcome]);
        let client = client_with(backend.clone(), 3);

        let error = client
            .actor("a")
            .server()
            .request_stream("/", Default::default())
            .await
            .err()
            .expect("the transport error ends the call");

        assert!(
            matches!(error, ApifyClientError::Http(_) | ApifyClientError::Timeout),
            "{error:?}"
        );
        assert_eq!(backend.call_count(), 2, "the actor and one attempt");
    }
}

#[tokio::test]
async fn a_stream_response_returns_the_last_retried_status_once_the_retries_run_out() {
    let backend = MockBackend::new(vec![
        actor(json!({})),
        server_response(429, "slow down"),
        server_response(503, "unavailable"),
    ]);
    let client = client_with(backend.clone(), 1);

    let mut response = client
        .actor("a")
        .server()
        .request_stream("/", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 503);
    assert_eq!(response.body.next().await.unwrap().unwrap(), b"unavailable");
    assert_eq!(backend.call_count(), 3, "the actor and two attempts");
    assert_eq!(
        client.stats().rate_limit_errors().iter().sum::<u64>(),
        0,
        "a server's rate limit is not one of the API"
    );
}

/// The body of a streaming response.
type ErrorBody = std::pin::Pin<
    Box<dyn futures_util::Stream<Item = apify_client::ApifyClientResult<Vec<u8>>> + Send>,
>;

/// A backend whose streaming error responses never finish their body: the status arrives at once
/// and the body then stays silent for as long as the test lets it.
#[derive(Debug)]
struct HangingErrorBodyBackend {
    actor: Vec<u8>,
    stream_calls: std::sync::atomic::AtomicUsize,
}

#[async_trait::async_trait]
impl apify_client::http_client::HttpBackend for HangingErrorBodyBackend {
    async fn send(&self, _request: HttpRequest) -> apify_client::ApifyClientResult<HttpResponse> {
        Ok(HttpResponse {
            status: 200,
            headers: Default::default(),
            body: self.actor.clone(),
        })
    }

    async fn send_streaming(
        &self,
        _request: HttpRequest,
    ) -> apify_client::ApifyClientResult<apify_client::http_client::StreamingHttpResponse> {
        let call = self
            .stream_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // The first attempt fails with a `503` whose body never ends, the second one succeeds.
        let (status, body): (u16, ErrorBody) = if call == 0 {
            (503, Box::pin(futures_util::stream::pending()))
        } else {
            (
                200,
                Box::pin(futures_util::stream::once(async { Ok(b"ready".to_vec()) })),
            )
        };
        Ok(apify_client::http_client::StreamingHttpResponse {
            status,
            headers: Default::default(),
            body,
        })
    }
}

/// A retry that follows drops the failed response unread: a `503` whose body never ends does not
/// delay or prevent the retry.
#[tokio::test(start_paused = true)]
async fn a_stream_retry_does_not_wait_for_the_body_of_the_failed_response() {
    let backend = Arc::new(HangingErrorBodyBackend {
        actor: json!({ "data": { "id": "a", "standbyUrl": SERVER_URL } })
            .to_string()
            .into_bytes(),
        stream_calls: Default::default(),
    });
    let client = apify_client::ApifyClient::builder()
        .token("test-token")
        .max_retries(1)
        .min_delay_between_retries(std::time::Duration::from_millis(1))
        .http_backend(backend)
        .build();

    let started = tokio::time::Instant::now();
    let mut response = client
        .actor("a")
        .server()
        .request_stream("/", Default::default())
        .await
        .unwrap();

    assert_eq!(response.status, 200);
    assert_eq!(response.body.next().await.unwrap().unwrap(), b"ready");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "the retry waited for the body of the failed response"
    );
}

/// The builds listing that looks up a build number uses the `Short` tier, like every API request
/// of the server client, unless the client has a timeout of its own.
#[tokio::test]
async fn the_build_number_lookup_lists_builds_with_the_short_tier() {
    let backend = MockBackend::new(vec![
        actor(json!({ "actorStandby": { "build": "0.1.2" }, "taggedBuilds": {} })),
        builds(json!([{ "id": "b2", "buildNumber": "0.1.2" }])),
        build(json!({ "id": "b2", "actorDefinition": { "webServerSchema": {} } })),
    ]);
    let client = client_with(backend.clone(), 0);

    client
        .actor("a")
        .server()
        .get_open_api_spec()
        .await
        .unwrap();

    let short = std::time::Duration::from_secs(5);
    assert_eq!(backend.timeouts(), vec![short, short, short]);
}
