# Apify Rust client — documentation

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
apify-client = "0.15"
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
    println!("Logged in as {}", user.username.as_deref().unwrap_or("(none)"));
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

- Actors: `ActorStartOptions`, `ActorCallOptions`, `TaskStartOptions`, `ActorBuildOptions`, `ActorListOptions`
- Runs: `RunListOptions`, `RunResurrectOptions`, `RunMetamorphOptions`, `RunChargeOptions`, `LastRunOptions`
- Datasets: `DatasetListItemsOptions`, `DatasetDownloadOptions`, `DownloadItemsFormat`
- Key-value stores: `ListKeysOptions`, `GetRecordOptions`
- Request queues: `ListRequestsOptions`
- Store: `StoreListOptions`
- Logs: `LogOptions`, `LogRedirect`, `LogSink`, `StreamedLog`, `StreamedLogOptions`
- Shared: `ListOptions`, `StorageListOptions`
- Client configuration: `RequestCompression`

plus the common container `PaginationList`, `ListIterator`
(the return type of every collection client's `iterate()` method), `StoreActorIterator`
(a type alias for `ListIterator<ActorStoreListItem>`, the return type of
`StoreCollectionClient::iterate`), and the two cursor-based iterators `KeyValueStoreKeysIterator`
(from `KeyValueStoreClient::iterate_keys`) and `RequestQueueRequestsIterator` (from
`RequestQueueClient::paginate_requests`). Import any of them directly from `apify_client`:

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
| `timeout_short(d)` | `5s` | Duration of the `Timeout::Short` tier (simple metadata reads and writes). Like the other timeout options, a zero duration makes every request of the client fail with `InvalidArgument`. |
| `timeout_medium(d)` | `30s` | Duration of the `Timeout::Medium` tier (listing, batch and trigger calls). |
| `timeout_long(d)` | `360s` | Duration of the `Timeout::Long` tier (downloads, uploads and streaming). |
| `timeout_max(d)` | `360s` | Cap on the timeout of a single attempt. `timeout(d)` is the same option under its original name. |
| `user_agent_suffix(s)` | none | Extra text appended to the `User-Agent` header. |
| `default_header(name, value)` | none | Header sent with every request. It wins over the built-in `User-Agent` and `Authorization` headers; a header a request sets itself wins over it. Names are compared case-insensitively. |
| `request_compression(c)` | `Brotli` | Encoding for large request bodies: `RequestCompression::Brotli` (`br`) or `RequestCompression::Gzip` (`gzip`). |
| `http_compressor(c)` | Brotli, quality 6 | A custom `Arc<dyn HttpCompressor>`: `BrotliHttpCompressor::new(quality)` (0 to 11, default 6), `GzipHttpCompressor::new(quality)` (1 to 9, default 6), or your own implementation. |
| `http_backend(b)` | reqwest | Replaceable HTTP transport (`apify_client::http_client::{HttpBackend, ReqwestBackend}`). A request whose `timeout` is `timeouts::NO_TIMEOUT` may run for as long as it takes, so a backend must check for it before it applies a timeout of its own. |

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

### Timeouts

Every method uses a timeout tier: `Timeout::Short` for simple metadata reads and writes,
`Timeout::Medium` for listing, batch and trigger calls, and `Timeout::Long` for downloads, uploads
and streaming. `wait_for_finish` polls without a client-side timeout. A tier or an explicit
timeout above `timeout_max` is lowered to it, and the timeout of each attempt doubles with every
retry, up to the same cap.

Every resource client has a `with_timeout(Timeout)` method that replaces the tier of all the
requests it sends with a tier, `Timeout::Custom(duration)`, or `Timeout::NoTimeout` (the client
never aborts the request, so a stalled connection is not retried either). A zero duration is
rejected: every request then fails with `InvalidArgument`, and so does every request of a client
built with a zero `timeout_short`, `timeout_medium`, `timeout_long` or `timeout_max`. Clients
obtained from it, such as
`run.dataset()`, do not inherit the timeout. A request queue client also has
`with_timeout_cap(duration)`, which only lowers the tiers longer than `duration`.

`dataset.push_items` is not retried after a transport timeout, because the request may have
reached the API and a retry could store the items twice. The same goes for the key-value store
`set_record_*_with_options` methods when `do_not_retry_timeouts` is set.

`client.stats()` returns the `Statistics` of the client and of every resource client obtained from
it: `calls()` (each call counts once), `requests()` (every attempt counts) and `rate_limit_errors()`
(the `429` responses, indexed by attempt).

```rust,no_run
use std::time::Duration;
use apify_client::{ApifyClient, DownloadItemsFormat, Timeout};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = ApifyClient::builder()
    .token("my-api-token")
    .timeout_short(Duration::from_secs(10))
    .build();

// This export may take longer than the default `Long` tier of 6 minutes.
let csv = client
    .dataset("my-dataset-id")
    .with_timeout(Timeout::NoTimeout)
    .download_items(DownloadItemsFormat::Csv, Default::default())
    .await?;
# let _ = csv;
# Ok(())
# }
```

### Convenience methods

Beyond the resource accessors, `ApifyClient` exposes one convenience method:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `set_status_message(message, is_terminal)` | `message: &str`, `is_terminal: bool` | `ActorRun` | Sets the status message of the *current* Actor run. |

`set_status_message` updates the run identified by the `ACTOR_RUN_ID` environment variable, so it
only works when called from inside an Actor run. `message` is the human-readable status text; when
`is_terminal` is `true` the message becomes final and is not overwritten by later updates. It
returns the updated [`ActorRun`](runs.md), or
[`ApifyClientError::InvalidArgument`](#error-handling) if `ACTOR_RUN_ID` is not set.

```rust,no_run
use apify_client::ApifyClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ApifyClient::new("my-api-token");
    // Called from inside an Actor run (reads ACTOR_RUN_ID from the environment).
    client.set_status_message("Processing input…", false).await?;
    Ok(())
}
```

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
`webhook_dispatches`, `store`, and the nested Actor `versions`/`env_vars`). `DatasetClient`
exposes `iterate_items()` for dataset items, and `KeyValueStoreClient` exposes `iterate_keys()`
for store keys (cursor-based). The options' `limit` caps the total number of items yielded (unset
iterates everything); to control the per-request page size, call `.with_chunk_size(n)` on the
returned iterator (offset-paginated iterators only). An offset-paginated iterator ignores the
`total` a page reports and keeps requesting pages until one scans no rows, so a listing that grows
while being iterated (e.g. a dataset a still-running Actor keeps pushing to) is read to the end
rather than cut off at the size it had when the loop started.

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

`get`/`delete` map a missing resource (any `404`) to `Ok(None)` / a no-op, on a client that names
its resource by id. A client chained from a parent, such as `run.dataset()` or `run.log()`, and the
methods that address a fixed sub-path of a resource (`get_statistics`, `get_input`, `get_log`,
`get_openapi_definition`, `limits`, `monthly_usage*`, webhook `test`) return the `404` as an error,
because the response cannot tell which resource is missing.

An id that is empty or a dot segment (`.` or `..`) cannot be placed in a URL path, so every request
through a client created with it fails with `InvalidArgument`.

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
| `raw_log` | Fetch and stream a run's raw (unprocessed) log via `LogOptions { raw: Some(true), ..Default::default() }`. |
