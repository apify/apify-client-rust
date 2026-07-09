//! Start an Actor run, wait for it to finish, then fetch the Actor's last run and access
//! its storages (dataset, key-value store, request queue).
//!
//! Run with: `APIFY_TOKEN=... cargo run --example run_and_last_run_storages`

use apify_client::{ApifyClient, LastRunOptions, RunOrigin, RunStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Start a run and wait for it to finish.
    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await?;
    println!("Run {} finished: {:?}", run.id, run.status);

    // Fetch the Actor's last successful run. `last_run` is scoped to *your own* runs of the Actor
    // (the run we just made above), and the platform's run index is eventually consistent, so it
    // can briefly return `None` right after a run finishes. Handle the `None` case rather than
    // unwrapping; here we fall back to the run we already have.
    let last_run = match client
        .actor("apify/hello-world")
        .last_run(Some(RunStatus::Succeeded))
        .get()
        .await?
    {
        Some(last_run) => last_run,
        None => {
            println!("Last run not indexed yet; using the run we just made.");
            run
        }
    };
    println!("Last run id: {}", last_run.id);

    // `last_run_with_options` can additionally filter by `origin` (how the run was started).
    // Runs started via the API (like the one above) have origin `RunOrigin::Api`, so this narrows
    // to API-started runs; leave a field as `None` to omit it.
    let api_last_run = client
        .actor("apify/hello-world")
        .last_run_with_options(LastRunOptions {
            status: Some(RunStatus::Succeeded),
            origin: Some(RunOrigin::Api),
        })
        .get()
        .await?;
    println!("Last API-origin run indexed: {}", api_last_run.is_some());

    // Access the last run's storages via the run client.
    let run_client = client.run(&last_run.id);
    let dataset = run_client
        .dataset()
        .list_items::<serde_json::Value>(Default::default())
        .await?;
    println!("Last run default dataset items: {}", dataset.items.len());

    let keys = run_client
        .key_value_store()
        .list_keys(Default::default())
        .await?;
    println!("Last run default KVS keys: {}", keys.items.len());

    let head = run_client.request_queue().list_head(Some(5)).await?;
    println!("Last run default RQ head items: {}", head.items.len());

    Ok(())
}
