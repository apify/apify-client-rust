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
| `wait_for_finish(wait_secs)` | `Option<i64>` | `ActorRun` | Polls until the run is terminal. |
| `dataset()` / `key_value_store()` / `request_queue()` / `log()` | — | resource client | Access the run's default storages and log. |

`RunResurrectOptions`: `build`, `memory_mbytes`, `timeout_secs`, `max_items`, `max_total_charge_usd`, `restart_on_error` (all optional).

`RunMetamorphOptions`: `build`, `content_type` (both optional; `content_type` defaults to `application/json`).

`RunChargeOptions`: `event_name` (required), `count` (defaults to `1`), `idempotency_key` (auto-generated when omitted).
