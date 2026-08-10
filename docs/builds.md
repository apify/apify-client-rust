# Actor builds

Obtained via `client.builds()` (collection) and `client.build(id)` (single). Nested build
collections are available via `actor.builds()`.

> Code blocks below use the rustdoc `# `-hidden-line convention — see
> [`docs/README.md`](README.md) for what those lines are.

## `BuildCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions` | `PaginationList<Build>` | Lists builds. |
| `iterate(options)` | `ListOptions` | `ListIterator<Build>` | Lazily iterates all builds across pages (auto-pagination). |

## `BuildClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Build>` | Fetches the build. Returns immediately (no server-side wait); see `get_with_options`. |
| `get_with_options(options)` | `BuildGetOptions { wait_for_finish: Option<i64> }` | `Option<Build>` | Fetches the build, optionally waiting server-side (max 60s) for it to reach a terminal state before responding. |
| `abort()` | — | `Build` | Aborts the build. |
| `delete()` | — | `()` | Deletes the build. |
| `wait_for_finish(wait_secs)` | `Option<i64>` | `Build` | Polls until the build is terminal. |
| `get_openapi_definition()` | — | `Option<serde_json::Value>` | Fetches the OpenAPI definition generated for the build (raw JSON, endpoint `.../openapi.json`). |
| `log()` | — | `LogClient` | Access the build's log. |

`BuildGetOptions` (the argument to `get_with_options`): `wait_for_finish: Option<i64>` —
maximum time, in seconds (capped at 60 by the API), to wait server-side for the build to reach a
terminal state before returning; `None` (the default, and what plain `get()` uses) returns
immediately without waiting. This is a single bounded server-side wait, distinct from
[`BuildClient::wait_for_finish`](#buildclient), which polls repeatedly until the build finishes
or a client-side budget is exhausted. The same server-side-wait semantics apply to
`ActorClient::default_build`'s `wait_for_finish` argument (see
[Actors → `ActorClient`](actors.md#actorclient)), which resolves the Actor's default build via
the analogous `waitForFinish` query parameter.

The returned `Build` model's fields (`id`, `status`, `build_number`, …) are documented in
[actors.md → `Build` fields](actors.md#build-fields).
