//! Run another Actor with log redirection turned on.
//!
//! "Log redirection" is the convenience of taking a *separate* Actor run's log and piping it,
//! live, into the calling program's own output as it is produced. This mirrors the JS reference
//! client's `StreamedLog`, which prefixes every message with `"{Actor name} runId:{run id} -> "`
//! so the redirected output is attributable to its source Actor.
//!
//! The first part lets `ActorClient::call` do it, which redirects the log to the standard output
//! by default. The second part starts the run itself and redirects its log to a custom sink.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example log_redirection`

use apify_client::{ActorCallOptions, ApifyClient, LogRedirect, StreamedLogOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // The (separate) Actor whose log we will redirect into this program's output.
    let source_actor = "apify/hello-world";

    // `call` redirects the log of the run to the standard output while it waits.
    let run = client
        .actor(source_actor)
        .call::<serde_json::Value>(None, ActorCallOptions::default())
        .await?;
    println!("--- run {} finished ---", run.id);

    // Start the run without waiting, and redirect its log to a sink of our own.
    let run = client
        .actor(source_actor)
        .start::<serde_json::Value>(None, Default::default())
        .await?;
    let run_client = client.run(&run.id);
    let mut streamed_log = run_client
        .get_streamed_log(StreamedLogOptions {
            to_log: LogRedirect::to_sink(move |message: &str| {
                println!("[{source_actor}] {message}")
            }),
            from_start: true,
        })
        .await?
        .ok_or("the run does not exist")?;
    streamed_log.start()?;
    run_client.wait_for_finish(Some(120)).await?;
    streamed_log.stop().await?;
    println!("--- log redirection ended (run finished) ---");

    Ok(())
}
