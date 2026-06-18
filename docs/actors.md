# Actors

Obtained via `client.actors()` (collection) and `client.actor(id)` (single). The `id` may
be an Actor ID or a `username~name` (or `username/name`) reference.

## `ActorCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions { offset, limit, desc }` | `PaginationList<Actor>` | Lists your Actors. |
| `create(actor)` | `&impl Serialize` | `Actor` | Creates an Actor from a definition. |

## `ActorClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Actor>` | Fetches the Actor (`None` if missing). |
| `update(fields)` | `&impl Serialize` | `Actor` | Updates the Actor. |
| `delete()` | — | `()` | Deletes the Actor. |
| `start(input, options)` | `Option<&impl Serialize>`, `ActorStartOptions` | `ActorRun` | Starts a run, returns immediately. |
| `call(input, options, wait_secs)` | `Option<&impl Serialize>`, `ActorStartOptions`, `Option<i64>` | `ActorRun` | Starts a run and waits for it to finish. |
| `build(version, options)` | `&str`, `ActorBuildOptions` | `Build` | Builds a version of the Actor. |
| `default_build(wait_for_finish)` | `Option<i64>` | `BuildClient` | Resolves the Actor's default build, optionally waiting up to `wait_for_finish` seconds. |
| `validate_input(input)` | `&impl Serialize` | `serde_json::Value` | Validates input against the schema. |
| `last_run(status)` | `Option<&str>` | `RunClient` | Client for the last run (optionally filtered). |
| `builds()` | — | `BuildCollectionClient` | The Actor's build collection. |
| `runs()` | — | `RunCollectionClient` | The Actor's run collection. |
| `version(n)` / `versions()` | `&str` / — | `ActorVersionClient` / collection | Version management. |
| `webhooks()` | — | `WebhookCollectionClient` | The Actor's webhooks. |

### `ActorStartOptions`

`build`, `memory_mbytes`, `timeout_secs`, `wait_for_finish`, `max_items`,
`max_total_charge_usd`, `content_type` — all optional. Used by both `start` and `call`
(for `call`, `wait_for_finish` is server-side; the `wait_secs` argument controls
client-side polling).

### `ActorBuildOptions`

`beta_packages`, `tag`, `use_cache`, `wait_for_finish` — all optional.

## Actor versions and environment variables

`ActorVersionClient`: `get`, `update`, `delete`, `env_var(name)`, `env_vars()`.
`ActorVersionCollectionClient`: `list(options)`, `create(version)`.
`ActorEnvVarClient`: `get`, `update`, `delete`.
`ActorEnvVarCollectionClient`: `list()`, `create(env_var)`.
