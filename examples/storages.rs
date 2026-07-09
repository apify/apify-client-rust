//! Create each storage type, write to it and read back from it.
//!
//! Demonstrates datasets, key-value stores and request queues.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example storages`

use apify_client::models::RequestQueueRequest;
use apify_client::ApifyClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // ---- Dataset: create, push, read ----
    let dataset = client.datasets().get_or_create(None).await?;
    let dataset_client = client.dataset(&dataset.id);
    dataset_client
        .push_items(&json!([{ "hello": "dataset" }]))
        .await?;
    let items = dataset_client
        .list_items::<serde_json::Value>(Default::default())
        .await?;
    println!("Dataset {} has {} item(s)", dataset.id, items.items.len());
    dataset_client.delete().await?;

    // ---- Key-value store: create, set, get ----
    let store = client.key_value_stores().get_or_create(None).await?;
    let store_client = client.key_value_store(&store.id);
    store_client
        .set_record_json("OUTPUT", &json!({ "hello": "kvs" }))
        .await?;
    // A key just written to a key-value store is immediately readable — within-store
    // read-after-write is strongly consistent — so this lookup is expected to return the record.
    let record = store_client
        .get_record("OUTPUT")
        .await?
        .expect("OUTPUT was just written and is readable within the same store");
    println!("KVS {} OUTPUT = {} bytes", store.id, record.value.len());
    store_client.delete().await?;

    // ---- Request queue: create, add, read ----
    let queue = client.request_queues().get_or_create(None).await?;
    let queue_client = client.request_queue(&queue.id);
    let request = RequestQueueRequest {
        id: None,
        url: "https://example.com/".to_string(),
        unique_key: Some("example".to_string()),
        method: Some("GET".to_string()),
        user_data: None,
        extra: Default::default(),
    };
    let added = queue_client
        .add_request(&request, Default::default())
        .await?;
    println!("RQ {} added request {}", queue.id, added.request_id);
    let head = queue_client.list_head(Some(10)).await?;
    println!("RQ head has {} request(s)", head.items.len());
    queue_client.delete().await?;

    Ok(())
}
