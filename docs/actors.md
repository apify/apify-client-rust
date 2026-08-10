# Actors

Obtained via `client.actors()` (collection) and `client.actor(id)` (single). The `id` may
be an Actor ID or a `username~name` (or `username/name`) reference.

> Code blocks below use the rustdoc `# `-hidden-line convention — see
> [`docs/README.md`](README.md) for what those lines are.

## `ActorCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ActorListOptions { offset, limit, desc, my, sort_by }` | `PaginationList<Actor>` | Lists your Actors. |
| `iterate(options)` | `ActorListOptions` | `ListIterator<Actor>` | Lazily iterates all Actors across pages (auto-pagination). |
| `create(actor)` | `&impl Serialize` | `Actor` | Creates an Actor from a definition. |

`ActorListOptions.sort_by` is a string field naming the Actor field to sort by (no fixed enum);
accepted values per the
[Get list of Actors API](https://docs.apify.com/api/v2/acts-get) are `createdAt` (default) and
`stats.lastRunStartedAt`.

### Creating an Actor

`create` takes the same shape as the
[Create Actor API](https://docs.apify.com/api/v2/acts-post): at minimum a `name`, plus
`isPublic` and a `versions` array (each version needs `versionNumber`, `sourceType`, and
source-type-specific fields — `sourceFiles` for `SOURCE_FILES`, `gitRepoUrl` for `GIT_REPO`,
`tarballUrl` for `TARBALL`, `gitHubGistUrl` for `GITHUB_GIST`). Additional versions can be
added later via `versions().create(...)` (see [Actor versions and environment
variables](#actor-versions-and-environment-variables) below).

```rust,no_run
use apify_client::ApifyClient;
use serde_json::json;

# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let actor = client
    .actors()
    .create(&json!({
        "name": "my-rust-actor",
        "isPublic": false,
        "versions": [{
            "versionNumber": "0.0",
            "sourceType": "SOURCE_FILES",
            "buildTag": "latest",
            "sourceFiles": [
                { "name": "Dockerfile", "format": "TEXT",
                  "content": "FROM apify/actor-node:20\nCOPY . ./\nCMD node main.js" },
                { "name": "main.js", "format": "TEXT", "content": "console.log('hi');" }
            ]
        }]
    }))
    .await?;
println!("created actor {}", actor.id);
# Ok(())
# }
```

See the [`create_build_run_actor`](../examples/create_build_run_actor.rs) example for the full
lifecycle (create, build, run, fetch the run log, delete).

## `ActorClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Actor>` | Fetches the Actor (`None` if missing). |
| `update(fields)` | `&impl Serialize` | `Actor` | Updates the Actor. |
| `delete()` | — | `()` | Deletes the Actor. |
| `start::<T: Serialize>(input, options)` | `Option<&T>`, `ActorStartOptions` | `ActorRun` | Starts a run, returns immediately. `T` is a generic type parameter (not argument-position `impl Trait`), so it can be turbofished, e.g. `start::<serde_json::Value>(...)`. |
| `call::<T: Serialize>(input, options, wait_secs)` | `Option<&T>`, `ActorCallOptions`, `Option<i64>` | `ActorRun` | Starts a run and waits for it to finish. See [`ActorCallOptions`](#actorcalloptions) below. |
| `build(version, options)` | `&str`, `ActorBuildOptions` | `Build` | Builds a version of the Actor. |
| `default_build(wait_for_finish)` | `Option<i64>` | `BuildClient` | Resolves the Actor's default build. `wait_for_finish` (max 60) is a *server-side* wait — like [`RunClient::get_with_options`](runs.md#runclient)/[`BuildClient::get_with_options`](builds.md#buildclient), not client-side polling — for the build to reach a terminal state before the server responds; `None` returns immediately with whatever build is currently the default. Returns a `BuildClient` (rather than a `Build`) so you can chain further calls (`.get()`, `.log()`, `.wait_for_finish()`) on the resolved build without a second lookup. |
| `validate_input(input)` | `&impl Serialize` | `bool` | Validates input against the default build's schema; returns whether it is valid. |
| `validate_input_for_build(input, build)` | `&impl Serialize`, `Option<&str>` | `bool` | Validates input against a specific build's schema (`build` tag/number; `None` = default); returns whether it is valid. |
| `last_run(status)` | `Option<&str>` | `RunClient` | Client for the last run, optionally filtered by status. See [Actor runs](runs.md) for the accepted `status` values. |
| `last_run_with_options(options)` | `LastRunOptions { status: Option<String>, origin: Option<String> }` | `RunClient` | Client for the last run, optionally filtered by status and/or origin. See [Actor runs](runs.md) for the accepted `status` and `origin` values (common origins: `DEVELOPMENT`, `WEB`, `API`, `SCHEDULER`). |
| `builds()` | — | `BuildCollectionClient` | The Actor's build collection. |
| `runs()` | — | `RunCollectionClient` | The Actor's run collection. |
| `version(n)` / `versions()` | `&str` / — | `ActorVersionClient` / collection | Version management. |
| `webhooks()` | — | `WebhookCollectionClient` | The Actor's webhooks. |

### `ActorStartOptions`

All fields are optional. Used by `start` (`call` takes the narrower
[`ActorCallOptions`](#actorcalloptions) instead). The task equivalents,
[`TaskStartOptions`/`TaskCallOptions`](tasks.md#taskstartoptions-and-taskcalloptions), are
narrowed versions of this type — see that page for the differences.

| Field | Type | Description |
|---|---|---|
| `build` | `Option<String>` | Tag or number of the build to run (e.g. `latest`, `0.1.2`). |
| `memory_mbytes` | `Option<i64>` | Memory in megabytes allocated for the run. |
| `timeout_secs` | `Option<i64>` | Timeout for the run in seconds (`0` means no timeout). |
| `wait_for_finish` | `Option<i64>` | Maximum seconds to wait server-side for the run to finish (max 60). |
| `max_items` | `Option<i64>` | Maximum number of dataset items to charge (pay-per-result Actors). |
| `max_total_charge_usd` | `Option<f64>` | Maximum total charge in USD (pay-per-event Actors). |
| `content_type` | `Option<String>` | Content type of the input body. Defaults to `application/json`. |
| `restart_on_error` | `Option<bool>` | Whether to restart the run if it fails. |
| `force_permission_level` | `Option<String>` | Override the Actor's permission level for this run. |
| `webhooks` | `Option<Vec<serde_json::Value>>` | Ad-hoc webhooks to attach to this run. Encoded as base64 JSON in the `webhooks` query parameter, matching the reference clients. |

### `ActorCallOptions`

Same fields as [`ActorStartOptions`](#actorstartoptions) except `wait_for_finish`, matching the
JS reference client's `ActorCallOptions` (`Omit<ActorStartOptions, 'waitForFinish'>`).
`wait_for_finish` is the server-side wait, which would otherwise let a caller silently block
server-side (up to 60s) before `call`'s own client-side polling (via `wait_secs`) even begins —
dropping the field removes that footgun. Use `From<ActorCallOptions> for ActorStartOptions` (or
just construct an `ActorStartOptions` directly) if you need to pass an equivalent options value to
`start` instead.

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

All fields optional:

| Field | Type | Description |
|---|---|---|
| `tag` | `Option<String>` | Build tag to assign to the resulting build (e.g. `latest`). |
| `use_cache` | `Option<bool>` | Reuse cached Docker layers from previous builds to speed the build up (default `true`). |
| `beta_packages` | `Option<bool>` | If `true`, build against the beta versions of the Apify SDK/CLI packages instead of the stable ones. |
| `wait_for_finish` | `Option<i64>` | Maximum number of seconds the server waits for the build to finish before responding (a server-side wait, not client-side polling; max 60). |

### Input validation

`validate_input` / `validate_input_for_build` check an input value against the Actor's input
schema and return whether the input is valid, as a plain `bool` — matching the reference client's
`validateInput`, which returns `response.data.valid`. Under the hood the API's response is
**not** wrapped in a `{ "data": ... }` envelope like most endpoints — the top-level body is
`{ "valid": <bool> }` — but that shape is an implementation detail the client unwraps for you. A
failed *request* (e.g. unknown `build` tag, missing auth, malformed body) is not reported via the
returned `bool`; it surfaces as an `Err(ApifyClientError)` from the call instead.

```rust,no_run
use apify_client::ApifyClient;
use serde_json::json;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = ApifyClient::new(std::env::var("APIFY_TOKEN")?);
let actor = client.actor("apify~hello-world");

// Validate against the default build's input schema.
let is_valid = actor.validate_input(&json!({ "message": "hi" })).await?;
println!("input valid: {is_valid}");

// Validate against a specific build (by tag or version number). `None` == default build.
let is_valid = actor
    .validate_input_for_build(&json!({ "message": "hi" }), Some("latest"))
    .await?;
println!("validated against latest build: {is_valid}");
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

Obtained from an `ActorClient` via `version(number)` / `versions()` (the version collection), and
from an `ActorVersionClient` via `env_var(name)` / `env_vars()` (that version's env-var
collection).

### `ActorVersionCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions { offset, limit, desc }` | `PaginationList<ActorVersion>` | Lists the Actor's versions. `options` is accepted for interface stability but ignored: `GET /v2/actors/{actorId}/versions` takes no query parameters and always returns every version in one response, matching the reference client. |
| `iterate(options)` | `ListOptions` | `ListIterator<ActorVersion>` | Lazily iterates all versions (in practice a single unpaginated fetch, for the same reason as `list`). |
| `create(version)` | `&impl Serialize` | `ActorVersion` | Creates a new version. |

### `ActorVersionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<ActorVersion>` | Fetches the version (`None` if missing). |
| `update(fields)` | `&impl Serialize` | `ActorVersion` | Updates the version. |
| `delete()` | — | `()` | Deletes the version. |
| `env_var(name)` | `&str` | `ActorEnvVarClient` | Client for one of the version's environment variables. |
| `env_vars()` | — | `ActorEnvVarCollectionClient` | Client for the version's environment-variable collection. |

`create`/`update` take the same shape as the
[Create Actor version API](https://docs.apify.com/api/v2/actors-actor-id-versions-post): at
minimum `versionNumber` and `sourceType` (`SOURCE_FILES`, `GIT_REPO`, `TARBALL`, or
`GITHUB_GIST`), plus the source-type-specific field (`sourceFiles`, `gitRepoUrl`, `tarballUrl`,
or `gitHubGistUrl`) and optionally `buildTag`, `envVars`, `applyEnvVarsToBuild`.

### `ActorEnvVarCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list()` | — | `PaginationList<ActorEnvVar>` | Lists the version's environment variables. |
| `iterate()` | — | `ListIterator<ActorEnvVar>` | Iterates the environment variables (single page; see below). |
| `create(env_var)` | `&ActorEnvVar` | `ActorEnvVar` | Creates a new environment variable. |

`iterate()` is not offset-paginated — the API returns every variable in one page — so it fetches
once and yields all of them; it exists for interface parity with the other collection clients.

### `ActorEnvVarClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<ActorEnvVar>` | Fetches the environment variable by name (`None` if missing). |
| `update(env_var)` | `&ActorEnvVar` | `ActorEnvVar` | Updates the environment variable. |
| `delete()` | — | `()` | Deletes the environment variable. |

### `ActorVersion` fields

`ActorVersion` (from `apify_client::models`) is returned by `get`, `create`, `update`, and the
version `list`.

| Field | Type | Description |
|---|---|---|
| `version_number` | `String` | The version number, e.g. `0.1` (always present). |
| `source_type` | `Option<String>` | Source type: `SOURCE_FILES`, `GIT_REPO`, `TARBALL`, or `GITHUB_GIST`. |
| `extra` | `Extra` | Everything else — `sourceFiles`/`gitRepoUrl`/`tarballUrl`/`gitHubGistUrl`, `buildTag`, `envVars`, `applyEnvVarsToBuild`, and any other fields returned by the API. |

### `ActorEnvVar` fields

`ActorEnvVar` (from `apify_client::models`) is the payload for `env_vars().create(...)` and
`env_var(name).update(...)`, and the type returned by `get`/`create`/`update`/`list`.

| Field | Type | Description |
|---|---|---|
| `name` | `String` | The environment variable name (always present). |
| `value` | `Option<String>` | The value; may be omitted in responses for secret variables. |
| `is_secret` | `Option<bool>` | Whether the variable is a secret. |
| `extra` | `Extra` | Any other fields returned by the API. |

Create a version and set an environment variable on it:

```rust,no_run
use apify_client::models::ActorEnvVar;
use apify_client::ApifyClient;
use serde_json::json;

# async fn run(client: ApifyClient, actor_id: &str) -> Result<(), Box<dyn std::error::Error>> {
let actor_client = client.actor(actor_id);

// Create version 0.1 with inline source files.
let version = actor_client
    .versions()
    .create(&json!({
        "versionNumber": "0.1",
        "sourceType": "SOURCE_FILES",
        "sourceFiles": [
            { "name": "Dockerfile", "format": "TEXT",
              "content": "FROM apify/actor-node:20\nCOPY . ./\nCMD node main.js" },
            { "name": "main.js", "format": "TEXT", "content": "console.log('v0.1');" }
        ]
    }))
    .await?;
println!("created version {}", version.version_number);

// Set an environment variable on that version.
let version_client = actor_client.version(&version.version_number);
let env_var = version_client
    .env_vars()
    .create(&ActorEnvVar {
        name: "MY_VAR".to_string(),
        value: Some("hello".to_string()),
        is_secret: Some(false),
        extra: Default::default(),
    })
    .await?;
println!("set env var {} = {:?}", env_var.name, env_var.value);
# Ok(())
# }
```
