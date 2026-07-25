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

    // Fetch the raw log via the `raw` query parameter (LogOptions). The endpoint must accept
    // it and still return the log content.
    let raw_log = client
        .run(&run.id)
        .log()
        .get_with_options(apify_client::LogOptions { raw: Some(true) })
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

/// Complex flow: the Run resource's remaining CRUD operations not exercised elsewhere —
/// `update()`, `ApifyClient::set_status_message` (which delegates to `update()` with a fixed
/// body shape), `delete()`, and confirming `get()` returns `None` afterward. Together with
/// `run_actor_and_read_outputs` (create/get/list via `runs().list()`), this covers all five
/// CRUD-flow operations `test_requirements.md` asks for on a resource that supports them.
#[tokio::test(flavor = "multi_thread")]
async fn run_update_set_status_message_and_delete() {
    let client = require_client!();

    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await
        .expect("call hello-world actor");
    assert_eq!(run.status.as_deref(), Some("SUCCEEDED"));
    let run_client = client.run(&run.id);

    // `update()` directly: set a non-terminal status message.
    let updated = run_client
        .update(&serde_json::json!({
            "statusMessage": "updated via RunClient::update",
            "isStatusMessageTerminal": false,
        }))
        .await
        .expect("update run");
    assert_eq!(
        updated.status_message.as_deref(),
        Some("updated via RunClient::update")
    );

    // `ApifyClient::set_status_message` reads `ACTOR_RUN_ID` from the environment and delegates
    // to `run(id).update(...)`. Env vars are process-global, so this test owns the variable for
    // its duration and restores it afterward.
    let prev_run_id = std::env::var("ACTOR_RUN_ID").ok();
    std::env::set_var("ACTOR_RUN_ID", &run.id);
    let via_set_status_message = client
        .set_status_message("updated via set_status_message", true)
        .await;
    match prev_run_id {
        Some(v) => std::env::set_var("ACTOR_RUN_ID", v),
        None => std::env::remove_var("ACTOR_RUN_ID"),
    }
    let via_set_status_message = via_set_status_message.expect("set_status_message");
    assert_eq!(
        via_set_status_message.status_message.as_deref(),
        Some("updated via set_status_message")
    );

    // `get()` reflects the latest update.
    let fetched = run_client
        .get()
        .await
        .expect("get run")
        .expect("run should still exist");
    assert_eq!(
        fetched.status_message.as_deref(),
        Some("updated via set_status_message")
    );

    // `delete()`, then `get()` must return `None`.
    run_client.delete().await.expect("delete run");
    let after_delete = run_client.get().await.expect("get run after delete");
    assert!(
        after_delete.is_none(),
        "run should not exist after delete(), got {after_delete:?}"
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

/// Simple GET: run-scoped storage metadata accessors (`.dataset().get()`/`.get_statistics()`,
/// `.key_value_store().get()`/`.list_keys()`, `.request_queue().get()`/`.list_head()`) all
/// succeed against a finished run's default storages.
#[tokio::test(flavor = "multi_thread")]
async fn run_scoped_storage_metadata_reads() {
    let client = require_client!();
    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await
        .expect("call hello-world actor");

    let run_client = client.run(&run.id);

    let dataset = run_client.dataset();
    assert!(
        dataset.get().await.expect("get run dataset").is_some(),
        "a run's default dataset should exist"
    );
    assert!(
        dataset
            .get_statistics()
            .await
            .expect("get run dataset statistics")
            .is_some(),
        "a run's default dataset should report statistics"
    );

    let kvs = run_client.key_value_store();
    assert!(
        kvs.get().await.expect("get run key-value store").is_some(),
        "a run's default key-value store should exist"
    );
    let keys = kvs
        .list_keys(Default::default())
        .await
        .expect("list run key-value store keys");
    assert!(
        keys.items.iter().any(|k| k.key == "OUTPUT"),
        "hello-world's default store should contain an OUTPUT key"
    );

    let rq = run_client.request_queue();
    assert!(
        rq.get().await.expect("get run request queue").is_some(),
        "a run's default request queue should exist"
    );
    let head = rq
        .list_head(Some(10))
        .await
        .expect("list run request queue head");
    assert!(head.items.len() as i64 <= 10);
}

/// Builds and returns a fresh private Actor whose container sleeps for ~60s before exiting.
///
/// Several lifecycle tests (`abort`, `reboot`, `metamorph`) need a run that is reliably still
/// `RUNNING` a few seconds after it starts — `apify/hello-world` finishes in a couple of
/// seconds, which is too fast to hit mid-run reliably. `name_prefix` should be unique per test so
/// concurrent runs of this suite (or of the same suite in another language) don't collide.
async fn create_slow_actor(client: &apify_client::ApifyClient, name_prefix: &str) -> ActorFixture {
    let name = common::unique_name(name_prefix).replace('-', "");
    let name = format!("a{}", &name[..name.len().min(20)]);

    let definition = serde_json::json!({
        "name": name,
        "isPublic": false,
        "versions": [{
            "versionNumber": "0.0",
            "sourceType": "SOURCE_FILES",
            "buildTag": "latest",
            "sourceFiles": [
                {
                    "name": "Dockerfile",
                    "format": "TEXT",
                    "content": "FROM apify/actor-node:20\nCOPY . ./\nCMD node main.js"
                },
                {
                    "name": "main.js",
                    "format": "TEXT",
                    "content": "console.log('sleeping'); setTimeout(() => console.log('woke'), 60000);"
                }
            ]
        }]
    });

    let actor = client
        .actors()
        .create(&definition)
        .await
        .expect("create actor");

    let cleanup_client = client.clone();
    let cleanup_id = actor.id.clone();
    let guard = common::Cleanup::new(move || async move {
        let _ = cleanup_client.actor(&cleanup_id).delete().await;
    });

    let actor_client = client.actor(&actor.id);
    let build = actor_client
        .build("0.0", Default::default())
        .await
        .expect("start build");
    client
        .build(&build.id)
        .wait_for_finish(Some(120))
        .await
        .expect("wait for build");

    ActorFixture {
        id: actor.id,
        _cleanup: guard,
    }
}

/// A slow Actor created by [`create_slow_actor`], kept alive with its cleanup guard.
struct ActorFixture {
    id: String,
    _cleanup: common::Cleanup,
}

/// Lifecycle: abort a running Actor run.
#[tokio::test(flavor = "multi_thread")]
async fn run_abort() {
    let client = require_client!();
    let actor = create_slow_actor(&client, "run-abort").await;
    let actor_client = client.actor(&actor.id);

    let run = actor_client
        .start(None::<&serde_json::Value>, Default::default())
        .await
        .expect("start slow run");
    let run_client = client.run(&run.id);

    let aborted = run_client
        .abort(Some(false))
        .await
        .expect("abort running Actor");
    assert!(
        matches!(
            aborted.status.as_deref(),
            Some("ABORTING") | Some("ABORTED")
        ),
        "expected ABORTING or ABORTED after abort, got {:?}",
        aborted.status
    );

    let finished = run_client
        .wait_for_finish(Some(60))
        .await
        .expect("wait for aborted run to settle");
    assert_eq!(finished.status.as_deref(), Some("ABORTED"));
}

/// Lifecycle: reboot a running Actor run (restarts its container, keeping the run ID and
/// storages). The run is aborted afterward to avoid burning extra compute.
#[tokio::test(flavor = "multi_thread")]
async fn run_reboot() {
    let client = require_client!();
    let actor = create_slow_actor(&client, "run-reboot").await;
    let actor_client = client.actor(&actor.id);

    let run = actor_client
        .start(None::<&serde_json::Value>, Default::default())
        .await
        .expect("start slow run");
    let run_client = client.run(&run.id);

    let rebooted = run_client.reboot().await.expect("reboot running Actor");
    assert_eq!(rebooted.id, run.id, "reboot must keep the same run ID");
    assert!(
        !rebooted.is_terminal(),
        "a freshly-rebooted run should still be active, got status {:?}",
        rebooted.status
    );

    let _ = run_client.abort(Some(false)).await;
}

/// Lifecycle: resurrecting a finished run starts it again (a new active run reusing the same
/// run ID). The resurrected run is aborted immediately afterward to avoid burning extra compute.
#[tokio::test(flavor = "multi_thread")]
async fn run_resurrect() {
    let client = require_client!();
    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), Some(120))
        .await
        .expect("call hello-world actor");
    assert_eq!(run.status.as_deref(), Some("SUCCEEDED"));

    let run_client = client.run(&run.id);
    let resurrected = run_client
        .resurrect(Default::default())
        .await
        .expect("resurrect finished run");
    assert!(
        !resurrected.is_terminal(),
        "a just-resurrected run should be active again, got status {:?}",
        resurrected.status
    );

    // Clean up the extra compute immediately; we only need to prove `resurrect` restarts it.
    let _ = run_client.abort(Some(false)).await;
}

