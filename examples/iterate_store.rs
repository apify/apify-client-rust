//! Lazily iterate Actors in the Apify Store using the convenience iteration method.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example iterate_store`

use apify_client::{ApifyClient, StoreListOptions};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Iterate the store, fetching pages of 5 on demand (`limit` is the per-page size, not a
    // total cap). The loop below stops after the first 10 actors regardless of page size.
    let stream = client.store().iterate(StoreListOptions {
        limit: Some(5),
        ..Default::default()
    });
    futures_util::pin_mut!(stream);

    let mut count = 0;
    while let Some(actor) = stream.next().await {
        let actor = actor?;
        println!("{}: {:?}", actor.id, actor.title.or(actor.name));
        count += 1;
        if count >= 10 {
            break;
        }
    }
    println!("Iterated {count} store actors");

    Ok(())
}
