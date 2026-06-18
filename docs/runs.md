# Actor runs

Obtained via `client.runs()` (collection) and `client.run(id)` (single). Nested run
collections are available via `actor.runs()` and `task.runs()`.

## `RunCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options, status)` | `ListOptions`, `Option<&str>` | `PaginationList<ActorRun>` | Lists runs, optionally filtered by status. |

## `RunClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<ActorRun>` | Fetches the run. |
| `update(fields)` | `&impl Serialize` | `ActorRun` | Updates the run (e.g. status message). |
| `delete()` | — | `()` | Deletes the run. |
| `abort(gracefully)` | `bool` | `ActorRun` | Aborts the run. |
| `metamorph(target, input, build)` | `&str`, `Option<&impl Serialize>`, `Option<&str>` | `ActorRun` | Transforms the run into another Actor. |
| `reboot()` | — | `ActorRun` | Reboots the run's container. |
| `resurrect(options)` | `RunResurrectOptions` | `ActorRun` | Resurrects a finished run. |
| `charge(event_name, count)` | `&str`, `i64` | `()` | Charges a pay-per-event run. |
| `wait_for_finish(wait_secs)` | `Option<i64>` | `ActorRun` | Polls until the run is terminal. |
| `dataset()` / `key_value_store()` / `request_queue()` / `log()` | — | resource client | Access the run's default storages and log. |

`RunResurrectOptions`: `build`, `memory_mbytes`, `timeout_secs` (all optional).
