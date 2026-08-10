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
    // Load-bearing consistency check (unlike a tautological `total >= 0`, which is `i64` and
    // always true): a single page can never return more items than its own `limit`. Checked
    // against `limit`, not `total`, because this suite runs many tests concurrently against the
    // same shared account; `total` and `items` can be computed from separately-timed backend
    // reads under that write load, which made an `items.len() <= total` check here genuinely
    // flaky (observed under `cargo test --all-targets`), not just load-bearing.
    assert!(page.items.len() as i64 <= page.limit);
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
    // Load-bearing consistency check (unlike a tautological `total >= 0`, which is `i64` and
    // always true): a single page can never return more items than its own `limit`. Checked
    // against `limit`, not `total`, because this suite runs many tests concurrently against the
    // same shared account; `total` and `items` can be computed from separately-timed backend
    // reads under that write load, which made an `items.len() <= total` check here genuinely
    // flaky (observed under `cargo test --all-targets`), not just load-bearing.
    assert!(page.items.len() as i64 <= page.limit);
}

/// A webhook that fires when any run of the public hello-world Actor succeeds.
///
/// `tag` is embedded in `requestUrl` (as a query parameter, so it stays a structurally valid
/// URL) to make each test's webhook unique, matching the suite's UUID-isolation convention used
/// by every other resource (`unique_name`/`short_unique_name`). Without it, every one of this
/// function's 5 callers created byte-for-byte identical webhooks, which is harmless for the API
/// (webhooks have no uniqueness constraint) but means a bug that leaked one test's webhook into
/// another's assertions (e.g. via a stale/undeleted resource from a prior failed run) could not
/// be told apart from the current test's own webhook.
fn webhook_definition(tag: &str) -> serde_json::Value {
    let unique = common::unique_name(tag);
    json!({
        "eventTypes": ["ACTOR.RUN.SUCCEEDED"],
        "condition": { "actorId": "moJRLRc85AitArpNN" },
        "requestUrl": format!("https://example.com/webhook?test={unique}"),
        "isAdHoc": false
    })
}

/// Simple GET: fetch a single webhook by ID.
#[tokio::test(flavor = "multi_thread")]
async fn get_webhook() {
    let client = require_client!();
    let webhook = client
        .webhooks()
        .create(&webhook_definition("get-webhook"))
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
        .create(&webhook_definition("get-webhook-dispatch"))
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
        .expect("test webhook")
        .expect("webhook was just created, so it must still exist");
    assert!(!dispatch.id.is_empty());

    let fetched = client
        .webhook_dispatch(&dispatch.id)
        .get()
        .await
        .expect("get webhook dispatch by id")
        .expect("dispatch should exist");
    assert_eq!(fetched.id, dispatch.id);
}

/// Iteration: the webhook collection iterator yields a just-created webhook across pages.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_webhooks() {
    let client = require_client!();
    let webhook = client
        .webhooks()
        .create(&webhook_definition("iterate-webhooks"))
        .await
        .expect("create webhook");

    let cleanup_client = client.clone();
    let id = webhook.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.webhook(&id).delete().await;
    });

    let target = webhook.id.clone();
    assert!(
        common::iter_contains_eventually(
            || {
                client
                    .webhooks()
                    .iterate(apify_client::ListOptions {
                        desc: Some(true),
                        ..Default::default()
                    })
                    .with_chunk_size(5)
            },
            move |w| w.id == target,
        )
        .await,
        "webhook iteration should yield the created webhook"
    );
}

/// Iteration: the webhook-dispatch collection iterator yields a dispatch we just triggered.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_webhook_dispatches() {
    let client = require_client!();
    let webhook = client
        .webhooks()
        .create(&webhook_definition("iterate-webhook-dispatches"))
        .await
        .expect("create webhook");

    let cleanup_client = client.clone();
    let id = webhook.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.webhook(&id).delete().await;
    });

    // Trigger a real dispatch so there is a known dispatch id to find.
    let dispatch = client
        .webhook(&webhook.id)
        .test()
        .await
        .expect("test webhook")
        .expect("webhook was just created, so it must still exist");
    assert!(!dispatch.id.is_empty());

    let target = dispatch.id.clone();
    assert!(
        common::iter_contains_eventually(
            || {
                client
                    .webhook_dispatches()
                    .iterate(apify_client::ListOptions {
                        desc: Some(true),
                        ..Default::default()
                    })
                    .with_chunk_size(5)
            },
            move |d| d.id == target,
        )
        .await,
        "webhook-dispatch iteration should yield the triggered dispatch"
    );
}

/// Complex flow: create -> get -> update -> delete a webhook.
#[tokio::test(flavor = "multi_thread")]
async fn webhook_crud_flow() {
    let client = require_client!();

    let webhook = client
        .webhooks()
        .create(&webhook_definition("webhook-crud-flow"))
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

    // List this webhook's dispatches. No dispatch has fired yet (the test dispatch below is
    // triggered after this), so the collection must be genuinely empty — a load-bearing
    // assertion, unlike a tautological `total >= 0`.
    let dispatches = webhook_client
        .dispatches()
        .list(Default::default())
        .await
        .expect("list webhook dispatches");
    assert_eq!(
        dispatches.total, 0,
        "a webhook with no triggered dispatch should have none listed"
    );
    assert_eq!(dispatches.total as usize, dispatches.items.len());

    // Trigger a test dispatch.
    let dispatch = webhook_client
        .test()
        .await
        .expect("test webhook")
        .expect("webhook was just created, so it must still exist");
    assert!(!dispatch.id.is_empty(), "test dispatch should have an id");

    // Delete.
    webhook_client.delete().await.expect("delete webhook");
    assert!(webhook_client
        .get()
        .await
        .expect("get after delete")
        .is_none());
}
