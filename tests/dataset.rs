//! Integration tests for the dataset resource.

mod common;

use serde_json::json;

/// Simple GET: listing datasets should succeed and return a paginated structure.
#[tokio::test]
async fn list_datasets() {
    let client = require_client!();
    let page = client
        .datasets()
        .list(Default::default())
        .await
        .expect("listing datasets should succeed");
    // `total` is non-negative; `items` length never exceeds `limit` when set.
    assert!(page.total >= 0);
}

/// Simple GET: fetch a single dataset by ID.
#[tokio::test]
async fn get_dataset() {
    let client = require_client!();
    let name = common::unique_name("dataset-get");
    let dataset = client
        .datasets()
        .get_or_create(Some(&name))
        .await
        .expect("create dataset");

    // Panic-safe cleanup: deletes the dataset even if an assertion below fails.
    let cleanup_client = client.clone();
    let id = dataset.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.dataset(&id).delete().await;
    });

    let fetched = client
        .dataset(&dataset.id)
        .get()
        .await
        .expect("get dataset by id");
    let fetched = fetched.expect("dataset should exist");
    assert_eq!(fetched.id, dataset.id);
}

/// Complex flow: create -> get -> push items -> read items -> update -> delete.
#[tokio::test]
async fn dataset_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("dataset");

    // Create (get-or-create with a fresh unique name).
    let dataset = client
        .datasets()
        .get_or_create(Some(&name))
        .await
        .expect("create dataset");
    assert_eq!(dataset.name.as_deref(), Some(name.as_str()));

    // Panic-safe cleanup so a mid-flow failure does not leak the dataset.
    let cleanup_client = client.clone();
    let cleanup_id = dataset.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.dataset(&cleanup_id).delete().await;
    });

    let dataset_client = client.dataset(&dataset.id);

    // Get.
    let fetched = dataset_client.get().await.expect("get dataset");
    assert!(fetched.is_some(), "created dataset should be retrievable");

    // Push items.
    dataset_client
        .push_items(&json!([{ "n": 1 }, { "n": 2 }, { "n": 3 }]))
        .await
        .expect("push items");

    // Read items back.
    let items = dataset_client
        .list_items::<serde_json::Value>(Default::default())
        .await
        .expect("list items");
    assert_eq!(items.items.len(), 3, "should read back the 3 pushed items");

    // Download items as CSV with export options (exercises the export/format path).
    let csv = dataset_client
        .download_items(
            apify_client::DownloadItemsFormat::Csv,
            apify_client::DatasetDownloadOptions {
                bom: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("download items as csv");
    assert!(!csv.is_empty(), "CSV export should not be empty");

    // Build a public items URL (signed if the dataset is private).
    let public_url = dataset_client
        .create_items_public_url(Default::default(), None)
        .await
        .expect("create items public url");
    assert!(public_url.contains("/datasets/"));
    assert!(public_url.contains("/items"));

    // Read statistics (simple GET).
    let _stats = dataset_client
        .get_statistics()
        .await
        .expect("get statistics");

    // Update (rename).
    let renamed = common::unique_name("dataset-renamed");
    let updated = dataset_client
        .update(&json!({ "name": renamed }))
        .await
        .expect("update dataset");
    assert_eq!(updated.name.as_deref(), Some(renamed.as_str()));

    // Delete.
    dataset_client.delete().await.expect("delete dataset");
    let after_delete = dataset_client.get().await.expect("get after delete");
    assert!(
        after_delete.is_none(),
        "dataset should be gone after delete"
    );
}
