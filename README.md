# Apify API client for Rust

> **Official, but experimental — AI-generated and AI-maintained.** This is an official Apify
> client, but it is experimental: it is generated and maintained by AI. Review the code before
> relying on it in production and report issues on the repository.

An idiomatic Rust client for the [Apify API](https://docs.apify.com/api/v2).
It provides a resource-oriented, async interface that mirrors the official
[JavaScript](https://github.com/apify/apify-client-js) and
[Python](https://github.com/apify/apify-client-python) clients.

- Async (Tokio-friendly), built on `reqwest`.
- Transparent authentication, retries with exponential backoff, and timeouts.
- Resource clients for Actors, runs, builds, tasks, datasets, key-value stores, request
  queues, schedules, webhooks, the Apify Store, users and logs.
- Convenience helpers: run/wait, log streaming (redirection; needs the `futures-util` crate —
  see [Installation](#installation)), lazy Store iteration.
- A replaceable HTTP transport for testing or custom runtimes.

## Installation

```toml
[dependencies]
apify-client = "0.5"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"          # for the `serde_json::Value` responses used in the Quick start
```

The Quick start below reads dynamically-typed records with `serde_json::Value`, so a fresh
project needs `serde_json`. Two more dependencies are needed only for specific features:

- `futures-util = "0.3"` — to consume any of the log streams (`LogClient::stream()` /
  `stream_with_options()` and the run shortcuts `RunClient::get_streamed_log()` /
  `get_streamed_log_with_options()`); it provides the `StreamExt` trait used to poll the returned
  stream. Both the [`log_redirection`](examples/log_redirection.rs) and
  [`raw_log`](examples/raw_log.rs) examples import `futures_util::StreamExt` for this. See
  [`docs/misc.md`](docs/misc.md#logs--clientlogbuild_or_run_id).
- `chrono = "0.4"` — only if you construct or read timestamp values yourself. Model timestamp
  fields (e.g. `Actor::created_at`, `ActorRun::started_at`) are typed as `chrono::DateTime<Utc>`
  and `chrono` is **not** re-exported, so snippets that call `chrono::Utc::now()` (e.g. the
  `monthly_usage` example in [`docs/misc.md`](docs/misc.md#users--clientme--clientuserid) and the
  account example) need it as a direct dependency.
- `reqwest = "0.12"` — only if you build a custom HTTP transport by constructing a
  `reqwest::Client` yourself and passing it to `ReqwestBackend::with_client(...)` (see
  [Custom HTTP transport](#custom-http-transport)). `reqwest` is **not** re-exported, so that
  snippet needs it as a direct dependency, and its version must match this crate's `reqwest`
  version (`0.12`) so the `reqwest::Client` types are compatible.

By default the client uses the system TLS (`native-tls`). To use rustls instead:

```toml
apify-client = { version = "0.5", default-features = false, features = ["rustls"] }
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

The type parameter on `call::<T>(...)` is the Actor's input body type (`T: Serialize`). The
turbofish is only needed here because the input is `None`, which is otherwise untyped; when you
pass a real payload (e.g. `Some(&my_input)`) the compiler infers `T` and the turbofish can be
dropped.

## Configuration

Build a customized client with `ApifyClient::builder()`:

```rust,no_run
use std::time::Duration;
use apify_client::ApifyClient;

let client = ApifyClient::builder()
    .token("my-api-token")
    .base_url("https://api.apify.com")     // `/v2` is appended automatically
    .public_base_url("https://api.apify.com") // origin for public/shareable URLs; defaults to `base_url`
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

### Releasing

The crate is distributed on [crates.io](https://crates.io/crates/apify-client). The
[`Publish Rust client to crates.io`](.github/workflows/rust-publish.yml) workflow is the release
mechanism: a maintainer triggers it manually (`workflow_dispatch`), it runs the same
fmt/clippy/build quality gate as CI, verifies packaging with `cargo publish --dry-run`, then tags
the released commit (`vX.Y.Z`, derived from the `version` in `Cargo.toml`), creates a matching
GitHub release whose notes are the corresponding `CHANGELOG.md` section (falling back to a generated
one-liner if that section is missing), and finally runs `cargo publish`.

The workflow **only runs from `master`** — it hard-fails on any other ref — and refuses to run if
the resolved tag already exists, so a release can never clobber a prior one. A `dry_run` input runs
all checks but performs no publish, tag, or release.

Authentication to crates.io uses **Trusted Publishing** (OIDC): the job has `id-token: write`
permission and `rust-lang/crates-io-auth-action@v1` exchanges the GitHub Actions identity for a
short-lived crates.io token at publish time (auto-revoked when the job ends). No long-lived
crates.io API token is stored as a repository secret; the only publishing secret used is the
built-in `GITHUB_TOKEN` (for the tag push and GitHub release).

Prerequisites and steps to cut a release:

1. Configure a **Trusted Publisher** for the `apify-client` crate on crates.io (one-time setup),
   pointing at this repository and the `rust-publish.yml` workflow. No `CARGO_REGISTRY_TOKEN` secret
   is needed; the tag and GitHub release use the default `GITHUB_TOKEN`.
2. Bump `version` in `Cargo.toml` and add a matching dated entry to `CHANGELOG.md` (the release
   notes are extracted from that section), then merge to `master`.
3. Trigger the workflow from `master`.

The tag is pushed and the GitHub release created before `cargo publish`, because the crates.io
publish is the only truly unrepeatable step — failing before it leaves the tag and release
consistent with the crate version. The GitHub-release step is idempotent (on a re-run it updates an
existing release rather than failing), so it never needs manual cleanup.

**Recovering from a failed release.** If the run fails *after* the tag was pushed but *before*
`cargo publish` succeeded (e.g. a transient registry error), the tag now exists, so a plain re-run
is blocked by the "tag already exists" guard. The one thing that unblocks the re-run is **deleting
the tag** — the existing GitHub release does not need deleting (the idempotent release step will
update it on the next run). Delete the remote tag and re-trigger the workflow (replace `vX.Y.Z`
with the actual release version, e.g. `v0.2.3`):

```bash
# Replace vX.Y.Z with the real version, e.g. v0.2.3.
git push origin :refs/tags/vX.Y.Z   # delete the remote tag — this is what clears the guard
```

`git push origin :refs/tags/vX.Y.Z` deletes only the tag, which is all that is required. If you
also want to remove the GitHub release (optional, cosmetic, and independent of the required tag
deletion), use `gh release delete vX.Y.Z --yes` — without `--cleanup-tag` it removes only the
release and leaves the tag handling to the command above. (Alternatively, `gh release delete
vX.Y.Z --cleanup-tag --yes` is an all-in-one that deletes the release *and* the tag in a single
step, replacing the `git push origin :refs/tags/...` command above rather than adding to it.)

If `cargo publish` itself already succeeded, that version is permanent on crates.io; bump the
`version` in `Cargo.toml` for the next release instead of re-running.

## Examples

Runnable examples live in [`examples/`](examples). Each requires `APIFY_TOKEN` and can be
run with `cargo run --example <name>`:

- `run_store_actor` — run a Store Actor, wait, read its default dataset.
- `storages` — create each storage type (dataset, key-value store, request queue), write and read back.
- `get_account` — fetch the current account's details and monthly usage (current cycle and for a specific date via `monthly_usage_for_date`).
- `create_build_run_actor` — create an Actor, build it, run it, fetch the run log.
- `run_and_last_run_storages` — run an Actor, then access the last run and its storages.
- `iterate_store` — lazily iterate Actors in the Apify Store.
- `log_redirection` — run a separate Actor and redirect its run log into your output live, with each line prefixed by the source Actor's name.
- `raw_log` — fetch and stream a run's raw (unprocessed) log via `LogOptions { raw: Some(true) }`.

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
error `type` (as the field `error_type`), `message`, `status_code` and request details.

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

The transport is a replaceable component: any type implementing `http_client::HttpBackend` can be
injected with `ApifyClientBuilder::http_backend`. The most common customization is to reuse the
built-in `ReqwestBackend` with a `reqwest::Client` you pre-configure (custom proxy, TLS, connection
pool, …), as shown below; to route requests through an entirely different HTTP stack, implement
`HttpBackend` (a single `async fn send`) on your own type instead. The snippet below constructs a
`reqwest::Client` directly, so it requires adding `reqwest = "0.12"` (matching this crate's
`reqwest` version) as your own dependency — see [Installation](#installation):

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
