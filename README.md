# Apify API client for Rust

The official, idiomatic Rust client for the [Apify API](https://docs.apify.com/api/v2).
It provides a resource-oriented, async interface that mirrors the official
[JavaScript](https://github.com/apify/apify-client-js) and
[Python](https://github.com/apify/apify-client-python) clients.

- Async (Tokio-friendly), built on `reqwest`.
- Transparent authentication, retries with exponential backoff, and timeouts.
- Resource clients for Actors, runs, builds, tasks, datasets, key-value stores, request
  queues, schedules, webhooks, the Apify Store, users and logs.
- Convenience helpers: run/wait, log streaming (redirection), lazy Store iteration.
- A replaceable HTTP transport for testing or custom runtimes.

## Installation

```toml
[dependencies]
apify-client = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

By default the client uses the system TLS (`native-tls`). To use rustls instead:

```toml
apify-client = { version = "0.1", default-features = false, features = ["rustls"] }
```

## Quick start

```rust,no_run
use apify_client::ApifyClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ApifyClient::new("my-api-token");

    // Start an Actor and wait for it to finish.
    let run = client
        .actor("apify/hello-world")
        .call::<serde_json::Value>(None, Default::default(), None)
        .await?;
    println!("Run finished with status {:?}", run.status);

    // Read items from the run's default dataset.
    if let Some(dataset_id) = &run.default_dataset_id {
        let items = client
            .dataset(dataset_id)
            .list_items::<serde_json::Value>(Default::default())
            .await?;
        println!("Got {} items", items.items.len());
    }
    Ok(())
}
```

## Configuration

Build a customized client with `ApifyClient::builder()`:

```rust,no_run
use std::time::Duration;
use apify_client::ApifyClient;

let client = ApifyClient::builder()
    .token("my-api-token")
    .base_url("https://api.apify.com")     // `/v2` is appended automatically
    .max_retries(8)                         // default: 8
    .min_delay_between_retries(Duration::from_millis(500)) // default: 500ms
    .timeout(Duration::from_secs(360))      // default: 360s
    .user_agent_suffix("my-app/1.0")
    .build();
```

## Resource clients

| Accessor | Returns | Purpose |
|---|---|---|
| `client.actors()` / `client.actor(id)` | collection / single | Manage Actors |
| `client.builds()` / `client.build(id)` | collection / single | Actor builds |
| `client.runs()` / `client.run(id)` | collection / single | Actor runs |
| `client.tasks()` / `client.task(id)` | collection / single | Actor tasks |
| `client.datasets()` / `client.dataset(id)` | collection / single | Datasets |
| `client.key_value_stores()` / `client.key_value_store(id)` | collection / single | Key-value stores |
| `client.request_queues()` / `client.request_queue(id)` | collection / single | Request queues |
| `client.schedules()` / `client.schedule(id)` | collection / single | Schedules |
| `client.webhooks()` / `client.webhook(id)` | collection / single | Webhooks |
| `client.webhook_dispatches()` / `client.webhook_dispatch(id)` | collection / single | Webhook dispatches |
| `client.store()` | collection | Browse the Apify Store |
| `client.me()` / `client.user(id)` | single | User accounts |
| `client.log(build_or_run_id)` | single | Build/run logs |

## Versioning

```rust
use apify_client::{CLIENT_VERSION, API_SPEC_VERSION};
println!("client {CLIENT_VERSION}, built against API spec {API_SPEC_VERSION}");
```

- `CLIENT_VERSION` — semantic version of this crate.
- `API_SPEC_VERSION` — the Apify OpenAPI spec version this client targets.

## Examples

Runnable examples live in [`examples/`](examples). Each requires `APIFY_TOKEN` and can be
run with `cargo run --example <name>`:

- `run_store_actor` — run a Store Actor, wait, read its default dataset.
- `storages` — create each storage type (dataset, key-value store, request queue), write and read back.
- `get_account` — fetch the current account's details.
- `create_build_run_actor` — create an Actor, build it, run it, fetch the run log.
- `run_and_last_run_storages` — run an Actor, then access the last run and its storages.
- `iterate_store` — lazily iterate Actors in the Apify Store.
- `log_redirection` — run an Actor with its log streamed (redirected) live.

See [`docs/`](docs) for the full API documentation.

## Scope

This client deliberately mirrors the public surface of the official JavaScript reference
client. A few documented API endpoints are intentionally **not** exposed because the
reference client does not expose them either, keeping the clients consistent:

- Synchronous run endpoints (`run-sync`, `run-sync-get-dataset-items`).
- Cryptographic tools (`/tools/encode-and-sign`, `/tools/decode-and-verify`).
- `/browser-info`.
- The `POST` create-with-explicit-key variants of Actor versions
  (`POST /v2/actors/{actorId}/versions/{versionNumber}`) and version env-vars
  (`POST /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName}`). The reference
  client creates versions/env-vars via `POST` to the collection and upserts via `PUT` on the
  keyed path (both supported here as `versions().create(...)` / `version(n).update(...)` and
  the env-var equivalents), so the redundant keyed-`POST` create is intentionally omitted.
- The `POST` store-record variant (`POST /v2/key-value-stores/{storeId}/records/{recordKey}`),
  which the spec defines as behaving identically to the covered `PUT` variant. Records are
  stored via `set_record_raw(...)` / `set_record_json(...)` (`PUT`), matching the reference
  client; the redundant keyed-`POST` alias is intentionally omitted.

If you need these, call them directly through a custom `HttpBackend` or open an issue.

## Error handling

All fallible methods return `Result<T, ApifyClientError>`. API errors expose the parsed
`type`, `message`, `status_code` and request details.

An Actor (or task/store) id may be a bare id, a `username/name` reference, or the equivalent
`username~name` form (the client encodes the first `/` as `~` on the wire, so the two are
interchangeable). The example below uses `"nonexistent~actor"` — a deliberately non-existent
reference — to show how a missing resource surfaces as `Ok(None)` rather than an error, while
real calls elsewhere in this README use a live actor such as `"apify/hello-world"`:

```rust,no_run
# use apify_client::ApifyClient;
# async fn run() {
# let client = ApifyClient::new("t");
match client.actor("nonexistent~actor").get().await {
    Ok(Some(actor)) => println!("found {}", actor.id),
    Ok(None) => println!("actor does not exist"),
    Err(err) => {
        if let Some(api) = err.as_api_error() {
            eprintln!("API error {}: {}", api.status_code, api.message);
        }
    }
}
# }
```

`get`/`delete` operations resolve a missing resource (`404 record-not-found`) to
`Ok(None)` / a successful no-op, matching the reference clients.

## Custom HTTP transport

The transport is a replaceable component. Implement `http_client::HttpBackend` and inject
it with `ApifyClientBuilder::http_backend`:

```rust,no_run
use std::sync::Arc;
use apify_client::ApifyClient;
use apify_client::http_client::ReqwestBackend;

// Share a pre-configured reqwest client (proxy, TLS, connection pool, ...).
let reqwest_client = reqwest::Client::builder().build().unwrap();
let backend = Arc::new(ReqwestBackend::with_client(reqwest_client));
let client = ApifyClient::builder().token("t").http_backend(backend).build();
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
