# Actors

Obtained via `client.actors()` (collection) and `client.actor(id)` (single). The `id` may
be an Actor ID or a `username~name` (or `username/name`) reference.

## `ActorCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ActorListOptions { offset, limit, desc, my, sort_by }` | `PaginationList<Actor>` | Lists your Actors. |
| `iterate(options)` | `ActorListOptions` | `ListIterator<Actor>` | Lazily iterates all Actors across pages (auto-pagination). |
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
| `validate_input(input)` | `&impl Serialize` | `serde_json::Value` | Validates input against the default build's schema. |
| `validate_input_for_build(input, build)` | `&impl Serialize`, `Option<&str>` | `serde_json::Value` | Validates input against a specific build's schema (`build` tag/number; `None` = default). |
| `last_run(status)` | `Option<&str>` | `RunClient` | Client for the last run, optionally filtered by status. See [Actor runs](runs.md) for the accepted `status` values. |
| `last_run_with_options(options)` | `LastRunOptions { status, origin }` | `RunClient` | Client for the last run, optionally filtered by status and/or origin. See [Actor runs](runs.md) for the accepted `status` and `origin` values (common origins: `DEVELOPMENT`, `WEB`, `API`, `SCHEDULER`). |
| `builds()` | — | `BuildCollectionClient` | The Actor's build collection. |
| `runs()` | — | `RunCollectionClient` | The Actor's run collection. |
| `version(n)` / `versions()` | `&str` / — | `ActorVersionClient` / collection | Version management. |
| `webhooks()` | — | `WebhookCollectionClient` | The Actor's webhooks. |

### `ActorStartOptions`

`build`, `memory_mbytes`, `timeout_secs`, `wait_for_finish`, `max_items`,
`max_total_charge_usd`, `content_type` — all optional. Used by both `start` and `call`
(for `call`, `wait_for_finish` is server-side; the `wait_secs` argument controls
client-side polling).

The `wait_secs` argument of `call` (and of `wait_for_finish` on runs/builds) controls the
client-side polling budget:

- `None` polls indefinitely until the run reaches a terminal state.
- `Some(n)` bounds the wait to roughly `n` seconds; if the run has not finished by then, the
  **last fetched (still non-terminal) run is returned** rather than an error — inspect
  `run.status` / `run.is_terminal()` on the result.

> Note: `list` here takes `ActorListOptions` (fields `offset, limit, desc, my, sort_by`),
> which is distinct from the generic `ListOptions { offset, limit, desc }` used by most other
> collection `list` methods (builds, tasks, schedules, webhooks). The dataset/key-value-store/
> request-queue collections take `StorageListOptions`, runs take `ListOptions` + `RunListOptions`,
> and the Store takes `StoreListOptions`.

### `ActorBuildOptions`

All optional:

- `tag` — build tag to assign to the resulting build (e.g. `latest`).
- `use_cache` — reuse cached Docker layers from previous builds to speed the build up.
- `beta_packages` — build against the beta versions of the Apify SDK/CLI packages instead of the
  stable ones.
- `wait_for_finish` — maximum number of seconds the server waits for the build to finish before
  responding (a server-side wait, not client-side polling).

### Input validation

`validate_input` / `validate_input_for_build` check an input value against the Actor's input
schema and return the API's JSON response as `serde_json::Value`. Unlike most endpoints this one
is **not** wrapped in a `{ "data": ... }` envelope — the returned `Value` is the top-level body
`{ "valid": <bool> }`, where `valid` reports whether the input satisfies the schema. A failed
*request* (e.g. unknown `build` tag, missing auth, malformed body) is not reported via `valid`;
it surfaces as an `Err(ApifyClientError)` from the call instead.

```rust,no_run
use apify_client::ApifyClient;
use serde_json::json;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = ApifyClient::new(std::env::var("APIFY_TOKEN")?);
let actor = client.actor("apify~hello-world");

// Validate against the default build's input schema.
let result = actor.validate_input(&json!({ "message": "hi" })).await?;
let is_valid = result.get("valid").and_then(|v| v.as_bool()).unwrap_or(false);
println!("input valid: {is_valid}");

// Validate against a specific build (by tag or version number). `None` == default build.
let result = actor
    .validate_input_for_build(&json!({ "message": "hi" }), Some("latest"))
    .await?;
println!("validated against latest build: {result}");
# Ok(())
# }
```

The `build` argument accepts a build **tag** (e.g. `"latest"`, `"beta"`) or a build **number**
(e.g. `"1.2.34"`); the referenced build must already exist for the API to resolve its schema.

## `Actor` fields

`Actor` (from `apify_client::models`) is returned by `get`, `create`, `update`, and the Actor
`list`. The commonly-used fields — including the `actor.id` read in the
[README error-handling example](../README.md#error-handling) and the `create_build_run_actor`
example:

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique Actor ID (always present); used to build a `client.actor(&actor.id)` client. |
| `user_id` | `Option<String>` | ID of the user who owns the Actor. |
| `name` | `Option<String>` | Technical name used in API paths. |
| `username` | `Option<String>` | Username of the Actor's owner. |
| `title` | `Option<String>` | Human-readable title shown in the UI. |
| `description` | `Option<String>` | Description of what the Actor does. |
| `is_public` | `Option<bool>` | Whether the Actor is published in Apify Store. |
| `created_at` | `Option<DateTime<Utc>>` | When the Actor was created. |
| `modified_at` | `Option<DateTime<Utc>>` | When the Actor was last modified. |
| `extra` | `Extra` | Any other fields returned by the API. |

```rust,no_run
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
if let Some(actor) = client.actor("apify~hello-world").get().await? {
    println!("actor {} ({:?})", actor.id, actor.title.or(actor.name));
}
# Ok(())
# }
```

## `Build` fields

`Build` (from `apify_client::models`) is returned by `build`, `default_build` resolution, `get`,
`abort` and `wait_for_finish` (see also [builds](builds.md)). The fields the
`create_build_run_actor` example reads (`build.id`, `build.status`):

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique build ID (always present); used to build a `client.build(&build.id)` client. |
| `act_id` | `Option<String>` | ID of the Actor that was built. |
| `status` | `Option<String>` | Current build status; the terminal values match the run statuses. |
| `started_at` | `Option<DateTime<Utc>>` | When the build started. |
| `finished_at` | `Option<DateTime<Utc>>` | When the build finished. |
| `build_number` | `Option<String>` | Build number, e.g. `0.1.2`. |
| `extra` | `Extra` | Any other fields returned by the API. |

`Build::is_terminal()` reports whether `status` is a terminal value, mirroring `ActorRun`.

```rust,no_run
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient, build_id: &str) -> Result<(), Box<dyn std::error::Error>> {
let build = client.build(build_id).wait_for_finish(Some(300)).await?;
println!("build {} status {:?}", build.id, build.status);
# Ok(())
# }
```

## Actor versions and environment variables

`ActorVersionClient`: `get`, `update`, `delete`, `env_var(name)`, `env_vars()`.
`ActorVersionCollectionClient`: `list(options)`, `create(version)`.
`ActorEnvVarClient`: `get`, `update`, `delete`.
`ActorEnvVarCollectionClient`: `list()`, `create(env_var)`.
