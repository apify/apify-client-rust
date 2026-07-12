# Store, users and logs

## Apify Store — `client.store()`

`StoreCollectionClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `StoreListOptions` | `PaginationList<ActorStoreListItem>` | One page of Store Actors. |
| `iterate(options)` | `StoreListOptions` | `StoreActorIterator` | Lazy, page-fetching iterator. |

`StoreListOptions`: `offset`, `limit`, `search`, `sort_by`, `category`, `username`,
`pricing_model`. `limit` means a single page's size for `list`, but a cap on the *total* number of
items yielded for `iterate` (see below).

`StoreActorIterator` is a type alias for `ListIterator<ActorStoreListItem>` (the shared iterator
returned by every collection's `iterate`), re-exported at the crate root alongside `ListIterator`
itself. Its `next()` is `async` and fallible — it returns
`ApifyClientResult<Option<ActorStoreListItem>>` (i.e. `Result<Option<ActorStoreListItem>, ApifyClientError>`),
fetching the next page on demand and yielding `Ok(None)` once the listing is exhausted. Drive it
with `.await?`:

```rust,no_run
# use apify_client::{ApifyClient, StoreListOptions};
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let mut iter = client.store().iterate(StoreListOptions::default());
while let Some(actor) = iter.next().await? {
    // `title` is the human-readable name; fall back to the technical `name`.
    println!("{}: {:?}", actor.id, actor.title.or(actor.name));
}
# Ok(())
# }
```

`options.limit` caps the total number of Actors the iterator yields (unset iterates the whole
Store). The per-request page size is separate: call `.with_chunk_size(n)` on the returned
`StoreActorIterator` to fetch `n` Actors per API call (when unset, the API's default page size is
used). If you set a large `limit` cap, also set `with_chunk_size` so the first request does not ask
for the entire cap at once — for example, `client.store().iterate(opts).with_chunk_size(50)`:

```rust,no_run
# use apify_client::{ApifyClient, StoreListOptions};
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let mut iter = client.store().iterate(StoreListOptions::default()).with_chunk_size(50);
while let Some(actor) = iter.next().await? {
    println!("{}", actor.id);
}
# Ok(())
# }
```

`ActorStoreListItem` (from `apify_client::models`) is the element type yielded by both `list`
and the iterator. Its fields:

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique Actor ID (always present). |
| `name` | `Option<String>` | Technical name of the Actor. |
| `username` | `Option<String>` | Username of the Actor's owner. |
| `title` | `Option<String>` | Human-readable title. |
| `extra` | `Extra` | Any other fields returned by the API. |

`name`, `username` and `title` are optional, so a display routine typically prefers `title`
and falls back to `name` (e.g. `actor.title.or(actor.name)`).

## Users — `client.me()` / `client.user(id)`

`UserClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<User>` | Account details (private for `me`, public otherwise). |
| `monthly_usage()` | — | `Value` | Current account's monthly usage for the current billing cycle (`me` only). |
| `monthly_usage_for_date(date)` | `Option<&str>` | `Value` | Monthly usage for the billing cycle containing the `YYYY-MM-DD` `date`; `None` == current month (`me` only). |
| `limits()` | — | `Value` | Current account's limits (`me` only). |
| `update_limits(limits)` | `&impl Serialize` | `()` | Updates the account's limits (`me` only). |

The methods marked **(`me` only)** operate on the authenticated account and are only valid on the
`client.me()` client. Calling any of them on a specific-user client (`client.user(id)`) returns
`Err(ApifyClientError::InvalidArgument(..))` without making a network request; `get()` is the only
method that works for both `me` and other users.

`get()` returns a `User` (from `apify_client::models`). Its fields — including the `user.id` and
`user.username` the [`get_account`](../examples/get_account.rs) example reads:

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique user ID (always present). |
| `username` | `Option<String>` | Username. |
| `extra` | `Extra` | Any other fields returned by the API (more fields are present for `me` than for a public `user(id)`). |

```rust,no_run
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
if let Some(user) = client.me().get().await? {
    println!("account id {}, username {:?}", user.id, user.username);
}
# Ok(())
# }
```

`monthly_usage()` is shorthand for `monthly_usage_for_date(None)` (current cycle). The client
unwraps the API's `{ data: ... }` envelope, so the returned `serde_json::Value` has the shape
`{ usageCycle: { startAt, endAt }, monthlyServiceUsage, dailyServiceUsages, ... }`. Billing
cycles are not calendar-month aligned — pass any day within a cycle to fetch that cycle.

The return value is an untyped `serde_json::Value`; access its fields with the non-panicking
`Value::get` (the same idiom as `examples/get_account.rs`) so a missing field yields `None`
instead of panicking:

```rust,no_run
use apify_client::ApifyClient;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = ApifyClient::new(std::env::var("APIFY_TOKEN")?);

// Current cycle.
let usage = client.me().monthly_usage().await?;

// The cycle containing a specific day (YYYY-MM-DD). Derive it from "now" rather than
// hard-coding a date so the lookup always lands on a real cycle.
let day = chrono::Utc::now().format("%Y-%m-%d").to_string();
let dated = client.me().monthly_usage_for_date(Some(&day)).await?;
if let Some(cycle) = dated.get("usageCycle") {
    let start = cycle.get("startAt").and_then(|v| v.as_str()).unwrap_or("?");
    let end = cycle.get("endAt").and_then(|v| v.as_str()).unwrap_or("?");
    println!("cycle {start} .. {end}");
}
# Ok(())
# }
```

## Logs — `client.log(build_or_run_id)`

Also reachable via `run.log()` and `build.log()`.

`LogClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<String>` | The entire log as text. |
| `get_with_options(options)` | `LogOptions` | `Option<String>` | As `get()`, with options (e.g. `raw`). |
| `stream()` | — | `Result<impl Stream<Item = Result<Vec<u8>>>>` (async — `.await` it) | Streams log chunks live (log redirection). |
| `stream_with_options(options)` | `LogOptions` | `Result<impl Stream<Item = Result<Vec<u8>>>>` (async — `.await` it) | As `stream()`, with options (e.g. `raw`). |

`LogOptions` has a single field, `raw: Option<bool>`. When `Some(true)`, the API returns the
raw log content without server-side processing (e.g. without the per-line timestamps it adds by
default); leaving it `None` uses the default processed format. Fetch a run's raw log as text:

```rust,no_run
use apify_client::{ApifyClient, LogOptions};

# async fn run(client: ApifyClient, run_id: &str) -> Result<(), Box<dyn std::error::Error>> {
let raw_log = client
    .run(run_id)
    .log()
    .get_with_options(LogOptions { raw: Some(true) })
    .await?;
if let Some(text) = raw_log {
    print!("{text}");
}
# Ok(())
# }
```

The streaming variant takes the same options — `client.run(run_id).log().stream_with_options(LogOptions { raw: Some(true) }).await?`
yields the raw log chunks (this is what log redirection uses).

Consuming `stream()` requires the `futures_util::StreamExt` trait (from the `futures-util`
crate) in scope to call `.next()` on the returned stream. Add it to your `Cargo.toml`:

```toml
[dependencies]
futures-util = "0.3"
```

Then redirect a run's log to stdout as it is produced:

```rust,no_run
use apify_client::ApifyClient;
use futures_util::StreamExt;

# async fn run(client: ApifyClient, run_id: &str) -> Result<(), Box<dyn std::error::Error>> {
let mut stream = client.run(run_id).log().stream().await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    print!("{}", String::from_utf8_lossy(&chunk));
}
# Ok(())
# }
```
