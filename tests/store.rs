//! Integration tests for the Apify Store resource.

mod common;

use apify_client::clients::store_collection::StoreListOptions;

/// Simple GET: listing Store Actors.
#[tokio::test]
async fn list_store() {
    let client = require_client!();
    let page = client
        .store()
        .list(StoreListOptions {
            limit: Some(5),
            ..Default::default()
        })
        .await
        .expect("listing the store should succeed");
    assert!(page.items.len() <= 5);
}

/// Convenience: lazy iteration across Store pages.
#[tokio::test]
async fn iterate_store() {
    let client = require_client!();
    let mut iter = client.store().iterate(StoreListOptions {
        limit: Some(5),
        ..Default::default()
    });

    // Pull a handful of items; the iterator should fetch pages transparently.
    let mut seen = 0;
    while let Some(_actor) = iter.next().await.expect("iterate store") {
        seen += 1;
        if seen >= 12 {
            break;
        }
    }
    assert!(seen > 0, "store iteration should yield at least one actor");
}
