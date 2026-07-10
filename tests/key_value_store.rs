//! Integration tests for the key-value store resource.

mod common;

use serde_json::json;

/// Simple GET: listing key-value stores should succeed.
#[tokio::test(flavor = "multi_thread")]
async fn list_key_value_stores() {
    let client = require_client!();
    let page = client
        .key_value_stores()
        .list(Default::default())
        .await
        .expect("listing key-value stores should succeed");
    assert!(page.total >= 0);
}

/// Simple GET: fetch a single key-value store by ID.
#[tokio::test(flavor = "multi_thread")]
async fn get_key_value_store() {
    let client = require_client!();
    let name = common::unique_name("kvs-get");
    let store = client
        .key_value_stores()
        .get_or_create(Some(&name))
        .await
        .expect("create store");

    let cleanup_client = client.clone();
    let id = store.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.key_value_store(&id).delete().await;
    });

    let fetched = client
        .key_value_store(&store.id)
        .get()
        .await
        .expect("get store by id")
        .expect("store should exist");
    assert_eq!(fetched.id, store.id);
}

/// Iteration: the key-value store collection iterator yields a just-created store across pages.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_key_value_stores() {
    let client = require_client!();
    let name = common::unique_name("kvs-iter");
    let store = client
        .key_value_stores()
        .get_or_create(Some(&name))
        .await
        .expect("create store");

    let cleanup_client = client.clone();
    let id = store.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.key_value_store(&id).delete().await;
    });

    let iter = client
        .key_value_stores()
        .iterate(apify_client::StorageListOptions {
            desc: Some(true),
            ..Default::default()
        })
        .with_chunk_size(5);
    let target = store.id.clone();
    assert!(
        common::iter_contains(iter, move |s| s.id == target).await,
        "key-value store iteration should yield the created store"
    );
}

/// Record keys containing characters that are valid for the API (`!`, `'`, `(`, `)`) but
/// reserved in a URL path must round-trip correctly, proving the path segment is
/// percent-encoded rather than interpolated raw.
///
/// The Apify API restricts record keys to `a-zA-Z0-9!-_.'()`; several of those (`!`, `'`,
/// `(`, `)`) are not RFC 3986 unreserved characters, so they must be percent-encoded in the
/// URL path. A raw `format!(".../{key}")` would send them unescaped.
#[tokio::test(flavor = "multi_thread")]
async fn record_key_with_special_chars_round_trips() {
    let client = require_client!();
    let name = common::unique_name("kvs-key");
    let store = client
        .key_value_stores()
        .get_or_create(Some(&name))
        .await
        .expect("create store");

    let cleanup_client = client.clone();
    let id = store.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.key_value_store(&id).delete().await;
    });

    let store_client = client.key_value_store(&store.id);
    // Valid API key with URL-reserved characters that must be percent-encoded in the path.
    let key = "my!key'(v1).json";
    store_client
        .set_record_json(key, &json!({ "ok": true }))
        .await
        .expect("set record with special-char key");

    assert!(
        store_client.record_exists(key).await.expect("exists"),
        "record with special-char key should exist (key must be percent-encoded)"
    );
    let record = store_client
        .get_record(key)
        .await
        .expect("get record")
        .expect("record should exist");
    let value: serde_json::Value = record.json().expect("parse json");
    assert_eq!(value["ok"], true);

    store_client
        .delete_record(key)
        .await
        .expect("delete record with special-char key");
    assert!(!store_client.record_exists(key).await.expect("exists after"));
}

/// Simple GET: download all records as a ZIP archive.
///
/// Stores a record, then downloads the whole store via `get_records` and asserts the response
/// is a non-empty ZIP archive (the spec response is `application/zip`; ZIP files start with the
/// `PK\x03\x04` local-file-header magic).
#[tokio::test(flavor = "multi_thread")]
async fn get_records_returns_zip_archive() {
    let client = require_client!();
    let name = common::unique_name("kvs-zip");
    let store = client
        .key_value_stores()
        .get_or_create(Some(&name))
        .await
        .expect("create store");

    let cleanup_client = client.clone();
    let id = store.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.key_value_store(&id).delete().await;
    });

    let store_client = client.key_value_store(&store.id);
    store_client
        .set_record_json("OUTPUT", &json!({ "zip": true }))
        .await
        .expect("set record");

    let archive = store_client
        .get_records(Default::default())
        .await
        .expect("download records as zip");
    assert!(!archive.is_empty(), "ZIP archive should not be empty");
    assert_eq!(
        &archive[..4],
        b"PK\x03\x04",
        "response should be a ZIP archive (PK magic bytes)"
    );

    // Happy-path cleanup in the body (the guard above remains a panic-safety net).
    store_client.delete().await.expect("delete store");
}

