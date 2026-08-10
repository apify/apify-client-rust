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
/// requires write permission over its Actor (here the shared, Apify-owned `apify/hello-world`,
/// per [`task_definition`]), so the API is expected to reject it with `insufficient-permissions` -
/// this exercises the same request path without needing a private Actor the test account can
/// actually publish (which would additionally require an SEO description, an input field
/// selection, and a dataset view set up on the task's public display configuration).
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
    assert_eq!(unpublished.extra.get("isPublic"), Some(&json!(false)));

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
