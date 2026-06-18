//! Create a new Actor, build it, run it, wait for it to finish, then fetch and print the
//! finished run's log.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example create_build_run_actor`

use apify_client::ApifyClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    // Create a minimal Actor with an inline source-files version.
    let suffix = uuid_like();
    let name = format!("rust-example-{suffix}");
    let definition = json!({
        "name": name,
        "isPublic": false,
        "versions": [{
            "versionNumber": "0.0",
            "sourceType": "SOURCE_FILES",
            "buildTag": "latest",
            "sourceFiles": [
                { "name": "Dockerfile", "format": "TEXT",
                  "content": "FROM apify/actor-node:20\nCOPY . ./\nCMD node main.js" },
                { "name": "main.js", "format": "TEXT",
                  "content": "console.log('Hello from the Rust client example!');" }
            ]
        }]
    });

    let actor = client.actors().create(&definition).await?;
    let actor_client = client.actor(&actor.id);
    println!("Created actor {}", actor.id);

    // Build the version and wait for the build to finish.
    let build = actor_client.build("0.0", Default::default()).await?;
    let finished_build = client.build(&build.id).wait_for_finish(Some(300)).await?;
    println!(
        "Build {} status: {:?}",
        finished_build.id, finished_build.status
    );

    // Run the Actor and wait for it to finish.
    let run = actor_client
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await?;
    println!("Run {} status: {:?}", run.id, run.status);

    // Fetch and print the finished run's log.
    if let Some(log) = client.run(&run.id).log().get().await? {
        println!("--- run log ---\n{log}");
    }

    // Clean up.
    actor_client.delete().await?;
    Ok(())
}

/// Tiny unique suffix without pulling in extra deps for the example.
fn uuid_like() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{nanos:x}")
}
