//! Integration tests for the webhook and webhook-dispatch resources.

mod common;

use serde_json::json;

/// Simple GET: listing webhooks.
#[tokio::test(flavor = "multi_thread")]
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
#[tokio::test(flavor = "multi_thread")]
async fn list_webhook_dispatches() {
    let client = require_client!();
    let page = client
        .webhook_dispatches()
        .list(Default::default())
        .await
        .expect("listing webhook dispatches should succeed");
    assert!(page.total >= 0);
}

fn webhook_definition() -> serde_json::Value {
    // A webhook that fires when any run of the public hello-world Actor succeeds.
    json!({
        "eventTypes": ["ACTOR.RUN.SUCCEEDED"],
        "condition": { "actorId": "moJRLRc85AitArpNN" },
        "requestUrl": "https://example.com/webhook",
        "isAdHoc": false
    })
}

/// Simple GET: fetch a single webhook by ID.
#[tokio::test(flavor = "multi_thread")]
async fn get_webhook() {
    let client = require_client!();
    let webhook = client
        .webhooks()
        .create(&webhook_definition())
        .await
        .expect("create webhook");

    let cleanup_client = client.clone();
    let id = webhook.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.webhook(&id).delete().await;
    });

    let fetched = client
        .webhook(&webhook.id)
        .get()
        .await
        .expect("get webhook by id")
        .expect("webhook should exist");
    assert_eq!(fetched.id, webhook.id);
}

/// Simple GET: fetch a single webhook dispatch by ID.
///
/// Creates a webhook, triggers a test dispatch to obtain a real dispatch ID, then exercises
/// `webhook_dispatch(id).get()`.
#[tokio::test(flavor = "multi_thread")]
async fn get_webhook_dispatch() {
    let client = require_client!();
    let webhook = client
        .webhooks()
        .create(&webhook_definition())
        .await
        .expect("create webhook");

    let cleanup_client = client.clone();
    let id = webhook.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.webhook(&id).delete().await;
    });

    let dispatch = client
        .webhook(&webhook.id)
        .test()
        .await
        .expect("test webhook");
    assert!(!dispatch.id.is_empty());

    let fetched = client
        .webhook_dispatch(&dispatch.id)
        .get()
        .await
        .expect("get webhook dispatch by id")
        .expect("dispatch should exist");
    assert_eq!(fetched.id, dispatch.id);
}

/// Complex flow: create -> get -> update -> delete a webhook.
#[tokio::test(flavor = "multi_thread")]
async fn webhook_crud_flow() {
    let client = require_client!();

    let webhook = client
        .webhooks()
        .create(&webhook_definition())
        .await
        .expect("create webhook");

    let cleanup_client = client.clone();
    let cleanup_id = webhook.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.webhook(&cleanup_id).delete().await;
    });

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
