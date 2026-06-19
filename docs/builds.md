# Actor builds

Obtained via `client.builds()` (collection) and `client.build(id)` (single). Nested build
collections are available via `actor.builds()`.

## `BuildCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions` | `PaginationList<Build>` | Lists builds. |

## `BuildClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Build>` | Fetches the build. |
| `abort()` | — | `Build` | Aborts the build. |
| `delete()` | — | `()` | Deletes the build. |
| `wait_for_finish(wait_secs)` | `Option<i64>` | `Build` | Polls until the build is terminal. |
| `log()` | — | `LogClient` | Access the build's log. |

The returned `Build` model's fields (`id`, `status`, `build_number`, …) are documented in
[actors.md → `Build` fields](actors.md#build-fields).
