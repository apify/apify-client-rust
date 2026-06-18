# Schedules

Obtained via `client.schedules()` (collection) and `client.schedule(id)` (single).

## `ScheduleCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions` | `PaginationList<Schedule>` | Lists schedules. |
| `create(schedule)` | `&impl Serialize` | `Schedule` | Creates a schedule. |

## `ScheduleClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Schedule>` | Fetches the schedule. |
| `update(fields)` | `&impl Serialize` | `Schedule` | Updates the schedule. |
| `delete()` | — | `()` | Deletes the schedule. |
| `get_log()` | — | `Option<String>` | Fetches the schedule's invocation log. |
