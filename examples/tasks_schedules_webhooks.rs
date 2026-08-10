//! Create an Actor task, a schedule that runs it, and a webhook that watches it.
//!
//! Demonstrates the create payload shape for all three resources (documented in
//! [`docs/tasks.md`](../docs/tasks.md), [`docs/schedules.md`](../docs/schedules.md) and
//! [`docs/webhooks.md`](../docs/webhooks.md)), plus each resource's basic lifecycle.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example tasks_schedules_webhooks`

use apify_client::ApifyClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // ---- Task: create a task that runs the public hello-world Actor ----
    let task = client
        .tasks()
        .create(&json!({
            "actId": "apify/hello-world",
            "name": format!("rust-example-task-{}", uuid_like_suffix()),
            "options": { "memoryMbytes": 256 },
            "input": { "message": "hi from the tasks/schedules/webhooks example" }
        }))
        .await?;
    println!("Created task {}", task.id);
    let task_client = client.task(&task.id);

    // ---- Schedule: run the task on a (disabled, so this example doesn't actually fire it) cron ----
    let schedule = client
        .schedules()
        .create(&json!({
            "name": format!("rust-example-schedule-{}", uuid_like_suffix()),
            "cronExpression": "0 12 * * *",
            "isEnabled": false,
            "isExclusive": true,
            "actions": [
                { "type": "RUN_ACTOR_TASK", "actorTaskId": task.id }
            ]
        }))
        .await?;
    println!("Created schedule {}", schedule.id);
    let schedule_client = client.schedule(&schedule.id);

    // A schedule that has never fired has no invocation log yet.
    let log = schedule_client.get_log().await?;
    println!("Schedule log present: {}", log.is_some());

    // ---- Webhook: notify a URL whenever this task's runs succeed ----
    let webhook = client
        .webhooks()
        .create(&json!({
            "eventTypes": ["ACTOR.RUN.SUCCEEDED"],
            "condition": { "actorTaskId": task.id },
            "requestUrl": "https://example.com/webhook"
        }))
        .await?;
    println!("Created webhook {}", webhook.id);
    let webhook_client = client.webhook(&webhook.id);

    // Trigger a test dispatch (does not require a real run) and list this webhook's dispatches.
    let dispatch = webhook_client
        .test()
        .await?
        .expect("webhook was just created, so it must still exist");
    println!("Test dispatch {}", dispatch.id);
    let dispatches = webhook_client.dispatches().list(Default::default()).await?;
    println!("Webhook has {} dispatch(es)", dispatches.total);

    // The task's own webhook sub-collection lists webhooks scoped to it (may be empty here since
    // the webhook above was created top-level, not via `task.webhooks()`).
    let task_webhooks = task_client.webhooks().list(Default::default()).await?;
    println!(
        "Task-scoped webhook collection has {} item(s)",
        task_webhooks.total
    );

    // ---- Update each resource, then clean up ----
    task_client
        .update(&json!({ "name": format!("{}-renamed", task.name.as_deref().unwrap_or("task")) }))
        .await?;
    schedule_client
        .update(&json!({ "isEnabled": false, "cronExpression": "30 12 * * *" }))
        .await?;
    webhook_client
        .update(&json!({ "requestUrl": "https://example.com/webhook-updated" }))
        .await?;

    webhook_client.delete().await?;
    schedule_client.delete().await?;
    task_client.delete().await?;
    println!("Cleaned up task, schedule and webhook");

    Ok(())
}

/// A short, timestamp-based suffix so repeated example runs don't collide on names.
fn uuid_like_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
