# Actor tasks

Obtained via `client.tasks()` (collection) and `client.task(id)` (single).

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
| `publish()` | — | `Task` | Publishes the task on its public landing page (sets `isPublic: true`). Requires the task's Actor to be public and the task to have `publicConfig` set up. |
| `unpublish()` | — | `Task` | Unpublishes the task from its public landing page (sets `isPublic: false`), preserving `publicConfig`. |
| `delete()` | — | `()` | Deletes the task. |
| `start(input, options)` | `Option<&impl Serialize>`, `ActorStartOptions` | `ActorRun` | Starts a run. See [`ActorStartOptions`](actors.md#actorstartoptions) for the full field list. |
| `call(input, options, wait_secs)` | `Option<&impl Serialize>`, `ActorStartOptions`, `Option<i64>` | `ActorRun` | Starts a run and waits. Same [`ActorStartOptions`](actors.md#actorstartoptions) as `start`. |
| `get_input()` / `update_input(input)` | — / `&impl Serialize` | `Option<Value>` / `Value` | The task's saved input. |
| `last_run(status)` | `Option<&str>` | `RunClient` | The task's last run, optionally filtered by status. See [Actor runs](runs.md) for the accepted `status` values. |
| `last_run_with_options(options)` | `LastRunOptions { status, origin }` | `RunClient` | The task's last run, optionally filtered by status and/or origin. See [Actor runs](runs.md) for the accepted `status` and `origin` values (common origins: `DEVELOPMENT`, `WEB`, `API`, `SCHEDULER`). |
| `runs()` | — | `RunCollectionClient` | The task's runs. |
| `webhooks()` | — | `WebhookCollectionClient` | The task's webhooks. |

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
| `is_public` | `Option<bool>` | Whether the task is published on its public landing page. Derived from `public_config.published_at`; set it via `publish()`/`unpublish()` or `update()`. |
| `public_config` | `Option<TaskPublicConfig>` | The task's public landing page display configuration, or `None` if never configured. |
| `extra` | `Extra` | Any other fields returned by the API. |

## The `TaskPublicConfig` model

`TaskPublicConfig` lives in `apify_client::models` (`use apify_client::models::TaskPublicConfig;`).
The task is published when `published_at` is set and unpublished when it is `None`; `published_at`
is server-controlled (read-only) - use `publish()`/`unpublish()` to change the publication state.

| Field | Type | Description |
|---|---|---|
| `published_at` | `Option<DateTime<Utc>>` | When the task was published, or `None` if unpublished. Read-only. |
| `seo_title` | `Option<String>` | Name shown by search engines. Defaults to the task title when unset. |
| `seo_description` | `Option<String>` | Description shown by search engines. Defaults to the task description when unset. |
| `input_schema_fields` | `Option<Vec<String>>` | Names of the task input fields displayed on the public task page. |
| `dataset_name` | `Option<String>` | Name of the Actor dataset schema entry whose results are displayed. `None` uses the Actor's default dataset. |
| `dataset_view` | `Option<String>` | Key of the dataset view (from the Actor's dataset schema) used to display results. Required to publish the task. |
