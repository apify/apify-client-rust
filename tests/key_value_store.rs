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

    let target = store.id.clone();
    assert!(
        common::iter_contains_eventually(
            || {
                client
                    .key_value_stores()
                    .iterate(apify_client::StorageListOptions {
                        desc: Some(true),
                        ..Default::default()
                    })
                    .with_chunk_size(5)
            },
            move |s| s.id == target,
        )
        .await,
        "key-value store iteration should yield the created store"
    );
}

/// Iteration: `iterate_keys` yields every key in the store.
///
/// Creates a store, writes several records, then drives the `KeyValueStoreKeysIterator` to
/// completion and asserts every written key is yielded exactly once. With only a handful of keys
/// this fits in the endpoint's single default page, so it validates the helper end to end but
/// does not cross a page boundary — `iterate_keys` has no per-page-size knob (its `limit` is a
/// total cap, matching the JS reference), so forcing >1 page would need >1000 keys. The
/// multi-page cursor-threading path (`exclusiveStartKey`/`nextExclusiveStartKey`) is covered
/// hermetically by `iterate_keys_walks_cursor_pages` in `tests/unit_http.rs`.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_keys_yields_all_keys() {
    let client = require_client!();
    let name = common::unique_name("kvs-keys-iter");
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
    let expected: Vec<String> = (0..5).map(|i| format!("iter-key-{i:02}")).collect();
    for key in &expected {
        store_client
            .set_record_json(key, &json!({ "n": key }))
            .await
            .expect("set record");
    }

    // Drive the iterator to completion and collect the yielded keys.
    let mut iter = store_client.iterate_keys(Default::default());
    let mut seen = Vec::new();
    while let Some(key) = iter.next().await.expect("key iteration should not error") {
        seen.push(key.key);
    }

    for key in &expected {
        assert!(
            seen.contains(key),
            "iterate_keys should yield every created key; missing {key}, saw {seen:?}"
        );
    }
    // No key should appear more than once across the paginated walk.
    let mut unique = seen.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        seen.len(),
        "iterate_keys must not yield duplicate keys, saw {seen:?}"
    );

    store_client.delete().await.expect("delete store");
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

/// `set_record_raw_with_content_encoding` uploads an already-compressed value without the
/// client compressing it a second time, and the value round-trips without corruption.
///
/// The payload is well above the client's own auto-compression size threshold, so this also
/// exercises that the `Content-Encoding` header (not just the small-body skip) is what the
/// client relies on to avoid double-compression; `set_record_with_content_encoding_is_not_compressed_again`
/// in `tests/unit_http.rs` pins that exact wire behaviour hermetically. What the API does with
/// the encoded upload server-side (store the compressed bytes as-is per
/// `KeyValueStoreClient::set_record_raw_with_content_encoding`'s docs, or transparently decode
/// it before storing) is outside the client's control, so this only asserts the value it reads
/// back decodes to the original payload either way, never that the bytes are corrupted or
/// double-compressed.
#[tokio::test(flavor = "multi_thread")]
async fn set_record_with_content_encoding_round_trips_without_corruption() {
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    let client = require_client!();
    let name = common::unique_name("kvs-enc");
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

    let payload = serde_json::to_vec(&json!({ "data": "x".repeat(4096) })).expect("serialize");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&payload).expect("gzip write");
    let gzipped = encoder.finish().expect("gzip finish");

    store_client
        .set_record_raw_with_content_encoding(
            "gz-record",
            gzipped.clone(),
            "application/json",
            "gzip",
        )
        .await
        .expect("set record with content encoding");

    let record = store_client
        .get_record("gz-record")
        .await
        .expect("get record")
        .expect("record should exist");

    // Accept either: the API served the exact gzip bytes we uploaded (decode them), or it
    // stored/served the decoded value directly (already equal to `payload`).
    let decoded = if record.value.starts_with(&[0x1f, 0x8b]) {
        let mut decoder = flate2::read::GzDecoder::new(&record.value[..]);
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut out).expect("gunzip response value");
        out
    } else {
        record.value
    };
    assert_eq!(
        decoded, payload,
        "the record must round-trip to the original payload without corruption"
    );
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
