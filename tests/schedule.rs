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

/// Complex flow: create -> get -> update -> delete a schedule.
#[tokio::test]
async fn schedule_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("schedule");

    let definition = json!({
        "name": name,
        "cronExpression": "0 0 * * *",
        "isEnabled": false,
        "isExclusive": true,
        "actions": []
    });

    let schedule = client
        .schedules()
        .create(&definition)
        .await
        .expect("create schedule");
    assert_eq!(schedule.name.as_deref(), Some(name.as_str()));

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
