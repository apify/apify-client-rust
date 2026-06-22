# Apify Rust client — documentation

This directory documents the public API of the Apify Rust client. The same descriptions
are available as rustdoc comments and can be browsed with `cargo doc --open`.

## Contents

- [Getting started](#getting-started)
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
apify-client = "0.2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Some snippets and examples need extra crates: add `serde_json = "1"` when you read
dynamically-typed responses as `serde_json::Value` (as the README Quick start does), and
`futures-util = "0.3"` to consume `LogClient::stream()` (log streaming — see
[Store, users and logs](misc.md#logs--clientlogbuild_or_run_id)).

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

## Imports

The client, the builder, and every option/parameter struct (`ActorListOptions`, `ListOptions`,
`StorageListOptions`, `StoreListOptions`, `RunListOptions`, `ActorStartOptions`,
`DatasetListItemsOptions`, `DownloadItemsFormat`, `GetRecordOptions`, …), plus the common
container `PaginationList`, are re-exported at the crate root, so you can import them directly
from `apify_client`:

```rust,no_run
use apify_client::{ApifyClient, ActorListOptions, StoreListOptions, DownloadItemsFormat};
```

You do **not** need the longer `apify_client::clients::<module>::<Type>` paths shown by
`cargo doc`'s module tree for these option types — the short crate-root path is the supported
way to import them.

API resource/response **models** (`Actor`, `ActorRun`, `Build`, `Dataset`, `KeyValueStore`,
`RequestQueue`, `RequestQueueRequest`, `RequestQueueHead`, `RequestQueueOperationInfo`,
`KeyValueStoreKeysPage`, `ActorStoreListItem`, `User`, …) live in the [`apify_client::models`]
module and are imported from there:

```rust,no_run
use apify_client::models::RequestQueueRequest;
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
| `http_backend(b)` | reqwest | Replaceable HTTP transport (`HttpBackend`). |

The `User-Agent` header has the form
`ApifyClient/{client_version} ({os}; Rust/{rust_version}); isAtHome/{true|false}` where
`isAtHome` reflects the `APIFY_IS_AT_HOME` environment variable (the platform variable the
reference clients read; rendered lowercase to match them).

Accessor methods return resource clients (see the per-resource pages):

`actors`/`actor`, `builds`/`build`, `runs`/`run`, `tasks`/`task`, `datasets`/`dataset`,
`key_value_stores`/`key_value_store`, `request_queues`/`request_queue`,
`schedules`/`schedule`, `webhooks`/`webhook`, `webhook_dispatches`/`webhook_dispatch`,
`store`, `me`/`user`, `log`.

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
| `log_redirection` | Stream (redirect) an Actor run's log live. |
