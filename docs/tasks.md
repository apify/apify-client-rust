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
| `start(input, options)` | `Option<&impl Serialize>`, `ActorStartOptions` | `ActorRun` | Starts a run. See [`ActorStartOptions`](actors.md#actorstartoptions) for the full field list. |
| `call(input, options, wait_secs)` | `Option<&impl Serialize>`, `ActorStartOptions`, `Option<i64>` | `ActorRun` | Starts a run and waits. Same [`ActorStartOptions`](actors.md#actorstartoptions) as `start`. |
| `get_input()` / `update_input(input)` | — / `&impl Serialize` | `Option<Value>` / `Value` | The task's saved input. |
| `last_run(status)` | `Option<&str>` | `RunClient` | The task's last run, optionally filtered by status. See [Actor runs](runs.md) for the accepted `status` values. |
| `last_run_with_options(options)` | `LastRunOptions { status, origin }` | `RunClient` | The task's last run, optionally filtered by status and/or origin. See [Actor runs](runs.md) for the accepted `status` and `origin` values (common origins: `DEVELOPMENT`, `WEB`, `API`, `SCHEDULER`). |
| `runs()` | — | `RunCollectionClient` | The task's runs. |
| `webhooks()` | — | `WebhookCollectionClient` | The task's webhooks. |

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
