//! Integration tests for the webhook and webhook-dispatch resources.

mod common;

use serde_json::json;

/// Simple GET: listing webhooks.
#[tokio::test]
async fn list_webhooks() {
    let client = require_client!();
    let page = client
        .webhooks()
        .list(Default::default())
        .await
        .expect("listing webhooks should succeed");
    assert!(page.total >= 0);
}

/// Simple GET: listing webhook dispatches.
#[tokio::test]
async fn list_webhook_dispatches() {
    let client = require_client!();
    let page = client
        .webhook_dispatches()
        .list(Default::default())
        .await
        .expect("listing webhook dispatches should succeed");
    assert!(page.total >= 0);
}

/// Complex flow: create -> get -> update -> delete a webhook.
#[tokio::test]
async fn webhook_crud_flow() {
    let client = require_client!();

    // A webhook that fires when any run of the public hello-world Actor succeeds.
    let definition = json!({
        "eventTypes": ["ACTOR.RUN.SUCCEEDED"],
        "condition": { "actorId": "moJRLRc85AitArpNN" },
        "requestUrl": "https://example.com/webhook",
        "isAdHoc": false
    });

    let webhook = client
        .webhooks()
        .create(&definition)
        .await
        .expect("create webhook");
    let webhook_client = client.webhook(&webhook.id);

    // Get.
    assert!(webhook_client.get().await.expect("get webhook").is_some());

    // Update (change request URL).
    let updated = webhook_client
        .update(&json!({ "requestUrl": "https://example.com/updated" }))
        .await
        .expect("update webhook");
    assert_eq!(
        updated.request_url.as_deref(),
        Some("https://example.com/updated")
    );

    // List this webhook's dispatches (should respond, likely empty).
    let dispatches = webhook_client
        .dispatches()
        .list(Default::default())
        .await
        .expect("list webhook dispatches");
    assert!(dispatches.total >= 0);

    // Trigger a test dispatch.
    let dispatch = webhook_client.test().await.expect("test webhook");
    assert!(!dispatch.id.is_empty(), "test dispatch should have an id");

    // Delete.
    webhook_client.delete().await.expect("delete webhook");
    assert!(webhook_client
        .get()
        .await
        .expect("get after delete")
        .is_none());
}