/// Complex flow: create -> get -> set record -> read record -> list keys -> update -> delete.
#[tokio::test(flavor = "multi_thread")]
async fn key_value_store_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("kvs");

    let store = client
        .key_value_stores()
        .get_or_create(Some(&name))
        .await
        .expect("create store");
    assert_eq!(store.name.as_deref(), Some(name.as_str()));

    let cleanup_client = client.clone();
    let cleanup_id = store.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.key_value_store(&cleanup_id).delete().await;
    });

    let store_client = client.key_value_store(&store.id);

    // Get.
    assert!(store_client.get().await.expect("get store").is_some());

    // Set a JSON record.
    store_client
        .set_record_json("OUTPUT", &json!({ "hello": "world" }))
        .await
        .expect("set record");

    // record_exists should now be true.
    assert!(store_client.record_exists("OUTPUT").await.expect("exists"));

    // Read record back and verify content.
    let record = store_client
        .get_record("OUTPUT")
        .await
        .expect("get record")
        .expect("record should exist");
    let value: serde_json::Value = record.json().expect("parse record json");
    assert_eq!(value["hello"], "world");

    // Read again via the explicit-options API (attachment override exercised).
    let record2 = store_client
        .get_record_with_options(
            "OUTPUT",
            apify_client::GetRecordOptions {
                attachment: Some(false),
                ..Default::default()
            },
        )
        .await
        .expect("get record with options")
        .expect("record should exist");
    let value2: serde_json::Value = record2.json().expect("parse record json");
    assert_eq!(value2["hello"], "world");

    // List keys.
    let keys = store_client
        .list_keys(Default::default())
        .await
        .expect("list keys");
    assert!(keys.items.iter().any(|k| k.key == "OUTPUT"));

    // Update (rename).
    let renamed = common::unique_name("kvs-renamed");
    let updated = store_client
        .update(&json!({ "name": renamed }))
        .await
        .expect("update store");
    assert_eq!(updated.name.as_deref(), Some(renamed.as_str()));

    // Delete record, then delete store.
    store_client
        .delete_record("OUTPUT")
        .await
        .expect("delete record");
    assert!(!store_client
        .record_exists("OUTPUT")
        .await
        .expect("exists after delete"));

    store_client.delete().await.expect("delete store");
    assert!(store_client
        .get()
        .await
        .expect("get after delete")
        .is_none());
}

/// Builds a record public URL and confirms it is well-formed and fetchable without auth.
///
/// The URL points at the public records endpoint; when the store exposes a URL-signing
/// secret key the URL additionally carries an HMAC `signature`. We fetch it with a bare HTTP
/// client (no Authorization header) and require success.
#[tokio::test(flavor = "multi_thread")]
async fn record_public_url_is_fetchable() {
    let client = require_client!();
    let name = common::unique_name("kvs-sig");
    let store = client
        .key_value_stores()
        .get_or_create(Some(&name))
        .await
        .expect("create store");

    let cleanup_client = client.clone();
    let cleanup_id = store.id.clone();
    let _guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.key_value_store(&cleanup_id).delete().await;
    });

    let store_client = client.key_value_store(&store.id);
    store_client
        .set_record_json("OUTPUT", &json!({ "signed": true }))
        .await
        .expect("set record");

    let url = store_client
        .get_record_public_url("OUTPUT")
        .await
        .expect("record public url");
    assert!(url.contains("/key-value-stores/"));
    assert!(url.contains("/records/OUTPUT"));

    // Fetch the URL with a bare HTTP client (no Authorization header).
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .expect("fetch public url");
    assert!(
        resp.status().is_success(),
        "public record URL should be fetchable, got {} for {url}",
        resp.status()
    );

    store_client.delete().await.expect("delete store");
}
