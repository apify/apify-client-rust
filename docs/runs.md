# Actor runs

Obtained via `client.runs()` (collection) and `client.run(id)` (single). Nested run
collections are available via `actor.runs()` and `task.runs()`.

> Code blocks below use the rustdoc `# `-hidden-line convention — see
> [`docs/README.md`](README.md) for what those lines are.

## `RunCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options, filter)` | `ListOptions`, `RunListOptions { status, started_after, started_before }` | `PaginationList<ActorRun>` | Lists runs, optionally filtered by status and start time. |
| `iterate(options, filter)` | `ListOptions`, `RunListOptions` | `ListIterator<ActorRun>` | Lazily iterates all runs across pages (auto-pagination). |

## `RunClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<ActorRun>` | Fetches the run. Returns immediately (no server-side wait); see `get_with_options`. |
| `get_with_options(options)` | `RunGetOptions { wait_for_finish: Option<i64> }` | `Option<ActorRun>` | Fetches the run, optionally waiting server-side (max 60s) for it to reach a terminal state before responding. Also reachable via `actor.last_run()`/`task.last_run()`, which share `RunClient`. |
| `update(fields)` | `&impl Serialize` | `ActorRun` | Updates the run (e.g. status message). |
| `delete()` | — | `()` | Deletes the run. |
| `abort(gracefully)` | `Option<bool>` | `ActorRun` | Aborts the run. `None` omits the param (server default, immediate); `Some(true)`/`Some(false)` abort gracefully/immediately. |
| `metamorph(target, input, options)` | `&str`, `Option<&impl Serialize>`, `RunMetamorphOptions` | `ActorRun` | Transforms the run into another Actor. |
| `reboot()` | — | `ActorRun` | Reboots the run's container. |
| `resurrect(options)` | `RunResurrectOptions` | `ActorRun` | Resurrects a finished run. |
| `charge(options)` | `RunChargeOptions` | `()` | Charges a pay-per-event run (always sends an idempotency key). |
| `wait_for_finish(wait_secs)` | `Option<i64>` | `ActorRun` | Polls until the run is terminal. `None` waits indefinitely; `Some(n)` bounds the wait and may return a still-running (non-terminal) run if `n` elapses first. |
| `dataset()` / `key_value_store()` / `request_queue()` / `log()` | — | resource client | Access the run's default storages and log. |
| `get_streamed_log()` | — | `Result<Option<impl Stream<Item = Result<Vec<u8>>>>>` (async — `.await` it) | Convenience for `log().stream()` — streams the run's log chunks live (log redirection), or `None` if the log does not exist. |
| `get_streamed_log_with_options(options)` | `LogOptions` | `Result<Option<impl Stream<Item = Result<Vec<u8>>>>>` (async — `.await` it) | As `get_streamed_log()`, forwarding `LogOptions` (e.g. `raw`) to the log stream. |

