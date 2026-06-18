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
| `monthly_usage()` | — | `Value` | Current account's monthly usage (`me` only). |
| `limits()` | — | `Value` | Current account's limits (`me` only). |
| `update_limits(limits)` | `&impl Serialize` | `()` | Updates the account's limits (`me` only). |

## Logs — `client.log(build_or_run_id)`

Also reachable via `run.log()` and `build.log()`.

`LogClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<String>` | The entire log as text. |
| `stream()` | — | `Stream<Item = Result<Vec<u8>>>` | Streams log chunks live (log redirection). |
