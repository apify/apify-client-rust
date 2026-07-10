//! Integration tests for the Actor build resource.

mod common;

use serde_json::json;

/// Simple GET: listing the account's builds.
#[tokio::test(flavor = "multi_thread")]
async fn list_builds() {
    let client = require_client!();
    let page = client
        .builds()
        .list(Default::default())
        .await
        .expect("listing builds should succeed");
    assert!(page.total >= 0);
}

/// Iteration: the build collection iterator yields a build we just started.
///
/// Scoped to a fresh Actor's builds so the collection is small and deterministic. The build is
/// started but not awaited — it appears in the listing immediately regardless of its state.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_builds() {
    let client = require_client!();
    let name = common::unique_name("build-iter").replace('-', "");
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
                { "name": "main.js", "format": "TEXT", "content": "console.log('iter');" }
            ]
        }]
    });

    let actor = client
        .actors()
        .create(&definition)
        .await
        .expect("create actor");

    let cleanup_client = client.clone();
    let cleanup_id = actor.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.actor(&cleanup_id).delete().await;
    });

    let actor_client = client.actor(&actor.id);
    let build = actor_client
        .build("0.0", Default::default())
        .await
        .expect("start build");

    let iter = actor_client.builds().iterate(apify_client::ListOptions {
        desc: Some(true),
        limit: Some(10),
        ..Default::default()
    });
    let target = build.id.clone();
    assert!(
        common::iter_contains(iter, move |b| b.id == target).await,
        "build iteration should yield the started build"
    );
}

/// Complex flow: create an Actor, build it, wait for the build to finish, fetch the build
/// and its log, then clean up.
#[tokio::test(flavor = "multi_thread")]
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

    let cleanup_client = client.clone();
    let cleanup_id = actor.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.actor(&cleanup_id).delete().await;
    });

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

    // Validate input against the just-built `latest` build, exercising the spec's optional
    // `build` query parameter on POST /v2/actors/{actorId}/validate-input. A real `latest`
    // build now exists (built above), so `build=latest` resolves to a concrete artifact.
    // The success response is an object with a `valid` boolean; asserting that field is present
    // (rather than merely `is_object()`) proves the call hit the endpoint with the param accepted
    // and did not return an error envelope.
    let validation = actor_client
        .validate_input_for_build(&json!({}), Some("latest"))
        .await
        .expect("validate input for latest build");
    assert!(
        validation.get("valid").and_then(|v| v.as_bool()).is_some(),
        "validate-input response should contain a `valid` boolean, got: {validation}"
    );

    // Clean up.
    actor_client.delete().await.expect("delete actor");
}
