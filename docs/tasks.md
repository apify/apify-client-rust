# Actor tasks

Obtained via `client.tasks()` (collection) and `client.task(id)` (single).

> Code blocks below use the rustdoc `# `-hidden-line convention — see
> [`docs/README.md`](README.md) for what those lines are.

## `TaskCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions` | `PaginationList<Task>` | Lists tasks. |
| `iterate(options)` | `ListOptions` | `ListIterator<Task>` | Lazily iterates all tasks across pages (auto-pagination). |
| `create(task)` | `&impl Serialize` | `Task` | Creates a task. |

## `TaskClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Task>` | Fetches the task. |
| `update(fields)` | `&impl Serialize` | `Task` | Updates the task. |
| `delete()` | — | `()` | Deletes the task. |
| `start(input, options)` | `Option<&impl Serialize>`, `TaskStartOptions` | `ActorRun` | Starts a run. See [`TaskStartOptions`](#taskstartoptions-and-taskcalloptions) below. |
| `call(input, options, wait_secs)` | `Option<&impl Serialize>`, `TaskCallOptions`, `Option<i64>` | `ActorRun` | Starts a run and waits. See [`TaskCallOptions`](#taskstartoptions-and-taskcalloptions) below. |
| `get_input()` / `update_input(input)` | — / `&impl Serialize` | `Option<Value>` / `Value` | The task's saved input. |
| `last_run(status)` | `Option<&str>` | `RunClient` | The task's last run, optionally filtered by status. See [Actor runs](runs.md) for the accepted `status` values. |
| `last_run_with_options(options)` | `LastRunOptions { status: Option<String>, origin: Option<String> }` | `RunClient` | The task's last run, optionally filtered by status and/or origin. See [Actor runs](runs.md) for the accepted `status` and `origin` values (common origins: `DEVELOPMENT`, `WEB`, `API`, `SCHEDULER`). |
| `runs()` | — | `RunCollectionClient` | The task's runs. |
| `webhooks()` | — | `WebhookCollectionClient` | The task's webhooks. |

### `TaskStartOptions` and `TaskCallOptions`

Both are narrowed versions of [`ActorStartOptions`](actors.md#actorstartoptions), matching the
JS reference client's `TaskStartOptions`/`TaskCallOptions`. A task's input content type is fixed
and a task run does not accept a permission-level override, so neither type has a `content_type`
or `force_permission_level` field (present on `ActorStartOptions` for Actor `start`/`call`).
`TaskCallOptions` additionally drops `wait_for_finish` (the server-side wait): `call`'s separate
`wait_secs` argument is how you control call's wait behavior, so the two should not be set
together.

`TaskStartOptions` fields (all optional):

| Field | Type | Description |
|---|---|---|
| `build` | `Option<String>` | Tag or number of the build to run (e.g. `latest`, `0.1.2`). |
| `memory_mbytes` | `Option<i64>` | Memory in megabytes allocated for the run. |
| `timeout_secs` | `Option<i64>` | Timeout for the run in seconds (`0` means no timeout). |
| `wait_for_finish` | `Option<i64>` | Maximum seconds to wait server-side for the run to finish (max 60). |
| `max_items` | `Option<i64>` | Maximum number of dataset items to charge (pay-per-result Actors). |
| `max_total_charge_usd` | `Option<f64>` | Maximum total charge in USD (pay-per-event Actors). |
| `restart_on_error` | `Option<bool>` | Whether to restart the run if it fails. |
| `webhooks` | `Option<Vec<serde_json::Value>>` | Ad-hoc webhooks to attach to this run. |

`TaskCallOptions` has the same fields except `wait_for_finish`.

### Creating a task

`create` takes the same shape as the [Create task API](https://docs.apify.com/api/v2/actor-tasks-post):
at minimum `actId` (the Actor to run) and a `name`; `options` and `input` seed the task's default
run configuration and input (both optional, and both overridable per-run via `start`/`call`).

```rust,no_run
use apify_client::ApifyClient;
use serde_json::json;

# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let task = client
    .tasks()
    .create(&json!({
        "actId": "apify/hello-world",
        "name": "my-hello-world-task",
        "options": { "memoryMbytes": 256 },
        "input": { "message": "hi" }
    }))
    .await?;
println!("created task {}", task.id);
# Ok(())
# }
```

See the [`tasks_schedules_webhooks`](../examples/tasks_schedules_webhooks.rs) example for the
task's full lifecycle (create, run, inspect its webhooks, update, delete).

## The `Task` model

`Task` lives in `apify_client::models` (`use apify_client::models::Task;`). Returned by `get`,
`create`, `update` and (as `PaginationList<Task>`) by `list`. Unknown fields returned by the API
are preserved in `extra`.

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique task ID (always present); used to build a `client.task(&task.id)` client. |
| `act_id` | `Option<String>` | ID of the Actor this task runs. |
| `user_id` | `Option<String>` | ID of the user who owns the task. |
| `name` | `Option<String>` | Technical name of the task, used in API paths. |
| `title` | `Option<String>` | Human-readable title shown in the UI. |
| `created_at` | `Option<DateTime<Utc>>` | When the task was created. |
| `modified_at` | `Option<DateTime<Utc>>` | When the task was last modified. |
| `extra` | `Extra` | Any other fields returned by the API. |
