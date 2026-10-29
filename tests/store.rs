//! Integration tests for the Apify Store resource.

mod common;

use apify_client::clients::store_collection::StoreListOptions;

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
    // Page size of 5 (via `with_chunk_size`) so pulling 12 items forces multiple page fetches;
    // `limit` is a total-item cap and is left unset so iteration is not bounded to one page.
    let mut iter = client
        .store()
        .iterate(StoreListOptions::default())
        .with_chunk_size(5);

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

/// Simple GET: `include_input_schema` is accepted by the Store endpoint and adds the compact input
/// schema to the listed Actors (the API then caps `limit` at 10).
#[tokio::test(flavor = "multi_thread")]
async fn list_store_with_input_schema() {
    let client = require_client!();
    let page = client
        .store()
        .list(StoreListOptions {
            limit: Some(3),
            include_input_schema: Some(true),
            ..Default::default()
        })
        .await
        .expect("listing the store with input schemas should succeed");
    assert!(!page.items.is_empty(), "the Store lists Actors");
    assert!(page.items.len() <= 3);
}
