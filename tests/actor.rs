//! Integration tests for the Actor resource (collection + CRUD).
//!
//! These create a private Actor on the test account, build it, and clean it up.

mod common;

use serde_json::json;

/// Simple GET: listing the account's Actors.
#[tokio::test(flavor = "multi_thread")]
async fn list_actors() {
    let client = require_client!();
    let page = client
        .actors()
        .list(Default::default())
        .await
        .expect("listing actors should succeed");
    assert!(page.total >= 0);
}

fn actor_name(prefix: &str) -> String {
    let name = common::unique_name(prefix).replace('-', ""); // actor names are stricter
    format!("a{}", &name[..name.len().min(20)])
}

/// Minimal Actor definition with one inline source-files version.
fn actor_definition(name: &str) -> serde_json::Value {
    json!({
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
    })
}

/// Simple GET: fetch a single Actor by ID.
#[tokio::test(flavor = "multi_thread")]
async fn get_actor() {
    let client = require_client!();
    let name = actor_name("actor-get");
    let actor = client
        .actors()
        .create(&actor_definition(&name))
        .await
        .expect("create actor");

    let cleanup_client = client.clone();
    let id = actor.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.actor(&id).delete().await;
    });

    let fetched = client
        .actor(&actor.id)
        .get()
        .await
        .expect("get actor by id")
        .expect("actor should exist");
    assert_eq!(fetched.id, actor.id);
}

/// Complex flow: create an Actor with a single version, get it, update it, list builds,
/// and delete it.
#[tokio::test(flavor = "multi_thread")]
async fn actor_crud_flow() {
    let client = require_client!();
    let name = actor_name("actor");

    let actor = client
        .actors()
        .create(&actor_definition(&name))
        .await
        .expect("create actor");
    assert_eq!(actor.name.as_deref(), Some(name.as_str()));

    let cleanup_client = client.clone();
    let cleanup_id = actor.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.actor(&cleanup_id).delete().await;
    });

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

/// Actor versions: create -> get -> list -> update -> delete on a fresh Actor.
#[tokio::test(flavor = "multi_thread")]
async fn actor_version_crud_flow() {
    use apify_client::models::ActorVersion;

    let client = require_client!();
    let name = actor_name("ver");

    // Create the Actor with its initial 0.0 version.
    let actor = client
        .actors()
        .create(&actor_definition(&name))
        .await
        .expect("create actor");

    let cleanup_client = client.clone();
    let cleanup_id = actor.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.actor(&cleanup_id).delete().await;
    });

    let actor_client = client.actor(&actor.id);

    // Create a new version (0.1) via the collection.
    let new_version = json!({
        "versionNumber": "0.1",
        "sourceType": "SOURCE_FILES",
        "buildTag": "beta",
        "sourceFiles": [
            {
                "name": "Dockerfile",
                "format": "TEXT",
                "content": "FROM apify/actor-node:20\nCOPY . ./\nCMD node main.js"
            },
            {
                "name": "main.js",
                "format": "TEXT",
                "content": "console.log('v0.1');"
            }
        ]
    });
    let created = actor_client
        .versions()
        .create(&new_version)
        .await
        .expect("create version");
    assert_eq!(created.version_number, "0.1");

    // Simple GET: fetch the version by its number.
    let version_client = actor_client.version("0.1");
    let fetched = version_client
        .get()
        .await
        .expect("get version")
        .expect("version 0.1 should exist");
    assert_eq!(fetched.version_number, "0.1");

    // List should include both 0.0 and 0.1.
    let versions = actor_client
        .versions()
        .list(Default::default())
        .await
        .expect("list versions");
    assert!(versions.items.iter().any(|v| v.version_number == "0.1"));

    // Update the version (change its build tag).
    let updated = version_client
        .update(&ActorVersion {
            version_number: "0.1".to_string(),
            source_type: Some("SOURCE_FILES".to_string()),
            extra: serde_json::from_value(json!({
                "buildTag": "latest",
                "sourceFiles": [
                    {
                        "name": "Dockerfile",
                        "format": "TEXT",
                        "content": "FROM apify/actor-node:20\nCOPY . ./\nCMD node main.js"
                    },
                    { "name": "main.js", "format": "TEXT", "content": "console.log('v0.1');" }
                ]
            }))
            .expect("build version extra"),
        })
        .await
        .expect("update version");
    assert_eq!(updated.version_number, "0.1");

    // Delete the version.
    version_client.delete().await.expect("delete version");
    assert!(
        actor_client
            .version("0.1")
            .get()
            .await
            .expect("get after delete")
            .is_none(),
        "version 0.1 should be gone after delete"
    );
}

/// Actor version environment variables: create -> get -> list -> update -> delete.
#[tokio::test(flavor = "multi_thread")]
async fn actor_env_var_crud_flow() {
    use apify_client::models::ActorEnvVar;

    let client = require_client!();
    let name = actor_name("env");

    let actor = client
        .actors()
        .create(&actor_definition(&name))
        .await
        .expect("create actor");

    let cleanup_client = client.clone();
    let cleanup_id = actor.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.actor(&cleanup_id).delete().await;
    });

    let version_client = client.actor(&actor.id).version("0.0");

    // Create an environment variable on version 0.0.
    let created = version_client
        .env_vars()
        .create(&ActorEnvVar {
            name: "MY_VAR".to_string(),
            value: Some("hello".to_string()),
            is_secret: Some(false),
            extra: Default::default(),
        })
        .await
        .expect("create env var");
    assert_eq!(created.name, "MY_VAR");

    // Simple GET: fetch the env var by name.
    let env_var_client = version_client.env_var("MY_VAR");
    let fetched = env_var_client
        .get()
        .await
        .expect("get env var")
        .expect("env var should exist");
    assert_eq!(fetched.name, "MY_VAR");

    // List should include it.
    let env_vars = version_client
        .env_vars()
        .list()
        .await
        .expect("list env vars");
    assert!(env_vars.items.iter().any(|e| e.name == "MY_VAR"));

    // Update its value.
    let updated = env_var_client
        .update(&ActorEnvVar {
            name: "MY_VAR".to_string(),
            value: Some("world".to_string()),
            is_secret: Some(false),
            extra: Default::default(),
        })
        .await
        .expect("update env var");
    assert_eq!(updated.name, "MY_VAR");

    // Delete it.
    env_var_client.delete().await.expect("delete env var");
    assert!(
        version_client
            .env_var("MY_VAR")
            .get()
            .await
            .expect("get after delete")
            .is_none(),
        "env var should be gone after delete"
    );
}
