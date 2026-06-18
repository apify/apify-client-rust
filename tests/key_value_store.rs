//! Integration tests for the key-value store resource.

mod common;

use serde_json::json;

/// Simple GET: listing key-value stores should succeed.
#[tokio::test]
async fn list_key_value_stores() {
    let client = require_client!();
    let page = client
        .key_value_stores()
        .list(Default::default())
        .await
        .expect("listing key-value stores should succeed");
    assert!(page.total >= 0);
}

/// Complex flow: create -> get -> set record -> read record -> list keys -> update -> delete.
#[tokio::test]
async fn key_value_store_crud_flow() {
    let client = require_client!();
    let name = common::unique_name("kvs");

    let store = client
        .key_value_stores()
        .get_or_create(Some(&name))
        .await
        .expect("create store");
    assert_eq!(store.name.as_deref(), Some(name.as_str()));

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
#[tokio::test]
async fn record_public_url_is_fetchable() {
    let client = require_client!();
    let name = common::unique_name("kvs-sig");
    let store = client
        .key_value_stores()
        .get_or_create(Some(&name))
        .await
        .expect("create store");
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
