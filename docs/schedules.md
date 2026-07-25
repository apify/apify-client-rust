# Schedules

Obtained via `client.schedules()` (collection) and `client.schedule(id)` (single).

## `ScheduleCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions` | `PaginationList<Schedule>` | Lists schedules. |
| `iterate(options)` | `ListOptions` | `ListIterator<Schedule>` | Lazily iterates all schedules across pages (auto-pagination). |
| `create(schedule)` | `&impl Serialize` | `Schedule` | Creates a schedule. |

## `ScheduleClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Schedule>` | Fetches the schedule. |
| `update(fields)` | `&impl Serialize` | `Schedule` | Updates the schedule. |
| `delete()` | — | `()` | Deletes the schedule. |
| `get_log()` | — | `Option<String>` | Fetches the schedule's invocation log. |

### Creating a schedule

`create` takes the same shape as the
[Create schedule API](https://docs.apify.com/api/v2/schedules-post): `cronExpression` and
`isEnabled` are required, plus an `actions` array of `{ type, actorId | actorTaskId, ... }` objects
describing what the schedule runs (`type` is `"RUN_ACTOR"` or `"RUN_ACTOR_TASK"`).

```rust,no_run
use apify_client::ApifyClient;
use serde_json::json;

# async fn run(client: ApifyClient, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
let schedule = client
    .schedules()
    .create(&json!({
        "name": "my-daily-schedule",
        "cronExpression": "0 12 * * *",
        "isEnabled": true,
        "actions": [
            { "type": "RUN_ACTOR_TASK", "actorTaskId": task_id }
        ]
    }))
    .await?;
println!("created schedule {}", schedule.id);
# Ok(())
# }
```

See the [`tasks_schedules_webhooks`](../examples/tasks_schedules_webhooks.rs) example for a
schedule bound to a task, plus `get_log`, `update` and `delete`.

## The `Schedule` model

`Schedule` lives in `apify_client::models` (`use apify_client::models::Schedule;`). Returned by
`get`, `create`, `update` and (as `PaginationList<Schedule>`) by `list`. Unknown fields returned
by the API are preserved in `extra`.

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique schedule ID (always present); used to build a `client.schedule(&schedule.id)` client. |
| `user_id` | `Option<String>` | ID of the user who owns the schedule. |
| `name` | `Option<String>` | Technical name of the schedule. |
| `cron_expression` | `Option<String>` | The cron expression that determines when the schedule fires. |
| `is_enabled` | `Option<bool>` | Whether the schedule is currently enabled. |
| `extra` | `Extra` | Any other fields returned by the API. |
