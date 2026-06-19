//! Lazily iterate Actors in the Apify Store using the convenience iteration method.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example iterate_store`

use apify_client::{ApifyClient, StoreListOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Iterate the store, fetching pages on demand. Stop after the first 10 actors.
    let mut iter = client.store().iterate(StoreListOptions {
        limit: Some(5),
        ..Default::default()
    });

    let mut count = 0;
    while let Some(actor) = iter.next().await? {
        println!("{}: {:?}", actor.id, actor.title.or(actor.name));
        count += 1;
        if count >= 10 {
            break;
        }
    }
    println!("Iterated {count} store actors");

    Ok(())
}
