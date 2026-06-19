# Store, users and logs

## Apify Store — `client.store()`

`StoreCollectionClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `StoreListOptions` | `PaginationList<ActorStoreListItem>` | One page of Store Actors. |
| `iterate(options)` | `StoreListOptions` | `StoreActorIterator` | Lazy, page-fetching iterator. |

`StoreListOptions`: `offset`, `limit`, `search`, `sort_by`, `category`, `username`,
`pricing_model`.

`StoreActorIterator::next()` returns `Option<ActorStoreListItem>`, fetching the next page
on demand until the listing is exhausted.

## Users — `client.me()` / `client.user(id)`

`UserClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<User>` | Account details (private for `me`, public otherwise). |
| `monthly_usage()` | — | `Value` | Current account's monthly usage for the current month (`me` only). |
| `monthly_usage_for_date(date)` | `Option<&str>` | `Value` | Monthly usage for the billing cycle containing the `YYYY-MM-DD` `date`; `None` == current month (`me` only). |
| `limits()` | — | `Value` | Current account's limits (`me` only). |
| `update_limits(limits)` | `&impl Serialize` | `()` | Updates the account's limits (`me` only). |

The methods marked **(`me` only)** operate on the authenticated account and are only valid on the
`client.me()` client. Calling any of them on a specific-user client (`client.user(id)`) returns
`Err(ApifyClientError::InvalidArgument(..))` without making a network request; `get()` is the only
method that works for both `me` and other users.

`monthly_usage()` is shorthand for `monthly_usage_for_date(None)` (current cycle). The client
unwraps the API's `{ data: ... }` envelope, so the returned `Value` has the shape
`{ usageCycle: { startAt, endAt }, monthlyServiceUsage, dailyServiceUsages, ... }`. Billing
cycles are not calendar-month aligned — pass any day within a cycle to fetch that cycle:

```rust,no_run
use apify_client::ApifyClient;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = ApifyClient::new(std::env::var("APIFY_TOKEN")?);

// Current cycle.
let usage = client.me().monthly_usage().await?;

// The cycle containing a specific day (YYYY-MM-DD).
let march = client.me().monthly_usage_for_date(Some("2026-03-15")).await?;
let cycle = &march["usageCycle"];
println!("cycle {} .. {}", cycle["startAt"], cycle["endAt"]);
# Ok(())
# }
```

## Logs — `client.log(build_or_run_id)`

Also reachable via `run.log()` and `build.log()`.

`LogClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<String>` | The entire log as text. |
| `stream()` | — | `Stream<Item = Result<Vec<u8>>>` | Streams log chunks live (log redirection). |
