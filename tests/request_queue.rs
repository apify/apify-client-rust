//! Integration tests for the request queue resource.

mod common;

use apify_client::models::RequestQueueRequest;
use serde_json::json;

/// Simple GET: listing request queues should succeed.
#[tokio::test(flavor = "multi_thread")]
async fn list_request_queues() {
    let client = require_client!();
    let page = client
        .request_queues()
        .list(Default::default())
        .await
        .expect("listing request queues should succeed");
    assert!(page.total >= 0);
}

/// Simple GET: fetch a single request queue by ID.
#[tokio::test(flavor = "multi_thread")]
async fn get_request_queue() {
    let client = require_client!();
    let name = common::unique_name("rq-get");
    let queue = client
        .request_queues()
        .get_or_create(Some(&name))
        .await
        .expect("create queue");

    let cleanup_client = client.clone();
    let id = queue.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.request_queue(&id).delete().await;
    });

    let fetched = client
        .request_queue(&queue.id)
        .get()
        .await
        .expect("get queue by id")
        .expect("queue should exist");
    assert_eq!(fetched.id, queue.id);
}

/// Iteration: the request queue collection iterator yields a just-created queue across pages.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_request_queues() {
    let client = require_client!();
    let name = common::unique_name("rq-iter");
    let queue = client
        .request_queues()
        .get_or_create(Some(&name))
        .await
        .expect("create queue");

    let cleanup_client = client.clone();
    let id = queue.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.request_queue(&id).delete().await;
    });

    let target = queue.id.clone();
    assert!(
        common::iter_contains_eventually(
            || {
                client
                    .request_queues()
                    .iterate(apify_client::StorageListOptions {
                        desc: Some(true),
                        ..Default::default()
                    })
                    .with_chunk_size(5)
            },
            move |q| q.id == target,
        )
        .await,
        "request queue iteration should yield the created queue"
    );
}

/// Complex flow: create -> get -> add request -> read request -> list head -> update -> delete.
#[tokio::test(flavor = "multi_thread")]
async fn request_queue_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("rq");

    let queue = client
        .request_queues()
        .get_or_create(Some(&name))
        .await
        .expect("create queue");
    assert_eq!(queue.name.as_deref(), Some(name.as_str()));

    let cleanup_client = client.clone();
    let cleanup_id = queue.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.request_queue(&cleanup_id).delete().await;
    });

    let queue_client = client.request_queue(&queue.id);

    // Get.
    assert!(queue_client.get().await.expect("get queue").is_some());

    // Add a request.
    let request = RequestQueueRequest {
        id: None,
        url: "https://example.com/".to_string(),
        unique_key: Some("example".to_string()),
        method: Some("GET".to_string()),
        user_data: Some(json!({ "label": "START" })),
        extra: Default::default(),
    };
    let added = queue_client
        .add_request(&request, false)
        .await
        .expect("add request");
    assert!(!added.request_id.is_empty());

    // Read the request back.
    let fetched = queue_client
        .get_request(&added.request_id)
        .await
        .expect("get request")
        .expect("request should exist");
    assert_eq!(fetched.url, "https://example.com/");

    // List the head.
    let head = queue_client.list_head(Some(10)).await.expect("list head");
    assert!(head
        .items
        .iter()
        .any(|r| r.id.as_deref() == Some(added.request_id.as_str())));

    // Update (rename the queue).
    let renamed = common::unique_name("rq-renamed");
    let updated = queue_client
        .update(&json!({ "name": renamed }))
        .await
        .expect("update queue");
    assert_eq!(updated.name.as_deref(), Some(renamed.as_str()));

    // Delete the request, then the queue.
    queue_client
        .delete_request(&added.request_id)
        .await
        .expect("delete request");
    queue_client.delete().await.expect("delete queue");
    assert!(queue_client
        .get()
        .await
        .expect("get after delete")
        .is_none());
}

