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
    // Load-bearing consistency check (unlike a tautological `total >= 0`, which is `i64` and
    // always true): a single page can never return more items than its own `limit`. Checked
    // against `limit`, not `total`, because this suite runs many tests concurrently against the
    // same shared account; `total` and `items` can be computed from separately-timed backend
    // reads under that write load, which made an `items.len() <= total` check here genuinely
    // flaky (observed under `cargo test --all-targets`), not just load-bearing.
    assert!(page.items.len() as i64 <= page.limit);
}

/// Iteration: the build collection iterator yields a build we just started.
///
/// Scoped to a fresh Actor's builds so the collection is small and deterministic. The build is
/// started but not awaited — it appears in the listing immediately regardless of its state.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_builds() {
    let client = require_client!();
    let name = common::short_unique_name('b', "build-iter", 21);

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

    let target = build.id.clone();
    assert!(
        common::iter_contains_eventually(
            || {
                actor_client
                    .builds()
                    .iterate(apify_client::ListOptions {
                        desc: Some(true),
                        ..Default::default()
                    })
                    .with_chunk_size(5)
            },
            move |b| b.id == target,
        )
        .await,
        "build iteration should yield the started build"
    );
}

/// Complex flow: create an Actor, build it, wait for the build to finish, fetch the build
/// and its log, then clean up.
#[tokio::test(flavor = "multi_thread")]
async fn build_actor_flow() {
    let client = require_client!();
    let name = common::short_unique_name('b', "build", 21);

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
    // build now exists (built above), so `build=latest` resolves to a concrete artifact. An
    // empty object is a valid input for an Actor with no required schema fields, so asserting
    // `true` proves the call hit the endpoint with the param accepted and did not return an
    // error envelope.
    let is_valid = actor_client
        .validate_input_for_build(&json!({}), Some("latest"))
        .await
        .expect("validate input for latest build");
    assert!(
        is_valid,
        "validate-input should report the empty-object input as valid"
    );

    // Clean up.
    actor_client.delete().await.expect("delete actor");
}

/// `BuildClient::abort()` and `BuildClient::delete()`: start a deliberately slow build, abort it
/// mid-build, then delete it. A trivial build finishes too fast to reliably hit the RUNNING
/// state, so this Actor's Dockerfile sleeps before the (never-reached) `COPY`/`CMD` steps.
#[tokio::test(flavor = "multi_thread")]
async fn build_abort_and_delete() {
    let client = require_client!();
    let name = common::short_unique_name('b', "build-abort", 21);

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
                    "content": "FROM apify/actor-node:20\nRUN sleep 90\nCOPY . ./\nCMD node main.js"
                },
                { "name": "main.js", "format": "TEXT", "content": "console.log('unreachable');" }
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
        .expect("start slow build");
    let build_client = client.build(&build.id);

    let aborted = build_client.abort().await.expect("abort build");
    assert!(
        matches!(
            aborted.status.as_deref(),
            Some("ABORTING") | Some("ABORTED")
        ),
        "expected ABORTING or ABORTED after abort, got {:?}",
        aborted.status
    );

    let finished = build_client
        .wait_for_finish(Some(60))
        .await
        .expect("wait for aborted build to settle");
    assert_eq!(finished.status.as_deref(), Some("ABORTED"));

    build_client.delete().await.expect("delete build");
    assert!(build_client
        .get()
        .await
        .expect("get build after delete")
        .is_none());
}
