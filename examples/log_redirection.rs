//! Run an Actor with its log redirected (streamed) to stdout in real time.
//!
//! This demonstrates the log streaming feature: while the run is in progress, log chunks
//! are streamed and printed as they arrive.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example log_redirection`

use apify_client::ApifyClient;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Start the Actor (without waiting), so we can stream its log while it runs.
    let run = client
        .actor("apify/hello-world")
        .start::<serde_json::Value>(None, Default::default())
        .await?;
    println!("Started run {} — redirecting its log:", run.id);

    // Stream the run's log and print each chunk as it arrives.
    let mut stream = client.run(&run.id).log().stream().await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        print!("{}", String::from_utf8_lossy(&chunk));
    }
    println!("\n--- log stream ended (run finished) ---");

    Ok(())
}
