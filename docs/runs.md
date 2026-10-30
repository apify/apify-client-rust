# Actor runs

Obtained via `client.runs()` (collection) and `client.run(id)` (single). Nested run
collections are available via `actor.runs()` and `task.runs()`.

## `RunCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options, filter)` | `ListOptions`, `RunListOptions { status, started_after, started_before }` | `PaginationList<ActorRun>` | Lists runs, optionally filtered by status and start time. |
| `iterate(options, filter)` | `ListOptions`, `RunListOptions` | `ListIterator<ActorRun>` | Lazily iterates all runs across pages (auto-pagination). |

## `RunClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<ActorRun>` | Fetches the run. |
| `iterate_dataset_items::<T>(options)` | `DatasetListItemsOptions` | `RunDatasetItemsIterator<T>` | Lazily iterates the items of the run's default dataset while the run is still producing them, until it finishes. `with_chunk_size(n)` sets the rows per request (default 1000) and `with_poll_interval(d)` the wait for the run between polls (default 5s). `offset` and `limit` of the options bound the rows that are read; `desc: Some(true)` is rejected with `InvalidArgument`. A `last_run()` client sticks to the run its first request resolves to. |
| `update(fields)` | `&impl Serialize` | `ActorRun` | Updates the run (e.g. status message). |
| `delete()` | — | `()` | Deletes the run. |
| `abort(gracefully)` | `Option<bool>` | `ActorRun` | Aborts the run. `None` omits the param (server default, immediate); `Some(true)`/`Some(false)` abort gracefully/immediately. |
| `metamorph(target, input, options)` | `&str`, `Option<&impl Serialize>`, `RunMetamorphOptions` | `ActorRun` | Transforms the run into another Actor. |
| `metamorph_raw(target, input, options)` | `&str`, `&[u8]`, `RunMetamorphOptions` | `ActorRun` | Like `metamorph`, but sends `input` as a raw request body (no JSON serialization), paired with `options.content_type`. |
| `reboot()` | — | `ActorRun` | Reboots the run's container. |
| `resurrect(options)` | `RunResurrectOptions` | `ActorRun` | Resurrects a run in a terminal state (`SUCCEEDED`, `FAILED`, `ABORTED`, `TIMED-OUT`). To continue a run that reached its cost limit, pass a higher `max_total_charge_usd`; the limit covers the whole run, including the cost before the resurrection. |
| `charge(options)` | `RunChargeOptions` | `()` | Charges a pay-per-event run (always sends an idempotency key). |
| `wait_for_finish(wait_secs)` | `Option<i64>` | `ActorRun` | Polls until the run is terminal. `None` waits indefinitely, without a client-side timeout on the polls; `Some(n)` bounds the wait, gives each poll the `Medium` tier plus the hold it asks the API for, and may return a still-running (non-terminal) run if `n` elapses first. A timeout set with `with_timeout` replaces both. A run ID that keeps 404ing past a 3-second grace period (reset by any successful fetch) returns the `404` as an error instead of polling forever. |
| `dataset()` / `key_value_store()` / `request_queue()` / `log()` | — | resource client | Access the run's default storages and log. |
| `get_streamed_log(options)` | `StreamedLogOptions { to_log, from_start }` | `Result<Option<StreamedLog>>` (async — `.await` it) | The [redirection](#log-redirection) of the run's log; `None` if `to_log` is `LogRedirect::Disabled`, or, for `LogRedirect::Default`, if the run does not exist. A custom sink makes no request, so a missing run shows up as the error of `stop()`. |

### Log redirection

`get_streamed_log` returns a `StreamedLog`: `start()` spawns a task that follows the run's log, and
`stop()` ends it (waiting up to a second for the lines in flight, then reading the rest of the
log). The log is split into messages at the timestamps its lines begin with, so a stack trace
stays in one piece, and each message goes to a `LogSink`:

- `LogRedirect::Default` prints to the standard output, prefixing each message with
  `"{Actor name} runId:{run id} -> "`. It costs a request for the run and one for its Actor.
- `LogRedirect::to_sink(sink)` sends the messages, without a prefix, to a `LogSink`: a trait with
  one method, implemented by every `Fn(&str) + Send + Sync + 'static` closure, so it can forward to
  a logging framework.
- `LogRedirect::Disabled` redirects nothing.

`from_start: false` drops the messages stamped before the call. A run that has not logged yet is
followed by reopening its empty log stream until the first bytes arrive. `ActorClient::call` does
all of this for the run it waits for; when the redirection ends early on an error, it passes a
`Log redirection stopped due to error: ..` message to the sink instead of failing the call. To read the log as raw chunks instead, use
`run.log().stream_with_options(..)`; the [`raw_log`](../examples/raw_log.rs) example does.

```rust,no_run
use apify_client::{ApifyClient, LogRedirect, StreamedLogOptions};

# async fn run(client: ApifyClient, run_id: &str) -> Result<(), Box<dyn std::error::Error>> {
let run = client.run(run_id);
let mut streamed_log = run
    .get_streamed_log(StreamedLogOptions {
        to_log: LogRedirect::to_sink(|message: &str| println!("run -> {message}")),
        from_start: true,
    })
    .await?
    .ok_or("the run does not exist")?;
streamed_log.start()?;
run.wait_for_finish(None).await?;
streamed_log.stop().await?;
# Ok(())
# }
```

`RunResurrectOptions`: `build`, `memory_mbytes`, `run_timeout_secs`, `max_items`, `max_total_charge_usd`, `restart_on_error` (all optional).

`RunMetamorphOptions`: `build`, `content_type` (both optional; `content_type` defaults to `application/json`).

`RunChargeOptions`: `event_name` (required), `count` (defaults to `1`), `idempotency_key`
(auto-generated when omitted; the API requires this header and forgets the key 3 minutes after
the charge, so a retry past that window creates a new charge instead of being deduplicated),
`request_id` (ID of the Server Actor request being charged, from the `X-Actor-Request-ID` header of
the incoming request; required for multi-tenant Server Actor runs, where it maps the charge to the
caller's account; omitted from the request when `None`).

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
| `container_url` | `Option<String>` | URL of the run's container, while running. Normalized to its WHATWG URL serialization, like every URL field below. |
| `console_url` | `Option<String>` | URL of the run in Apify Console. |
| `output` | `Option<HashMap<String, String>>` | Output links of the run, generated from the output schema of the Actor build (present only if the build defines one). |
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
    .call::<serde_json::Value>(None, Default::default())
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
