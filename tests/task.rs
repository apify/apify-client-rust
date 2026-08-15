//! Integration tests for the Actor task resource.

mod common;

use serde_json::json;

/// Simple GET: listing tasks.
#[tokio::test(flavor = "multi_thread")]
async fn list_tasks() {
    let client = require_client!();
    let page = client
        .tasks()
        .list(Default::default())
        .await
        .expect("listing tasks should succeed");
    assert!(page.total >= 0);
}

fn task_definition(name: &str) -> serde_json::Value {
    json!({
        "actId": "apify/hello-world",
        "name": name,
        "options": { "memoryMbytes": 256 },
        "input": { "message": "hi" }
    })
}

/// Simple GET: fetch a single task by ID.
#[tokio::test(flavor = "multi_thread")]
async fn get_task() {
    let client = require_client!();
    let name = common::unique_name("task-get");
    let task = client
        .tasks()
        .create(&task_definition(&name))
        .await
        .expect("create task");

    let cleanup_client = client.clone();
    let id = task.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.task(&id).delete().await;
    });

    let fetched = client
        .task(&task.id)
        .get()
        .await
        .expect("get task by id")
        .expect("task should exist");
    assert_eq!(fetched.id, task.id);
}

/// Iteration: the task collection iterator yields a just-created task across pages.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_tasks() {
    let client = require_client!();
    let name = common::unique_name("task-iter");
    let task = client
        .tasks()
        .create(&task_definition(&name))
        .await
        .expect("create task");

    let cleanup_client = client.clone();
    let id = task.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.task(&id).delete().await;
    });

    let target = task.id.clone();
    assert!(
        common::iter_contains_eventually(
            || {
                client
                    .tasks()
                    .iterate(apify_client::ListOptions {
                        desc: Some(true),
                        ..Default::default()
                    })
                    .with_chunk_size(5)
            },
            move |t| t.id == target,
        )
        .await,
        "task iteration should yield the created task"
    );
}

/// `publish`/`unpublish` are thin wrappers around `update` that flip `isPublic`. Unpublishing an
/// already-unpublished task is a documented no-op, so it round-trips cleanly; publishing a task
/// (and, per the spec, editing `publicConfig` at all) requires write permission over its Actor.
/// Here that's the shared, Apify-owned `apify/hello-world` (per [`task_definition`]), so both are
/// expected to be rejected with `insufficient-permissions` - this exercises the same request path
/// without needing a private Actor the test account can actually publish. See
/// [`task_public_config_update`] for the `publicConfig`-editing path against an owned Actor.
#[tokio::test(flavor = "multi_thread")]
async fn task_publish_unpublish() {
    let client = require_client!();
    let name = common::unique_name("task-publish");
    let task = client
        .tasks()
        .create(&task_definition(&name))
        .await
        .expect("create task");

    let cleanup_client = client.clone();
    let id = task.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.task(&id).delete().await;
    });

    let task_client = client.task(&task.id);

    let unpublished = task_client
        .unpublish()
        .await
        .expect("unpublish an already-unpublished task should be a no-op");
    assert_eq!(unpublished.is_public, Some(false));

    match task_client.publish().await {
        Err(apify_client::ApifyClientError::Api(err)) => {
            assert_eq!(err.status_code, 403);
            assert_eq!(err.error_type.as_deref(), Some("insufficient-permissions"));
        }
        other => panic!(
            "expected publish() on a task for an Actor we don't own to fail with \
             `insufficient-permissions`, got {other:?}"
        ),
    }
}

