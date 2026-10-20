//! Integration tests for the dataset resource.

mod common;

use futures_util::StreamExt;
use serde_json::json;

/// Simple GET: listing datasets should succeed and return a paginated structure.
#[tokio::test(flavor = "multi_thread")]
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
#[tokio::test(flavor = "multi_thread")]
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

/// Iteration: the dataset collection iterator yields a just-created dataset across pages.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_datasets() {
    let client = require_client!();
    let name = common::unique_name("dataset-iter");
    let dataset = client
        .datasets()
        .get_or_create(Some(&name))
        .await
        .expect("create dataset");

    let cleanup_client = client.clone();
    let id = dataset.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.dataset(&id).delete().await;
    });

    // Newest-first with a small page size so the iterator must fetch at least one page.
    let target = dataset.id.clone();
    assert!(
        common::iter_contains_eventually(
            || {
                client
                    .datasets()
                    .iterate(apify_client::StorageListOptions {
                        desc: Some(true),
                        ..Default::default()
                    })
                    .with_chunk_size(5)
            },
            move |d| d.id == target,
        )
        .await,
        "dataset iteration should yield the created dataset"
    );
}

/// Iteration: dataset item iterator yields every pushed item exactly once across multiple pages.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_dataset_items() {
    let client = require_client!();
    let name = common::unique_name("dataset-items-iter");
    let dataset = client
        .datasets()
        .get_or_create(Some(&name))
        .await
        .expect("create dataset");

    let cleanup_client = client.clone();
    let id = dataset.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.dataset(&id).delete().await;
    });

    let dataset_client = client.dataset(&dataset.id);
    // Push 5 items so a page size of 2 forces three pages.
    dataset_client
        .push_items(&json!([{ "n": 0 }, { "n": 1 }, { "n": 2 }, { "n": 3 }, { "n": 4 }]))
        .await
        .expect("push items");

    // A page size of 2 forces three pages; `limit` is a total-item cap (left unset here) rather
    // than the page size, so every pushed item must still be yielded across pages.
    let mut iter = dataset_client
        .iterate_items::<serde_json::Value>(apify_client::DatasetListItemsOptions::default())
        .with_chunk_size(2);
    let mut seen = std::collections::HashSet::new();
    while let Some(item) = iter.next().await.expect("iterate items") {
        let n = item["n"].as_i64().expect("item has n");
        assert!(seen.insert(n), "item {n} yielded more than once");
    }
    assert_eq!(
        seen,
        (0..5).collect::<std::collections::HashSet<_>>(),
        "item iteration must yield every pushed item exactly once across pages"
    );
}

/// Complex flow: create -> get -> push items -> read items -> update -> delete.
#[tokio::test(flavor = "multi_thread")]
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

    // The `signature` query param must be settable on list_items (spec + JS reference both
    // expose it for fetching items from a private dataset with a pre-shared signature). On this
    // owned dataset the signature is unnecessary, but the request must still succeed.
    let signed = dataset_client
        .list_items::<serde_json::Value>(apify_client::DatasetListItemsOptions {
            signature: Some("test-signature".to_string()),
            ..Default::default()
        })
        .await
        .expect("list items with signature option");
    assert_eq!(signed.items.len(), 3);

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

/// `download_items_stream` must yield the same bytes as the buffered `download_items`, just
/// delivered incrementally instead of all at once.
#[tokio::test(flavor = "multi_thread")]
async fn download_items_stream_matches_buffered_download() {
    let client = require_client!();
    let name = common::unique_name("dataset-stream");
    let dataset = client
        .datasets()
        .get_or_create(Some(&name))
        .await
        .expect("create dataset");

    let cleanup_client = client.clone();
    let id = dataset.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.dataset(&id).delete().await;
    });

    let dataset_client = client.dataset(&dataset.id);
    dataset_client
        .push_items(&json!([{ "n": 1 }, { "n": 2 }, { "n": 3 }]))
        .await
        .expect("push items");

    let buffered = dataset_client
        .download_items(apify_client::DownloadItemsFormat::Json, Default::default())
        .await
        .expect("buffered download");

    let mut stream = dataset_client
        .download_items_stream(apify_client::DownloadItemsFormat::Json, Default::default())
        .await
        .expect("stream download");
    let mut streamed = Vec::new();
    while let Some(chunk) = stream.next().await {
        streamed.extend(chunk.expect("stream chunk"));
    }

    assert_eq!(
        streamed, buffered,
        "streamed export must match the buffered export byte-for-byte"
    );
}

/// A failed `download_items_stream` (nonexistent dataset) carries the API's structured error
/// (status/type/message), not a generic transport error — like `log().stream()` of a
/// nonexistent run, this bypasses the buffered backend path to keep the connection open, so it
/// has to parse the error body itself on a non-2xx response.
#[tokio::test(flavor = "multi_thread")]
async fn download_items_stream_of_missing_dataset_carries_structured_api_error() {
    let client = require_client!();

    let err = client
        .dataset("nonexistent-dataset-id-xyz")
        .download_items_stream(apify_client::DownloadItemsFormat::Json, Default::default())
        .await
        .map(|_| ())
        .expect_err("streaming a nonexistent dataset's items should fail");

    let api_error = err
        .as_api_error()
        .expect("error should be a structured ApiError, not a generic transport error");
    assert_eq!(api_error.status_code, 404);
    assert!(api_error.is_not_found());
}
