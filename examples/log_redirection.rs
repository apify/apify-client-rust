//! Run another Actor with log redirection turned on.
//!
//! "Log redirection" is the convenience of taking a *separate* Actor run's log and piping it,
//! live, into the calling program's own output as it is produced. This mirrors the JS reference
//! client's `StreamedLog` / `LoggerActorRedirect`, which redirects a run's streamed log into
//! another `Log` sink and prefixes every line with `"{name} -> "` so the redirected output is
//! attributable to its source Actor.
//!
//! Here we start the `apify/hello-world` Actor (a different Actor from this program), then
//! redirect its run log into our stdout in real time, prefixing each line with the Actor's name.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example log_redirection`

use apify_client::ApifyClient;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // The (separate) Actor whose log we will redirect into this program's output.
    let source_actor = "apify/hello-world";

    // Start the Actor without waiting, so its log can be redirected while the run is in progress.
    let run = client
        .actor(source_actor)
        .start::<serde_json::Value>(None, Default::default())
        .await?;
    println!(
        "Started run {} of {source_actor} — redirecting its log:",
        run.id
    );

    // Redirect the run's log into our stdout, line by line, with a source prefix (the same
    // `"{name} -> "` convention the JS reference uses for redirected Actor logs). The log is
    // streamed as raw byte chunks, so we buffer until a newline to emit whole, prefixed lines.
    let prefix = format!("{source_actor} -> ");
    let mut stream = client.run(&run.id).log().stream().await?;
    let mut buf = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buf.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(newline) = buf.find('\n') {
            let line: String = buf.drain(..=newline).collect();
            print!("{prefix}{line}");
        }
    }
    // Flush any trailing partial line that was not newline-terminated.
    if !buf.is_empty() {
        println!("{prefix}{buf}");
    }
    println!("--- log redirection ended (run finished) ---");

    Ok(())
}
