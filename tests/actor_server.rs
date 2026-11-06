//! Integration test for the server interface of an Actor (`ActorClient::server`).
//!
//! It creates and builds a temporary Server Actor that echoes every request it receives, sends
//! requests to it, and deletes it.

mod common;

use apify_client::clients::actor_server::{ActorServerBody, ActorServerRequestOptions};
use apify_client::http_client::HttpMethod;
use serde_json::{json, Value};

/// A web server that answers every request, including the platform's readiness probe, with the
/// request it received, under the status in the `status` query parameter, or 200.
const ECHO_SERVER_SOURCE: &str = r#"
const http = require('http');

http.createServer((req, res) => {
    let body = '';
    req.on('data', (chunk) => { body += chunk; });
    req.on('end', () => {
        const status = Number(new URL(req.url, 'http://localhost').searchParams.get('status')) || 200;
        res.writeHead(status, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ method: req.method, url: req.url, headers: req.headers, body }));
    });
}).listen(process.env.ACTOR_WEB_SERVER_PORT);
"#;

const DOCKERFILE: &str = "FROM node:22-alpine\nCOPY main.js ./\nCMD [\"node\", \"main.js\"]\n";

/// How long to wait for the build of the echo Actor, in seconds.
const BUILD_WAIT_SECS: i64 = 300;

/// The web server schema of the echo Actor.
fn openapi_spec() -> Value {
    json!({
        "openapi": "3.0.0",
        "info": { "title": "Echo server", "version": "1.0.0" },
        "paths": { "/echo": { "post": { "responses": { "200": { "description": "The echoed request." } } } } },
    })
}

fn query(params: &[(&str, &str)]) -> Vec<(String, String)> {
    params
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_server_client_returns_the_url_and_spec_and_sends_requests_to_the_server() {
    let client = require_client!();
    let name = common::unique_actor_name("server");
    let actor_json = json!({
        "actorSpecification": 1,
        "name": name,
        "version": "0.0",
        "usesStandbyMode": true,
        "webServerSchema": openapi_spec(),
    });
    let actor = client
        .actors()
        .create(&json!({
            "name": name,
            "isPublic": false,
            "versions": [{
                "versionNumber": "0.0",
                "sourceType": "SOURCE_FILES",
                "buildTag": "latest",
                "sourceFiles": [
                    { "name": "Dockerfile", "format": "TEXT", "content": DOCKERFILE },
                    { "name": "main.js", "format": "TEXT", "content": ECHO_SERVER_SOURCE },
                    { "name": ".actor/actor.json", "format": "TEXT", "content": actor_json.to_string() },
                ],
            }],
            "actorStandby": { "isEnabled": true, "memoryMbytes": 256, "idleTimeoutSecs": 60 },
        }))
        .await
        .expect("create the server actor");

    let cleanup_client = client.clone();
    let cleanup_id = actor.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.actor(&cleanup_id).delete().await;
    });

    let actor_client = client.actor(&actor.id);
    let build = actor_client
        .build("0.0", Default::default())
        .await
        .expect("start the build");
    let finished = client
        .build(&build.id)
        .wait_for_finish(Some(BUILD_WAIT_SECS))
        .await
        .expect("wait for the build");
    assert_eq!(finished.status.as_deref(), Some("SUCCEEDED"));

    let fetched = actor_client
        .get()
        .await
        .expect("get the actor")
        .expect("the actor exists");
    assert_eq!(
        fetched.actor_standby.as_ref().and_then(|s| s.is_enabled),
        Some(true)
    );
    let server = actor_client.server();

    assert_eq!(
        Some(server.get_url().await.expect("get the url")),
        fetched.standby_url
    );
    assert_eq!(
        server.get_open_api_spec().await.expect("get the spec"),
        Some(openapi_spec())
    );

    // A JSON POST with a query reaches the server as it was sent.
    let response = server
        .request(
            "/echo",
            ActorServerRequestOptions {
                method: Some(HttpMethod::Post),
                params: query(&[("q", "apify")]),
                body: Some(ActorServerBody::Json(json!({ "limit": 5 }))),
                ..Default::default()
            },
        )
        .await
        .expect("post to the server");
    assert_eq!(response.status, 200, "{}", response.text());
    let echoed: Value = response.json().expect("a JSON echo");
    assert_eq!(echoed["method"], "POST");
    assert_eq!(echoed["url"], "/echo?q=apify");
    let body: Value = serde_json::from_str(echoed["body"].as_str().unwrap()).unwrap();
    assert_eq!(body, json!({ "limit": 5 }));
    // The platform consumes the client's token, so it never reaches the Actor's code.
    assert!(echoed["headers"].get("x-apify-authorization").is_none());
    assert!(echoed["headers"].get("authorization").is_none());

    // Headers of the request, `Authorization` included, reach the Actor as they are.
    let mut own_authorization = ActorServerRequestOptions::default();
    own_authorization.headers.insert(
        "Authorization".to_string(),
        "Bearer actor-token".to_string(),
    );
    let response = server
        .request("/echo", own_authorization)
        .await
        .expect("get with an own authorization");
    let echoed: Value = response.json().expect("a JSON echo");
    assert_eq!(echoed["headers"]["authorization"], "Bearer actor-token");

    // An error status is returned, not raised.
    let response = server
        .request(
            "/echo",
            ActorServerRequestOptions {
                params: query(&[("status", "404")]),
                ..Default::default()
            },
        )
        .await
        .expect("an error status is a response");
    assert_eq!(response.status, 404);
    let echoed: Value = response.json().expect("a JSON echo");
    assert_eq!(echoed["url"], "/echo?status=404");
}