/// Minimal private-Actor definition, owned by the test account (unlike the shared
/// `apify/hello-world` used elsewhere in this file), so the test account has write permission
/// to edit a task's `publicConfig`.
fn owned_actor_definition(name: &str) -> serde_json::Value {
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

/// Exercises `Task::public_config` (`TaskPublicConfig`): editing it via `update` requires write
/// permission to the task's Actor, so this uses a private Actor owned by the test account. Sets
/// the `seoTitle`/`seoDescription` metadata fields and reads them back typed - `inputSchemaFields`/
/// `datasetView` are omitted since the API validates them against the Actor's input/dataset
/// schema (which this minimal Actor doesn't declare) even on a plain `publicConfig` edit, not
/// only on publish. Also checks that `unpublish` (a no-op here, since the task was never
/// published) leaves `publicConfig` untouched.
///
/// Task creation resolves the Actor's default (`latest`) build tag up front, so - unlike the
/// other Actor-owning tests in this suite, which only need the Actor to exist - the Actor must
/// actually be built first (mirrors `tests/build.rs::build_actor_flow`'s build-and-wait).
#[tokio::test(flavor = "multi_thread")]
async fn task_public_config_update() {
    let client = require_client!();

    let actor_name = common::unique_name("task-pubcfg-actor").replace('-', "");
    let actor = client
        .actors()
        .create(&owned_actor_definition(
            &actor_name[..actor_name.len().min(24)],
        ))
        .await
        .expect("create owned actor");
    let cleanup_actor_client = client.clone();
    let actor_id = actor.id.clone();
    let _actor_guard = common::Cleanup::new(move || async move {
        let _ = cleanup_actor_client.actor(&actor_id).delete().await;
    });

    let build = client
        .actor(&actor.id)
        .build("0.0", Default::default())
        .await
        .expect("start build");
    let finished = client
        .build(&build.id)
        .wait_for_finish(Some(300))
        .await
        .expect("wait for build");
    assert!(
        finished.is_terminal(),
        "build should reach a terminal state before the task can reference it"
    );

    let task_name = common::unique_name("task-pubcfg");
    let task = client
        .tasks()
        .create(&json!({ "actId": actor.id, "name": task_name }))
        .await
        .expect("create task for owned actor");
    let cleanup_task_client = client.clone();
    let task_id = task.id.clone();
    let _task_guard = common::Cleanup::new(move || async move {
        let _ = cleanup_task_client.task(&task_id).delete().await;
    });

    let task_client = client.task(&task.id);

    let configured = task_client
        .update(&json!({
            "publicConfig": {
                "seoTitle": "Test task",
                "seoDescription": "A test task.",
            }
        }))
        .await
        .expect("set publicConfig on an owned actor's task");
    let public_config = configured
        .public_config
        .expect("publicConfig should be set after update");
    assert_eq!(public_config.seo_title.as_deref(), Some("Test task"));
    assert_eq!(
        public_config.seo_description.as_deref(),
        Some("A test task.")
    );
    assert_eq!(public_config.published_at, None);
    assert_eq!(configured.is_public, Some(false));

    let unpublished = task_client
        .unpublish()
        .await
        .expect("unpublish an already-unpublished task should be a no-op");
    assert_eq!(unpublished.is_public, Some(false));
    assert_eq!(
        unpublished
            .public_config
            .as_ref()
            .and_then(|c| c.seo_title.as_deref()),
        Some("Test task"),
        "unpublish must not disturb the previously-configured publicConfig"
    );
}

/// Complex flow: create a task for the public hello-world Actor, get it, update its input,
/// list its runs, and delete it.
#[tokio::test(flavor = "multi_thread")]
async fn task_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("task");

    let task = client
        .tasks()
        .create(&task_definition(&name))
        .await
        .expect("create task");
    assert_eq!(task.name.as_deref(), Some(name.as_str()));

    let cleanup_client = client.clone();
    let cleanup_id = task.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.task(&cleanup_id).delete().await;
    });

    let task_client = client.task(&task.id);

    // Get.
    assert!(task_client.get().await.expect("get task").is_some());

    // Update input.
    let new_input = json!({ "message": "updated" });
    task_client
        .update_input(&new_input)
        .await
        .expect("update input");
    let input = task_client.get_input().await.expect("get input");
    assert!(input.is_some());

    // Update (rename).
    let renamed = common::unique_name("task-renamed");
    let updated = task_client
        .update(&json!({ "name": renamed }))
        .await
        .expect("update task");
    assert_eq!(updated.name.as_deref(), Some(renamed.as_str()));

    // List its runs (likely empty, but the endpoint should respond).
    let runs = task_client
        .runs()
        .list(Default::default(), Default::default())
        .await
        .expect("list task runs");
    assert!(runs.total >= 0);

    // Delete.
    task_client.delete().await.expect("delete task");
    assert!(task_client.get().await.expect("get after delete").is_none());
}
