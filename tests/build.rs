//! Integration tests for the Actor build resource.

mod common;

use serde_json::json;

/// Simple GET: listing the account's builds.
#[tokio::test]
async fn list_builds() {
    let client = require_client!();
    let page = client
        .builds()
        .list(Default::default())
        .await
        .expect("listing builds should succeed");
    assert!(page.total >= 0);
}

/// Complex flow: create an Actor, build it, wait for the build to finish, fetch the build
/// and its log, then clean up.
#[tokio::test]
async fn build_actor_flow() {
    let client = require_client!();
    let name = common::unique_name("build").replace('-', "");
    let name = format!("b{}", &name[..name.len().min(20)]);

    let definition = json!({
        "name": name,
        "isPublic": false,
        "versions": [{
            "versionNumber": "0.0",
            "sourceType": "SOURCE_FILES",
            "buildTag": "latest",
            "sourceFiles": [
                {
                    "name": "Dockerfile",
                    "format": "TEXT",
                    "content": "FROM apify/actor-node:20\nCOPY . ./\nCMD node main.js"
                },
                {
                    "name": "main.js",
                    "format": "TEXT",
                    "content": "console.log('built by rust client test');"
                }
            ]
        }]
    });

    let actor = client
        .actors()
        .create(&definition)
        .await
        .expect("create actor");
    let actor_client = client.actor(&actor.id);

    // Start a build and wait for it to finish.
    let build = actor_client
        .build("0.0", Default::default())
        .await
        .expect("start build");
    let build_client = client.build(&build.id);
    let finished = build_client
        .wait_for_finish(Some(300))
        .await
        .expect("wait for build");
    assert!(
        finished.is_terminal(),
        "build should reach a terminal state"
    );

    // Get the build back.
    let fetched = build_client.get().await.expect("get build");
    assert!(fetched.is_some());

    // Fetch the build log.
    let log = build_client.log().get().await.expect("get build log");
    assert!(log.is_some(), "finished build should have a log");

    // Fetch the build's OpenAPI definition (simple GET; may be None if not generated).
    let _openapi = build_client
        .get_openapi_definition()
        .await
        .expect("get openapi definition");

    // Clean up.
    actor_client.delete().await.expect("delete actor");
}
