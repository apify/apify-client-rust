//! Integration tests for the Actor task resource.

mod common;

use serde_json::json;

/// Simple GET: listing tasks.
#[tokio::test]
async fn list_tasks() {
    let client = require_client!();
    let page = client
        .tasks()
        .list(Default::default())
        .await
        .expect("listing tasks should succeed");
    assert!(page.total >= 0);
}

/// Complex flow: create a task for the public hello-world Actor, get it, update its input,
/// list its runs, and delete it.
#[tokio::test]
async fn task_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("task");

    let definition = json!({
        "actId": "apify/hello-world",
        "name": name,
        "options": { "memoryMbytes": 256 },
        "input": { "message": "hi" }
    });

    let task = client
        .tasks()
        .create(&definition)
        .await
        .expect("create task");
    assert_eq!(task.name.as_deref(), Some(name.as_str()));

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
        .list(Default::default(), None)
        .await
        .expect("list task runs");
    assert!(runs.total >= 0);

    // Delete.
    task_client.delete().await.expect("delete task");
    assert!(task_client.get().await.expect("get after delete").is_none());
}
