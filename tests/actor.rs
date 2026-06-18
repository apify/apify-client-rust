//! Integration tests for the Actor resource (collection + CRUD).
//!
//! These create a private Actor on the test account, build it, and clean it up.

mod common;

use serde_json::json;

/// Simple GET: listing the account's Actors.
#[tokio::test]
async fn list_actors() {
    let client = require_client!();
    let page = client
        .actors()
        .list(Default::default())
        .await
        .expect("listing actors should succeed");
    assert!(page.total >= 0);
}

/// Complex flow: create an Actor with a single version, get it, update it, list builds,
/// and delete it.
#[tokio::test]
async fn actor_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("actor").replace('-', ""); // actor names are stricter
    let name = format!("a{}", &name[..name.len().min(20)]);

    // Minimal Actor definition with one inline source-files version.
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
                    "content": "console.log('hello from rust client test');"
                }
            ]
        }]
    });

    let actor = client
        .actors()
        .create(&definition)
        .await
        .expect("create actor");
    assert_eq!(actor.name.as_deref(), Some(name.as_str()));

    let actor_client = client.actor(&actor.id);

    // Get.
    let fetched = actor_client.get().await.expect("get actor");
    assert!(fetched.is_some(), "created actor should be retrievable");

    // Update (set title).
    let updated = actor_client
        .update(&json!({ "title": "Rust client test actor" }))
        .await
        .expect("update actor");
    assert_eq!(updated.title.as_deref(), Some("Rust client test actor"));

    // List builds (should succeed even if empty).
    let builds = actor_client
        .builds()
        .list(Default::default())
        .await
        .expect("list actor builds");
    assert!(builds.total >= 0);

    // List versions.
    let versions = actor_client
        .versions()
        .list(Default::default())
        .await
        .expect("list versions");
    assert!(versions.items.iter().any(|v| v.version_number == "0.0"));

    // Delete.
    actor_client.delete().await.expect("delete actor");
    assert!(actor_client
        .get()
        .await
        .expect("get after delete")
        .is_none());
}
