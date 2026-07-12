//! Lazily iterate Actors in the Apify Store using the convenience iteration method.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example iterate_store`

use apify_client::{ApifyClient, StoreListOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Iterate the store, fetching pages of 5 on demand. `with_chunk_size` sets the per-request
    // page size; the options' `limit` (left unset here) would instead cap the total number of
    // items yielded. The loop below stops after the first 10 actors regardless of page size.
    let mut iter = client
        .store()
        .iterate(StoreListOptions::default())
        .with_chunk_size(5);

    let mut count = 0;
    while let Some(actor) = iter.next().await? {
        let label = actor
            .title
            .or(actor.name)
            .unwrap_or_else(|| "(untitled)".to_string());
        println!("{}: {label}", actor.id);
        count += 1;
        if count >= 10 {
            break;
        }
    }
    println!("Iterated {count} store actors");

    Ok(())
}