/// Paginates across multiple pages: add N requests with a page limit < N and assert the
/// iterator yields all N exactly once. This exercises the second-page `cursor` path, which is
/// broken if the iterator feeds `nextCursor` back as `exclusiveStartId` instead of `cursor`.
#[tokio::test(flavor = "multi_thread")]
async fn request_queue_paginate_multiple_pages() {
    let client = require_client!();
    let name = common::unique_name("rq-page");

    let queue = client
        .request_queues()
        .get_or_create(Some(&name))
        .await
        .expect("create queue");

    let cleanup_client = client.clone();
    let cleanup_id = queue.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.request_queue(&cleanup_id).delete().await;
    });

    let queue_client = client.request_queue(&queue.id);

    // Add several requests (more than the page size used below).
    const TOTAL: usize = 5;
    let mut expected_urls = std::collections::HashSet::new();
    for i in 0..TOTAL {
        let url = format!("https://example.com/page/{i}");
        let request = RequestQueueRequest {
            id: None,
            url: url.clone(),
            unique_key: Some(format!("page-{i}")),
            method: Some("GET".to_string()),
            user_data: None,
            extra: Default::default(),
        };
        queue_client
            .add_request(&request, false)
            .await
            .expect("add request");
        expected_urls.insert(url);
    }

    // Paginate with a page size of 2, forcing several pages (and thus the cursor path).
    let mut iter = queue_client.paginate_requests(Some(2));
    let mut seen = std::collections::HashSet::new();
    while let Some(req) = iter.next().await.expect("paginate page") {
        // Each request must be yielded exactly once.
        assert!(
            seen.insert(req.url.clone()),
            "request {} yielded more than once (pagination cursor bug)",
            req.url
        );
    }

    assert_eq!(
        seen, expected_urls,
        "pagination must yield every added request exactly once across pages"
    );

    queue_client.delete().await.expect("delete queue");
}

/// Exercises the request lock lifecycle: add -> list_and_lock_head -> prolong -> unlock,
/// plus `list_requests` and `unlock_requests`.
#[tokio::test(flavor = "multi_thread")]
async fn request_queue_lock_lifecycle() {
    let client = require_client!();
    let name = common::unique_name("rq-lock");

    let queue = client
        .request_queues()
        .get_or_create(Some(&name))
        .await
        .expect("create queue");

    let cleanup_client = client.clone();
    let cleanup_id = queue.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.request_queue(&cleanup_id).delete().await;
    });

    // A stable client key lets us unlock our own locks.
    let queue_client = client
        .request_queue(&queue.id)
        .with_client_key("rust-test-client");

    // Add a request.
    let request = RequestQueueRequest {
        id: None,
        url: "https://example.com/lock".to_string(),
        unique_key: Some("lock-example".to_string()),
        method: Some("GET".to_string()),
        user_data: None,
        extra: Default::default(),
    };
    let added = queue_client
        .add_request(&request, false)
        .await
        .expect("add request");

    // List requests (cursor pagination endpoint).
    let listed = queue_client
        .list_requests(apify_client::ListRequestsOptions {
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("list requests");
    assert!(listed.get("items").is_some());

    // Exercise the `filter` parameter with both enum values (`locked`, `pending`). This verifies
    // the multi-value, comma-joined serialization (`filter=locked,pending`) is accepted by the API.
    let filtered = queue_client
        .list_requests(apify_client::ListRequestsOptions {
            limit: Some(10),
            filter: Some(vec!["locked".to_string(), "pending".to_string()]),
            ..Default::default()
        })
        .await
        .expect("list requests with filter");
    assert!(filtered.get("items").is_some());

    // Lazily paginate requests; we added one, so at least one should be yielded.
    let mut iter = queue_client.paginate_requests(Some(10));
    let first = iter.next().await.expect("paginate requests");
    assert!(first.is_some(), "pagination should yield the added request");

    // Lock the head.
    let locked = queue_client
        .list_and_lock_head(30, Some(5))
        .await
        .expect("lock head");
    assert!(locked.get("items").is_some());

    // Prolong, then release the lock on the added request.
    queue_client
        .prolong_request_lock(&added.request_id, 60, false)
        .await
        .expect("prolong lock");
    queue_client
        .delete_request_lock(&added.request_id, false)
        .await
        .expect("delete lock");

    // Unlock any remaining locks held by this client.
    queue_client
        .unlock_requests()
        .await
        .expect("unlock requests");

    queue_client.delete().await.expect("delete queue");
}
