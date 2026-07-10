//! Discover an existing Actor in the Apify Store, run it, wait for it to finish, and read
//! its default dataset.
//!
//! This example uses the Store API (`client.store()`) to find the actor first, so it really
//! exercises "run an Actor discovered in the Store"; it only falls back to the well-known
//! `apify/hello-world` identifier if the search does not surface it.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example run_store_actor`

use apify_client::{ApifyClient, StoreListOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Discover the public `apify/hello-world` Actor through the Apify Store API. It is free,
    // fast, and runs without input, which keeps this example reliable.
    let store_page = client
        .store()
        .list(StoreListOptions {
            search: Some("hello world".to_string()),
            limit: Some(25),
            ..Default::default()
        })
        .await?;

    // Prefer the ID discovered via search, but fall back to the well-known `apify/hello-world`
    // identifier if Store ranking pushes it out of the first page — that keeps the example
    // (and its CI smoke test) from failing when search results shift.
    let actor_id = store_page
        .items
        .into_iter()
        .find(|a| {
            a.username.as_deref() == Some("apify") && a.name.as_deref() == Some("hello-world")
        })
        .map(|a| a.id)
        .unwrap_or_else(|| "apify~hello-world".to_string());
    println!("Using Store actor {actor_id}");

    // Run the discovered Actor and wait up to 2 minutes for it to finish.
    let run = client
        .actor(&actor_id)
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await?;
    println!("Run {} finished with status {:?}", run.id, run.status);

    // Read items from the run's default dataset.
    if let Some(dataset_id) = &run.default_dataset_id {
        let items = client
            .dataset(dataset_id)
            .list_items::<serde_json::Value>(Default::default())
            .await?;
        println!("Default dataset has {} item(s)", items.items.len());
    }

    Ok(())
}
