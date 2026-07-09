//! Integration tests for the Apify Store resource.

mod common;

use apify_client::clients::store_collection::StoreListOptions;
use futures_util::StreamExt;

/// Simple GET: listing Store Actors.
#[tokio::test(flavor = "multi_thread")]
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
#[tokio::test(flavor = "multi_thread")]
async fn iterate_store() {
    let client = require_client!();
    let stream = client.store().iterate(StoreListOptions {
        limit: Some(5),
        ..Default::default()
    });
    futures_util::pin_mut!(stream);

    // Pull a handful of items; the stream should fetch pages transparently.
    let mut seen = 0;
    while let Some(actor) = stream.next().await {
        actor.expect("iterate store");
        seen += 1;
        if seen >= 12 {
            break;
        }
    }
    assert!(seen > 0, "store iteration should yield at least one actor");
}
