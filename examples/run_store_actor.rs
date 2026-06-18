//! Run an existing Store Actor, wait for it to finish, and read its default dataset.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example run_store_actor`

use apify_client::ApifyClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Start the public `apify/hello-world` Actor and wait up to 2 minutes for it to finish.
    let run = client
        .actor("apify/hello-world")
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
