# Actor runs

Obtained via `client.runs()` (collection) and `client.run(id)` (single). Nested run
collections are available via `actor.runs()` and `task.runs()`.

## `RunCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options, filter)` | `ListOptions`, `RunListOptions { status, started_after, started_before }` | `PaginationList<ActorRun>` | Lists runs, optionally filtered by status and start time. |

## `RunClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<ActorRun>` | Fetches the run. |
| `update(fields)` | `&impl Serialize` | `ActorRun` | Updates the run (e.g. status message). |
| `delete()` | — | `()` | Deletes the run. |
| `abort(gracefully)` | `Option<bool>` | `ActorRun` | Aborts the run. `None` omits the param (server default, immediate); `Some(true)`/`Some(false)` abort gracefully/immediately. |
| `metamorph(target, input, options)` | `&str`, `Option<&impl Serialize>`, `RunMetamorphOptions` | `ActorRun` | Transforms the run into another Actor. |
| `reboot()` | — | `ActorRun` | Reboots the run's container. |
| `resurrect(options)` | `RunResurrectOptions` | `ActorRun` | Resurrects a finished run. |
| `charge(options)` | `RunChargeOptions` | `()` | Charges a pay-per-event run (always sends an idempotency key). |
| `wait_for_finish(wait_secs)` | `Option<i64>` | `ActorRun` | Polls until the run is terminal. `None` waits indefinitely; `Some(n)` bounds the wait and may return a still-running (non-terminal) run if `n` elapses first. |
| `dataset()` / `key_value_store()` / `request_queue()` / `log()` | — | resource client | Access the run's default storages and log. |
| `get_streamed_log()` | — | `Stream<Item = Result<Vec<u8>>>` | Convenience for `log().stream()` — streams the run's log chunks live (log redirection). |
| `get_streamed_log_with_options(options)` | `LogOptions` | `Stream<Item = Result<Vec<u8>>>` | As `get_streamed_log()`, forwarding `LogOptions` (e.g. `raw`) to the log stream. |

`RunResurrectOptions`: `build`, `memory_mbytes`, `timeout_secs`, `max_items`, `max_total_charge_usd`, `restart_on_error` (all optional).

`RunMetamorphOptions`: `build`, `content_type` (both optional; `content_type` defaults to `application/json`).

`RunChargeOptions`: `event_name` (required), `count` (defaults to `1`), `idempotency_key` (auto-generated when omitted).

`ActorRun.status` is a stringly-typed `Option<String>` carrying the API's run status. Known
values are `READY`, `RUNNING`, `SUCCEEDED`, `FAILED`, `ABORTING`, `ABORTED`, `TIMING-OUT`, and
`TIMED-OUT`; the terminal ones (`SUCCEEDED`, `FAILED`, `ABORTED`, `TIMED-OUT`) are what
`is_terminal()` reports and what `wait_for_finish` polls for.

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
