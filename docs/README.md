# Apify Rust client — documentation

> **Official, but experimental — AI-generated and AI-maintained.** This is an official Apify
> client, but it is experimental: it is generated and maintained by AI. Review the code before
> relying on it in production and report issues on the repository.

This directory documents the public API of the Apify Rust client. The same descriptions
are available as rustdoc comments and can be browsed with `cargo doc --open`.

> **Note:** the cross-file links between these pages (e.g. to `actors.md`, `runs.md`, or
> `../examples/`) are written for the repository (GitHub) view. Because the pages are
> concatenated onto the crate root via `include_str!` when building rustdoc, those relative
> links do not resolve in `cargo doc` output — read the cross-references on GitHub.

## Contents

- [Getting started](#getting-started)
- [Imports](#imports)
- [`ApifyClient` and the builder](#apifyclient-and-the-builder)
- [Resource clients](#resource-clients)
  - [Actors](actors.md)
  - [Actor runs](runs.md)
  - [Actor builds](builds.md)
  - [Actor tasks](tasks.md)
  - [Storages: datasets, key-value stores, request queues](storages.md)
  - [Schedules](schedules.md)
  - [Webhooks and dispatches](webhooks.md)
  - [Store, users and logs](misc.md)
- [Error handling](#error-handling)
- [Examples](#examples)

## Getting started

Add the crate and an async runtime:

```toml
[dependencies]
apify-client = "0.6"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Some snippets and examples need extra crates: add `serde_json = "1"` when you read
dynamically-typed responses as `serde_json::Value` (as the README Quick start does),
`futures-util = "0.3"` to consume `LogClient::stream()` (log streaming — see
[Store, users and logs](misc.md#logs--clientlogbuild_or_run_id)), and `chrono = "0.4"` if you
construct or read timestamp values yourself (model timestamp fields are `chrono::DateTime<Utc>`
and `chrono` is not re-exported, so snippets calling `chrono::Utc::now()` — such as the
`monthly_usage` and account examples — need it as a direct dependency).

Create a client and call a resource:

```rust,no_run
use apify_client::ApifyClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ApifyClient::new("my-api-token");
    let user = client.me().get().await?.expect("account");
    println!("Logged in as {:?}", user.username);
    Ok(())
}
```

`ApifyClient::new` takes the API token as an argument — the client does **not** read it from the
environment for you. By convention the token is kept in the `APIFY_TOKEN` environment variable
(this is what every example in [`examples/`](../examples) reads, and what the integration-test
suite and the Apify platform use), so production code typically does
`ApifyClient::new(std::env::var("APIFY_TOKEN")?)`. The literal `"my-api-token"` above is only a
placeholder for brevity.

## Imports

The client, the builder, and **every** option/parameter struct are re-exported at the crate
root, so you can import them directly from `apify_client` — you never need the longer
`apify_client::clients::<module>::<Type>` path. The complete set of re-exported option/parameter
types is:

- Actors: `ActorStartOptions`, `ActorBuildOptions`, `ActorListOptions`
- Runs: `RunListOptions`, `RunResurrectOptions`, `RunMetamorphOptions`, `RunChargeOptions`, `LastRunOptions`
- Datasets: `DatasetListItemsOptions`, `DatasetDownloadOptions`, `DownloadItemsFormat`
- Key-value stores: `ListKeysOptions`, `GetRecordsOptions`, `GetRecordOptions`
- Request queues: `ListRequestsOptions`
- Store: `StoreListOptions`
- Logs: `LogOptions`
- Shared: `ListOptions`, `StorageListOptions`
- Client configuration: `RequestCompression`

plus the common container `PaginationList` and the query helper `QueryParams`. Import any of them
directly from `apify_client`:

```rust,no_run
use apify_client::{ApifyClient, ActorListOptions, StoreListOptions, DownloadItemsFormat};
```

(The `apify_client::clients::<module>::<Type>` paths shown by `cargo doc`'s module tree also work,
but the short crate-root path above is the supported, stable way to import these option types.)

API resource/response **models** (`Actor`, `ActorRun`, `Build`, `Dataset`, `KeyValueStore`,
`KeyValueStoreKey`, `KeyValueStoreKeysPage`, `KeyValueStoreRecord`, `RequestQueue`,
`RequestQueueRequest`, `RequestQueueHead`, `RequestQueueOperationInfo`, `Task`, `Schedule`,
`Webhook`, `WebhookDispatch`, `ActorStoreListItem`, `User`, …) all live in the
`apify_client::models` module and are imported from there:

```rust,no_run
use apify_client::models::RequestQueueRequest;
```

Every model exposes an `extra: Extra` field that captures any response fields not otherwise
modelled; `Extra` is a type alias for `std::collections::HashMap<String, serde_json::Value>`, also
in `apify_client::models`. When constructing a model, set `extra: Default::default()` (an empty
map); read passthrough fields from a returned model's `extra` by key:

```rust,no_run
use apify_client::models::Extra;

// `extra` is a HashMap<String, serde_json::Value>; look up any passthrough field by key.
fn some_field(extra: &Extra) -> Option<&serde_json::Value> {
    extra.get("someFieldNotModelled")
}
```

The error type `ApifyClientError` (and its companions `ApiError`, `ApifyClientResult`) is
re-exported at the crate root — see [Error handling](#error-handling) for import and matching:

```rust,no_run
use apify_client::ApifyClientError;
```

## `ApifyClient` and the builder

`ApifyClient` is the entry point. Construct it with `ApifyClient::new(token)` for the common
case, or with `ApifyClient::builder()` to customize behaviour.

Builder options:

| Method | Default | Description |
|---|---|---|
| `token(t)` | none | API token, sent as `Authorization: Bearer <token>`. |
| `base_url(u)` | `https://api.apify.com` | API origin; `/v2` is appended automatically. |
| `public_base_url(u)` | = `base_url` | Origin used when building public/shareable URLs. |
| `max_retries(n)` | `8` | Maximum retries for `429`/`5xx`/network errors. |
| `min_delay_between_retries(d)` | `500ms` | Base backoff delay (doubled each retry, with jitter). |
| `timeout(d)` | `360s` | Overall per-request timeout budget. |
| `user_agent_suffix(s)` | none | Extra text appended to the `User-Agent` header. |
| `request_compression(c)` | `Brotli` | Encoding for large request bodies: `RequestCompression::Brotli` (`br`) or `RequestCompression::Gzip` (`gzip`). |
| `http_backend(b)` | reqwest | Replaceable HTTP transport (`apify_client::http_client::{HttpBackend, ReqwestBackend}`). |

Request bodies of at least 1024 bytes are compressed once (before retries) with the selected
`request_compression` algorithm and sent with the matching `Content-Encoding` header. Brotli is
preferred; select gzip for environments or intermediaries that do not handle brotli:

```rust,no_run
use apify_client::{ApifyClient, RequestCompression};

let client = ApifyClient::builder()
    .token("my-api-token")
    .request_compression(RequestCompression::Gzip)
    .build();
```

The `User-Agent` header has the form
`ApifyClient/{client_version} ({os}; Rust/{rust_version}); isAtHome/{true|false}` where the `{os}`
token matches the reference client's `os.platform()` value (e.g. `darwin`, `win32`, `linux`) so it
is identical across all Apify clients, and `isAtHome` reflects the `APIFY_IS_AT_HOME` environment
variable (the platform variable the reference clients read; rendered lowercase to match them). To
reach those values the client maps Rust's native `std::env::consts::OS` spellings to Node's
(`macos` → `darwin`, `windows` → `win32`, `solaris`/`illumos` → `sunos`); all other tokens
(`linux`, `android`, `freebsd`, …) are already identical and pass through unchanged.

## Resource clients

Accessor methods on `ApifyClient` return resource clients — the collection accessor (plural)
lists/creates, and the single-resource accessor (singular) operates on one resource by id:

`actors`/`actor`, `builds`/`build`, `runs`/`run`, `tasks`/`task`, `datasets`/`dataset`,
`key_value_stores`/`key_value_store`, `request_queues`/`request_queue`,
`schedules`/`schedule`, `webhooks`/`webhook`, `webhook_dispatches`/`webhook_dispatch`,
`store`, `me`/`user`, `log`.

Each resource has a dedicated page, linked under **Resource clients** in the [Contents](#contents)
above (Actors, runs, builds, tasks, storages, schedules, webhooks, and store/users/logs).

### Iterating collections

A collection's `list(...)` method returns a single `PaginationList` page. To walk every item
across all pages without tracking offsets yourself, call `iterate(...)` instead: it returns a
lazy `ListIterator` (re-exported at the crate root) that fetches the next page from the API on
demand as you consume items. Every collection client provides it (`actors`, `builds`, `runs`,
`tasks`, `datasets`, `key_value_stores`, `request_queues`, `schedules`, `webhooks`,
`webhook_dispatches`, `store`, and the nested Actor `versions`/`env_vars`), and `DatasetClient`
exposes `iterate_items()` for dataset items. The options' `limit` caps the total number of items
yielded (unset iterates everything); to control the per-request page size, call
`.with_chunk_size(n)` on the returned iterator.

```rust,no_run
use apify_client::{ApifyClient, ActorListOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ApifyClient::new("my-api-token");
    let mut actors = client.actors().iterate(ActorListOptions {
        my: Some(true),
        ..Default::default()
    });
    while let Some(actor) = actors.next().await? {
        println!("{}", actor.id);
    }
    Ok(())
}
```

## Error handling

Every fallible method returns `Result<T, ApifyClientError>`. The variants are:

- `Api(Box<ApiError>)` — the API returned a non-success status. `ApiError` exposes
  `status_code`, `error_type`, `message`, `attempt`, `http_method`, `path` and optional
  `data`.
- `Http(String)` — transport/network failure.
- `Timeout` — the request exceeded its timeout.
- `Serde(..)` — (de)serialization failure.
- `InvalidResponse(..)` / `InvalidArgument(..)` — unexpected response or bad argument.

`get`/`delete` map a missing resource to `Ok(None)` / a no-op.

To inspect the API-level details of an error without matching every variant, use
`ApifyClientError::as_api_error`, which returns `Some(&ApiError)` for the `Api` variant and
`None` for any other (transport, timeout, serde, …):

```rust,no_run
# use apify_client::ApifyClient;
# async fn run() {
# let client = ApifyClient::new("t");
if let Err(err) = client.actor("nonexistent~actor").get().await {
    if let Some(api) = err.as_api_error() {
        eprintln!("API error {}: {}", api.status_code, api.message);
    }
}
# }
```

## Examples

Each example in [`../examples`](../examples) is runnable with
`APIFY_TOKEN=... cargo run --example <name>` and is covered by a CI test:

| Example | Demonstrates |
|---|---|
| `run_store_actor` | Run a Store Actor, wait, read its default dataset. |
| `storages` | Create + write + read each storage type. |
| `get_account` | Fetch the current account, plus its monthly usage (current cycle and for a specific date). |
| `create_build_run_actor` | Create an Actor, build, run, fetch the run log. |
| `run_and_last_run_storages` | Run an Actor, then read the last run's storages. |
| `iterate_store` | Lazily iterate Store Actors. |
| `log_redirection` | Redirect a separate Actor's run log into your output live, prefixing each line with the source Actor's name. |
| `raw_log` | Fetch and stream a run's raw (unprocessed) log via `LogOptions { raw: Some(true) }`. |
