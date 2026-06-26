# Actor tasks

Obtained via `client.tasks()` (collection) and `client.task(id)` (single).

## `TaskCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions` | `PaginationList<Task>` | Lists tasks. |
| `create(task)` | `&impl Serialize` | `Task` | Creates a task. |

## `TaskClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Task>` | Fetches the task. |
| `update(fields)` | `&impl Serialize` | `Task` | Updates the task. |
| `delete()` | — | `()` | Deletes the task. |
| `start(input, options)` | `Option<&impl Serialize>`, `ActorStartOptions` | `ActorRun` | Starts a run. |
| `call(input, options, wait_secs)` | `Option<&impl Serialize>`, `ActorStartOptions`, `Option<i64>` | `ActorRun` | Starts a run and waits. |
| `get_input()` / `update_input(input)` | — / `&impl Serialize` | `Option<Value>` / `Value` | The task's saved input. |
| `last_run(status, origin)` | `Option<&str>`, `Option<&str>` | `RunClient` | The task's last run, optionally filtered by status and/or origin. |
| `runs()` | — | `RunCollectionClient` | The task's runs. |
| `webhooks()` | — | `WebhookCollectionClient` | The task's webhooks. |

## The `Task` model

Returned by `get`, `create`, `update` and (as `PaginationList<Task>`) by `list`. Unknown fields
returned by the API are preserved in `extra`.

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
