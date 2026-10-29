//! Integration tests for running an Actor end-to-end.
//!
//! These run the public `apify/hello-world` Store Actor, which is fast and free, and
//! exercise the run lifecycle, log access and reading the run's default dataset.

mod common;

/// Simple GET: listing the account's runs.
#[tokio::test(flavor = "multi_thread")]
async fn list_runs() {
    let client = require_client!();
    let page = client
        .runs()
        .list(Default::default(), Default::default())
        .await
        .expect("listing runs should succeed");
    assert!(page.total >= 0);
}

/// Complex flow: call the hello-world Actor, wait for it to finish, fetch its log and
/// read its default dataset.
#[tokio::test(flavor = "multi_thread")]
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

    // Fetch the same log as raw bytes; it must decode to the same text `get()` returned.
    let log_bytes = client
        .run(&run.id)
        .log()
        .get_bytes()
        .await
        .expect("get run log as bytes")
        .expect("finished run should have a log");
    assert_eq!(
        String::from_utf8_lossy(&log_bytes),
        log.expect("log should be present"),
        "get_bytes() must decode to the same text as get()"
    );

    // Fetch the raw log via the `raw` query parameter (LogOptions). The endpoint must accept
    // it and still return the log content.
    let raw_log = client
        .run(&run.id)
        .log()
        .get_with_options(apify_client::LogOptions {
            raw: Some(true),
            ..Default::default()
        })
        .await
        .expect("get raw run log");
    assert!(raw_log.is_some(), "finished run should have a raw log");

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

/// Iteration: the run collection iterator yields a run we just started across pages.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_runs() {
    let client = require_client!();
    // Ensure at least one run exists on the account by calling the public hello-world Actor.
    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await
        .expect("call hello-world actor");

    // Newest-first with a small page size so the just-finished run is near the front. `limit`
    // is a total-item cap, so it is left unset here; page size is set via `with_chunk_size`.
    let target = run.id.clone();
    assert!(
        common::iter_contains_eventually(
            || {
                client
                    .runs()
                    .iterate(
                        apify_client::ListOptions {
                            desc: Some(true),
                            ..Default::default()
                        },
                        Default::default(),
                    )
                    .with_chunk_size(5)
            },
            move |r| r.id == target,
        )
        .await,
        "run iteration should yield the started run"
    );
}

/// Convenience: access the Actor's last run.
#[tokio::test(flavor = "multi_thread")]
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

/// `ActorClient::call_raw` sends a raw-bytes input (here, pre-serialized JSON bytes) instead of a
/// JSON-serializable value, and the run still completes normally — exercising the raw-input path
/// end to end against the real API.
#[tokio::test(flavor = "multi_thread")]
async fn call_raw_sends_raw_bytes_input() {
    let client = require_client!();

    let run = client
        .actor("apify/hello-world")
        .call_raw(
            b"{}",
            apify_client::ActorStartOptions {
                content_type: Some("application/json".to_string()),
                ..Default::default()
            },
            Some(120),
        )
        .await
        .expect("call_raw hello-world actor");

    assert_eq!(
        run.status.as_deref(),
        Some("SUCCEEDED"),
        "hello-world run started with a raw-bytes input should still succeed"
    );
}

/// A failed `log().stream()` carries the API's structured error (status/type/message), not a
/// generic transport error: this endpoint bypasses the buffered backend path to keep the
/// connection open, so it has to parse the error body itself on a non-2xx response.
#[tokio::test(flavor = "multi_thread")]
async fn log_stream_of_missing_run_carries_structured_api_error() {
    let client = require_client!();

    // The stream type doesn't implement `Debug`, so discard it on success via `map` before
    // `expect_err` (which requires the `Ok` variant to be `Debug` for its panic message).
    let err = client
        .run("nonexistent-run-id-xyz")
        .log()
        .stream()
        .await
        .map(|_| ())
        .expect_err("streaming a nonexistent run's log should fail");

    let api_error = err
        .as_api_error()
        .expect("error should be a structured ApiError, not a generic transport error");
    assert_eq!(api_error.status_code, 404);
    assert!(api_error.is_not_found());
}

/// A chained client cannot tell a missing run from a missing storage or log, so a `404` through it
/// is an error, while a client that names its resource by id resolves it to `None`.
#[tokio::test(flavor = "multi_thread")]
async fn not_found_through_a_chained_client_is_an_error() {
    let client = require_client!();
    let run = client.run("nonexistent-run-id-xyz");

    let err = run
        .dataset()
        .get()
        .await
        .expect_err("a chained client returns the 404");
    assert!(err.as_api_error().expect("an API error").is_not_found());
    let err = run
        .log()
        .get()
        .await
        .expect_err("a chained log client returns the 404");
    assert!(err.as_api_error().expect("an API error").is_not_found());

    assert!(client
        .log("nonexistent-run-id-xyz")
        .get()
        .await
        .expect("a log addressed by id resolves a 404")
        .is_none());
    assert!(client
        .log("nonexistent-run-id-xyz")
        .stream()
        .await
        .expect("a log stream addressed by id resolves a 404")
        .is_none());
    assert!(client
        .dataset("nonexistent-dataset-id-xyz")
        .get()
        .await
        .expect("a dataset addressed by id resolves a 404")
        .is_none());
}

/// An id that cannot be placed in a URL path fails without a request.
#[tokio::test(flavor = "multi_thread")]
async fn an_invalid_id_is_rejected() {
    let client = require_client!();
    let err = client
        .run("..")
        .get()
        .await
        .expect_err("a dot segment is not a run id");
    assert!(
        matches!(err, apify_client::ApifyClientError::InvalidArgument(_)),
        "{err:?}"
    );
}

/// The `deployment` filter is accepted by the run listing, together with the non-terminal status
/// it requires.
#[tokio::test(flavor = "multi_thread")]
async fn list_runs_filtered_by_deployment() {
    let client = require_client!();
    let page = client
        .runs()
        .list(
            Default::default(),
            apify_client::RunListOptions {
                status: vec!["RUNNING".to_string()],
                deployment: vec!["SINGLE_TENANT".to_string()],
                ..Default::default()
            },
        )
        .await
        .expect("listing runs by deployment should succeed");
    assert!(page.total >= 0);
}

/// Iterating the default dataset of a run reads the items pushed to it, including those past the
/// dataset's `itemCount`, which lags a few seconds behind the pushed items.
#[tokio::test(flavor = "multi_thread")]
async fn iterate_run_dataset_items_reads_the_items_of_a_finished_run() {
    let client = require_client!();
    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await
        .expect("call hello-world actor");
    let dataset_id = run
        .default_dataset_id
        .clone()
        .expect("a run has a default dataset");
    client
        .dataset(&dataset_id)
        .push_items(&serde_json::json!([{ "n": 1 }, { "n": 2 }, { "n": 3 }]))
        .await
        .expect("push items to the run's dataset");

    let items = client
        .run(&run.id)
        .iterate_dataset_items::<serde_json::Value>(Default::default())
        .with_chunk_size(2)
        .collect_all()
        .await
        .expect("iterate the run's dataset items");

    // The Actor may have written items of its own before the three that were pushed.
    assert!(items.len() >= 3, "{items:?}");
    let pushed = &items[items.len() - 3..];
    assert_eq!(pushed[0]["n"], 1);
    assert_eq!(pushed[2]["n"], 3);
}
