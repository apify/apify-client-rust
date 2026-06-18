//! Integration tests for running an Actor end-to-end.
//!
//! These run the public `apify/hello-world` Store Actor, which is fast and free, and
//! exercise the run lifecycle, log access and reading the run's default dataset.

mod common;

/// Simple GET: listing the account's runs.
#[tokio::test]
async fn list_runs() {
    let client = require_client!();
    let page = client
        .runs()
        .list(Default::default(), None)
        .await
        .expect("listing runs should succeed");
    assert!(page.total >= 0);
}

/// Complex flow: call the hello-world Actor, wait for it to finish, fetch its log and
/// read its default dataset.
#[tokio::test]
async fn run_actor_and_read_outputs() {
    let client = require_client!();

    // Start + wait for the public hello-world Actor.
    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await
        .expect("call hello-world actor");

    assert_eq!(
        run.status.as_deref(),
        Some("SUCCEEDED"),
        "hello-world run should succeed"
    );

    // Fetch the run again via the run client.
    let fetched = client
        .run(&run.id)
        .get()
        .await
        .expect("get run")
        .expect("run should exist");
    assert_eq!(fetched.id, run.id);

    // Fetch the run log.
    let log = client.run(&run.id).log().get().await.expect("get run log");
    assert!(log.is_some(), "finished run should have a log");

    // Read the run's default dataset. (The `hello-world` Actor writes its result to the
    // key-value store rather than the dataset, so the dataset may be empty — we only
    // assert that reading it succeeds.)
    let _items = client
        .run(&run.id)
        .dataset()
        .list_items::<serde_json::Value>(Default::default())
        .await
        .expect("read run dataset");

    // Read the run's default key-value store OUTPUT record, where hello-world stores its
    // result. This proves end-to-end retrieval of an Actor's output.
    let output = client
        .run(&run.id)
        .key_value_store()
        .get_record("OUTPUT")
        .await
        .expect("read OUTPUT record");
    assert!(
        output.is_some(),
        "hello-world should produce an OUTPUT record"
    );
}

/// Convenience: access the Actor's last run.
#[tokio::test]
async fn last_run_access() {
    let client = require_client!();
    // Ensure there is at least one run by calling the actor.
    client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await
        .expect("call hello-world actor");

    let last = client
        .actor("apify/hello-world")
        .last_run(Some("SUCCEEDED"))
        .get()
        .await
        .expect("get last run");
    assert!(last.is_some(), "there should be a last succeeded run");
}
