//! Integration tests for the schedule resource.

mod common;

use serde_json::json;

/// Simple GET: listing schedules.
#[tokio::test]
async fn list_schedules() {
    let client = require_client!();
    let page = client
        .schedules()
        .list(Default::default())
        .await
        .expect("listing schedules should succeed");
    assert!(page.total >= 0);
}

fn schedule_definition(name: &str) -> serde_json::Value {
    json!({
        "name": name,
        "cronExpression": "0 0 * * *",
        "isEnabled": false,
        "isExclusive": true,
        "actions": []
    })
}

/// Simple GET: fetch a single schedule by ID.
#[tokio::test]
async fn get_schedule() {
    let client = require_client!();
    let name = common::unique_name("schedule-get");
    let schedule = client
        .schedules()
        .create(&schedule_definition(&name))
        .await
        .expect("create schedule");

    let cleanup_client = client.clone();
    let id = schedule.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.schedule(&id).delete().await;
    });

    let fetched = client
        .schedule(&schedule.id)
        .get()
        .await
        .expect("get schedule by id")
        .expect("schedule should exist");
    assert_eq!(fetched.id, schedule.id);
}

/// Complex flow: create -> get -> update -> delete a schedule.
#[tokio::test]
async fn schedule_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("schedule");

    let schedule = client
        .schedules()
        .create(&schedule_definition(&name))
        .await
        .expect("create schedule");
    assert_eq!(schedule.name.as_deref(), Some(name.as_str()));

    let cleanup_client = client.clone();
    let cleanup_id = schedule.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.schedule(&cleanup_id).delete().await;
    });

    let schedule_client = client.schedule(&schedule.id);

    // Get.
    assert!(schedule_client.get().await.expect("get schedule").is_some());

    // Update (change cron expression).
    let updated = schedule_client
        .update(&json!({ "cronExpression": "0 12 * * *" }))
        .await
        .expect("update schedule");
    assert_eq!(updated.cron_expression.as_deref(), Some("0 12 * * *"));

    // Fetch the schedule's invocation log (simple GET; may be empty for a new schedule).
    let _log = schedule_client.get_log().await.expect("get schedule log");

    // Delete.
    schedule_client.delete().await.expect("delete schedule");
    assert!(schedule_client
        .get()
        .await
        .expect("get after delete")
        .is_none());
}
