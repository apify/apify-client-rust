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
| `last_run(status)` | `Option<&str>` | `RunClient` | The task's last run. |
| `runs()` | — | `RunCollectionClient` | The task's runs. |
| `webhooks()` | — | `WebhookCollectionClient` | The task's webhooks. |