/// Lifecycle: metamorphing a running Actor run into another Actor's run swaps it in place —
/// waiting for the (now `apify/hello-world`) run to finish should succeed.
#[tokio::test(flavor = "multi_thread")]
async fn run_metamorph() {
    let client = require_client!();
    let actor = create_slow_actor(&client, "run-morph").await;
    let actor_client = client.actor(&actor.id);

    let run = actor_client
        .start(None::<&serde_json::Value>, Default::default())
        .await
        .expect("start slow run");
    let run_client = client.run(&run.id);

    // `act_id` stays the original Actor's ID after metamorph (the run is still conceptually
    // "owned" by the Actor that started it); the target Actor's code is what actually runs. The
    // meaningful proof that the swap happened is behavioral: our own Actor's script sleeps for
    // ~60s and would never finish this fast on its own, so a `SUCCEEDED` result well within the
    // wait budget below can only mean the run's container is now actually executing
    // `apify/hello-world`, which exits in a few seconds.
    let morphed = run_client
        .metamorph::<serde_json::Value>("apify/hello-world", None, Default::default())
        .await
        .expect("metamorph into hello-world");
    assert_eq!(morphed.id, run.id, "metamorph must keep the same run ID");

    let finished = run_client
        .wait_for_finish(Some(30))
        .await
        .expect("wait for morphed run");
    assert_eq!(
        finished.status.as_deref(),
        Some("SUCCEEDED"),
        "the morphed run should finish quickly as a successful hello-world run, not keep \
         running the original ~60s sleep script"
    );
}
