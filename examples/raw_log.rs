//! Fetch an Actor run's log in its raw (unprocessed) form.
//!
//! By default the API adds a per-line timestamp prefix to log lines. Passing
//! `LogOptions { raw: Some(true) }` requests the unprocessed log instead — the same raw form
//! the JS reference's log redirection consumes. This example exercises both the buffered
//! (`get_with_options`) and the streaming (`stream_with_options`) raw-log paths end-to-end so
//! the `Test examples` CI step covers the new `raw` parameter against the live API.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example raw_log`

use apify_client::{ApifyClient, LogOptions};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Run a short public Actor and wait for it to finish so it has a complete log.
    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await?;
    println!("Run {} finished with status {:?}", run.id, run.status);

    // Buffered raw log: fetch the whole log without the server-added per-line timestamps.
    let raw_log = client
        .run(&run.id)
        .log()
        .get_with_options(LogOptions { raw: Some(true) })
        .await?;
    match raw_log {
        Some(log) => println!("Fetched raw log ({} bytes)", log.len()),
        None => println!("Run has no log"),
    }

    // Streaming raw log: stream the same unprocessed log chunk by chunk.
    let mut stream = client
        .run(&run.id)
        .get_streamed_log_with_options(LogOptions { raw: Some(true) })
        .await?;
    let mut streamed_bytes = 0usize;
    while let Some(chunk) = stream.next().await {
        streamed_bytes += chunk?.len();
    }
    println!("Streamed raw log ({streamed_bytes} bytes)");

    Ok(())
}