`get_streamed_log()` / `get_streamed_log_with_options()` are `async` and yield the stream inside a
`Result<Option<_>>`, so `.await?` them, then handle the `Option` (`None` means the log does not
exist). Polling the returned stream with `.next()` requires the
`futures_util::StreamExt` trait (from the `futures-util` crate — add `futures-util = "0.3"` to your
`Cargo.toml`) in scope; the [`raw_log`](../examples/raw_log.rs) example drives
`get_streamed_log_with_options` exactly this way. See
[Logs](misc.md#logs--clientlogbuild_or_run_id) for a full streaming snippet.

`RunGetOptions` (the argument to `get_with_options`): `wait_for_finish: Option<i64>` — maximum
time, in seconds (capped at 60 by the API), to wait server-side for the run to reach a terminal
state before returning; `None` (the default, and what plain `get()` uses) returns immediately
without waiting. This is a single bounded server-side wait, distinct from
[`RunClient::wait_for_finish`](#runclient), which polls repeatedly (using this same parameter
internally) until the run finishes or a client-side budget is exhausted.

`RunResurrectOptions` (all optional): `build: Option<String>`, `memory_mbytes: Option<i64>`,
`timeout_secs: Option<i64>`, `max_items: Option<i64>`, `max_total_charge_usd: Option<f64>`,
`restart_on_error: Option<bool>`.

`RunMetamorphOptions` (both optional): `build: Option<String>`, `content_type: Option<String>`
(defaults to `application/json`).

`RunChargeOptions`: `event_name: String` (required), `count: Option<i64>` (defaults to `1`),
`idempotency_key: Option<String>` (auto-generated when omitted).

`ActorRun.status` is a stringly-typed `Option<String>` carrying the API's run status. Known
values are `READY`, `RUNNING`, `SUCCEEDED`, `FAILED`, `ABORTING`, `ABORTED`, `TIMING-OUT`, and
`TIMED-OUT`; the terminal ones (`SUCCEEDED`, `FAILED`, `ABORTED`, `TIMED-OUT`) are what
`is_terminal()` reports and what `wait_for_finish` polls for.

The `last_run(status)` methods on `ActorClient` ([Actors](actors.md)) and `TaskClient`
([Actor tasks](tasks.md)) take these same `status` values (e.g. `last_run(Some("SUCCEEDED"))`); pass
`None` to leave it unfiltered. To additionally filter by `origin`, use the companion
`last_run_with_options(LastRunOptions { status, origin })`. `origin` restricts the last run by how
it was started; accepted values are the platform's run origins, the most common being
`DEVELOPMENT`, `WEB`, `API`, and `SCHEDULER` (e.g.
`last_run_with_options(LastRunOptions { status: Some("SUCCEEDED".into()), origin: Some("WEB".into()) })`).
Leave a field as `None` to omit it.

## `ActorRun` fields

`ActorRun` (from `apify_client::models`) is returned by `start`, `call`, `get`, `abort`,
`wait_for_finish`, and the run `list`. The fields most callers read — including the
`run.status` / `run.default_dataset_id` accessed in the [README Quick start](../README.md#quick-start):

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique run ID (always present); used to build a `client.run(&run.id)` client. |
| `act_id` | `Option<String>` | ID of the Actor that produced the run. |
| `actor_task_id` | `Option<String>` | ID of the task that started the run, if any. |
| `user_id` | `Option<String>` | ID of the user who owns the run. |
| `status` | `Option<String>` | Current run status (see the status values above). |
| `status_message` | `Option<String>` | Optional human-readable status message. |
| `started_at` | `Option<DateTime<Utc>>` | When the run started. |
| `finished_at` | `Option<DateTime<Utc>>` | When the run finished (absent while running). |
| `build_id` | `Option<String>` | ID of the build used for the run. |
| `default_dataset_id` | `Option<String>` | Default dataset ID — pass to `client.dataset(..)` to read results. |
| `default_key_value_store_id` | `Option<String>` | Default key-value store ID for the run. |
| `default_request_queue_id` | `Option<String>` | Default request queue ID for the run. |
| `container_url` | `Option<String>` | URL of the run's container, while running. |
| `extra` | `Extra` | Any other fields returned by the API. |

The three `default_*_id` fields are `Option<String>` because they are only populated once the
run has its storages assigned; the storages are reachable directly via `run.dataset()`,
`run.key_value_store()` and `run.request_queue()` (see [storages](storages.md)).

```rust,no_run
use apify_client::ApifyClient;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = ApifyClient::new(std::env::var("APIFY_TOKEN")?);
let run = client
    .actor("apify/hello-world")
    .call::<serde_json::Value>(None, Default::default(), None)
    .await?;

println!("run {} finished with status {:?}", run.id, run.status);
if let Some(dataset_id) = &run.default_dataset_id {
    let items = client
        .dataset(dataset_id)
        .list_items::<serde_json::Value>(Default::default())
        .await?;
    println!("got {} item(s)", items.items.len());
}
# Ok(())
# }
```
