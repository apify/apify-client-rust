# Storages: datasets, key-value stores, request queues

> Code blocks below use the rustdoc `# `-hidden-line convention — see
> [`docs/README.md`](README.md) for what those lines are.

## Storage metadata models (`Dataset`, `KeyValueStore`, `RequestQueue`)

`get` and `get_or_create` on each storage collection/client return a metadata model from
`apify_client::models` (`Dataset`, `KeyValueStore`, `RequestQueue`). All three share a common
core; the `.id` field is what the examples read to build a per-storage client
(`client.dataset(&dataset.id)`, `client.key_value_store(&store.id)`,
`client.request_queue(&queue.id)`):

| Field | Type | On | Description |
|---|---|---|---|
| `id` | `String` | all three | Unique storage ID (always present); pass to `client.dataset(..)` / `client.key_value_store(..)` / `client.request_queue(..)`. |
| `name` | `Option<String>` | all three | Technical name, if the storage is named. |
| `user_id` | `Option<String>` | all three | ID of the owner. |
| `created_at` | `Option<DateTime<Utc>>` | all three | When the storage was created. |
| `modified_at` | `Option<DateTime<Utc>>` | all three | When the storage was last modified. |
| `item_count` | `Option<i64>` | `Dataset` only | Total number of items in the dataset. |
| `total_request_count` | `Option<i64>` | `RequestQueue` only | Total number of requests ever added. |
| `extra` | `Extra` | all three | Any other fields returned by the API. |

```rust,no_run
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let dataset = client.datasets().get_or_create(None).await?;
// Use the metadata `id` to obtain a client for the storage itself.
let dataset_client = client.dataset(&dataset.id);
# let _ = dataset_client;
# Ok(())
# }
```

## Datasets — `client.datasets()` / `client.dataset(id)`

`DatasetCollectionClient`: `list(options: StorageListOptions)`,
`iterate(options: StorageListOptions)` (lazy `ListIterator<Dataset>` auto-pagination),
`get_or_create(name: Option<&str>)`.
`StorageListOptions`: `offset`, `limit`, `desc`, `unnamed`, `ownership`.

`get_or_create` takes `Option<&str>`: pass `Some("my-name")` to get-or-create a **named**
storage (reused across runs), or `None` for an unnamed one. The same signature applies to the
key-value-store and request-queue collections.

```rust,no_run
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
// Named: returns the existing dataset if one with this name exists, else creates it.
let dataset = client.datasets().get_or_create(Some("my-results")).await?;
// Unnamed:
let scratch = client.datasets().get_or_create(None).await?;
# let _ = (dataset, scratch);
# Ok(())
# }
```

`DatasetClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Dataset>` | Dataset metadata. |
| `update(fields)` | `&impl Serialize` | `Dataset` | Updates metadata. |
| `delete()` | — | `()` | Deletes the dataset. |
| `list_items::<T>(options)` | `DatasetListItemsOptions` | `PaginationList<T>` | Reads items (pagination via response headers). |
| `iterate_items::<T>(options)` | `DatasetListItemsOptions` | `ListIterator<T>` | Lazily iterates all items across pages (auto-pagination). |
| `push_items(items)` | `&impl Serialize` | `()` | Appends items (object or array). |
| `get_statistics()` | — | `Option<Value>` | Field statistics. |
| `download_items(format, options)` | `DownloadItemsFormat`, `DatasetDownloadOptions` | `Vec<u8>` | Export items as JSON/JSONL/CSV/XLSX/XML/RSS/HTML. |
| `create_items_public_url(options, expires_in_secs)` | `DatasetListItemsOptions`, `Option<i64>` | `String` | Shareable (HMAC-signed for private) items URL. |

`DatasetListItemsOptions` (all optional):

| Field | Type | Description |
|---|---|---|
| `offset` | `Option<i64>` | Number of items to skip. |
| `limit` | `Option<i64>` | Maximum number of items to return. |
| `desc` | `Option<bool>` | Reverse (newest-first) ordering. |
| `fields` | `Option<Vec<String>>` | Allow-list of top-level fields to keep in each item. |
| `output_fields` | `Option<Vec<String>>` | Positionally renames the fields selected by `fields` in the output; requires `fields`, and the two lists must have equal length (the i-th `output_fields` name becomes the output name of the i-th `fields` entry). |
| `omit` | `Option<Vec<String>>` | Fields to drop from each item. |
| `skip_empty` | `Option<bool>` | Omit items that are empty after field filtering. |
| `skip_hidden` | `Option<bool>` | Omit hidden fields (those whose names start with `#`). |
| `clean` | `Option<bool>` | Shorthand for `skip_hidden` + `skip_empty` (only non-empty, non-hidden items). |
| `unwind` | `Option<Vec<String>>` | Fields whose array values are expanded into separate items. |
| `flatten` | `Option<Vec<String>>` | Fields whose nested objects are flattened into dotted keys. |
| `view` | `Option<String>` | Name of a dataset view to apply. |
| `simplified` | `Option<bool>` | Return the simplified form of the items. |
| `skip_failed_pages` | `Option<bool>` | Skip items that come from failed pages (crawler datasets). |
| `signature` | `Option<String>` | Pre-shared URL signature granting access to a private dataset without an API token. |

`DatasetDownloadOptions` adds format-specific export controls (all optional except `items`,
which embeds a `DatasetListItemsOptions` for the same filtering/projection as `list_items`):

| Field | Type | Description |
|---|---|---|
| `items` | `DatasetListItemsOptions` | Shared item filtering/projection options (see above). |
| `attachment` | `Option<bool>` | Set the `Content-Disposition: attachment` header so browsers download the file. |
| `bom` | `Option<bool>` | Prepend a UTF-8 byte-order mark (useful for CSV opened in Excel). |
| `delimiter` | `Option<String>` | CSV field delimiter (default `,`). |
| `skip_header_row` | `Option<bool>` | Omit the CSV header row. |
| `xml_root` / `xml_row` | `Option<String>` / `Option<String>` | Element names for the XML root and per-item rows. |
| `feed_title` / `feed_description` | `Option<String>` / `Option<String>` | Title and description for RSS output. |

`create_items_public_url`'s `expires_in_secs` bounds how long the URL's HMAC-SHA256 `signature`
stays valid, as a **relative** number of seconds from the moment the URL is created (not a Unix
timestamp) — `None` produces a signature that never expires. It has no effect when the dataset
does not expose a URL-signing secret key (i.e. it isn't configured for signed access), in which
case the URL is returned unsigned regardless.

`DownloadItemsFormat` (re-exported at the crate root) selects the export format for
`download_items`. Variants: `Json`, `Jsonl`, `Csv`, `Xlsx`, `Xml`, `Rss`, `Html`. The method
returns the raw exported bytes (`Vec<u8>`) — for example, CSV text or the binary XLSX workbook —
which you can write to a file or forward to another service:

```rust,no_run
# use apify_client::{ApifyClient, DownloadItemsFormat};
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let dataset = client.datasets().get_or_create(None).await?;
let csv: Vec<u8> = client
    .dataset(&dataset.id)
    .download_items(DownloadItemsFormat::Csv, Default::default())
    .await?;
println!("exported {} bytes of CSV", csv.len());
# Ok(())
# }
```

## Key-value stores — `client.key_value_stores()` / `client.key_value_store(id)`

`KeyValueStoreCollectionClient`: `list(options: StorageListOptions)`,
`iterate(options: StorageListOptions)` (lazy `ListIterator<KeyValueStore>` auto-pagination),
`get_or_create(name: Option<&str>)`.

`KeyValueStoreClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<KeyValueStore>` | Store metadata. |
| `update(fields)` | `&impl Serialize` | `KeyValueStore` | Updates metadata. |
| `delete()` | — | `()` | Deletes the store. |
| `list_keys(options)` | `ListKeysOptions` | `KeyValueStoreKeysPage` | Lists one page of keys (key-based pagination). |
| `iterate_keys(options)` | `ListKeysOptions` | `KeyValueStoreKeysIterator` | Lazily iterates all keys across pages (cursor-based auto-pagination). |
| `record_exists(key)` | `&str` | `bool` | Whether a record exists (HEAD). |
| `get_record(key)` | `&str` | `Option<KeyValueStoreRecord>` | Reads a record's raw value. |
| `set_record_raw(key, bytes, content_type)` | `&str`, `Vec<u8>`, `&str` | `()` | Stores a raw record. |
| `set_record_json(key, value)` | `&str`, `&impl Serialize` | `()` | Stores a JSON record. |
| `delete_record(key)` | `&str` | `()` | Deletes a record. |
| `get_record_with_options(key, options)` | `&str`, `GetRecordOptions { attachment: Option<bool>, signature: Option<String> }` | `Option<KeyValueStoreRecord>` | Reads a record with explicit attachment/signature options. |
| `get_record_public_url(key)` | `&str` | `String` | Shareable (HMAC-signed for private) record URL. |
| `create_keys_public_url(expires_in_secs)` | `Option<i64>` | `String` | Shareable keys-list URL. |

`create_keys_public_url`'s `expires_in_secs` has the same semantics as
`create_items_public_url`'s (see above): a relative number of seconds from creation time
bounding the signature's validity, with `None` meaning it never expires, and no effect when the
store isn't configured for signed access.

`ListKeysOptions`: `limit: Option<i64>`, `exclusive_start_key: Option<String>`,
`prefix: Option<String>`, `collection: Option<String>`, `signature: Option<String>`. Like
`StoreListOptions.limit`, the meaning of `limit` depends on the method: for `list_keys` it is a
single page's size (max keys returned by one call, capped at 1000 by the API); for `iterate_keys`
it is a cap on the *total* number of keys yielded across all pages (unset iterates the whole
store).
`KeyValueStoreRecord` exposes `value: Vec<u8>`, `content_type`, plus `as_text()` and
`json::<T>()` helpers.

`iterate_keys(options)` returns a `KeyValueStoreKeysIterator` — the auto-paginating counterpart to
`list_keys` (which returns a single page). Key-value stores use cursor-based pagination, so the
iterator threads the `nextExclusiveStartKey` cursor through for you. Its `next()` is `async` and
fallible, returning `ApifyClientResult<Option<KeyValueStoreKey>>` and yielding `Ok(None)` once the
store is exhausted. `options.limit` caps the total number of keys yielded (unset iterates the whole
store); each individual request is bounded to the endpoint's maximum page size (1000), so a larger
cap still paginates. `prefix`/`collection`/`signature` filter every page:

```rust,no_run
# use apify_client::{ApifyClient, ListKeysOptions};
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
// Obtain a store id from a metadata model (e.g. get_or_create), then iterate its keys.
let store = client.key_value_stores().get_or_create(None).await?;
let mut keys = client.key_value_store(&store.id).iterate_keys(ListKeysOptions::default());
while let Some(key) = keys.next().await? {
    // `size` is optional; default to 0 bytes when the API does not report it.
    println!("{} ({} bytes)", key.key, key.size.unwrap_or(0));
}
# Ok(())
# }
```

`KeyValueStoreKey` (from `apify_client::models`) is the element type yielded by the iterator and
listed in `KeyValueStoreKeysPage::items`. Its fields:

| Field | Type | Description |
|---|---|---|
| `key` | `String` | The record key (always present). |
| `size` | `Option<i64>` | Size of the record value in bytes, if reported by the API. |
| `extra` | `Extra` | Any other fields returned by the API. |

## Request queues — `client.request_queues()` / `client.request_queue(id)`

`RequestQueueCollectionClient`: `list(options: StorageListOptions)`,
`iterate(options: StorageListOptions)` (lazy `ListIterator<RequestQueue>` auto-pagination),
`get_or_create(name: Option<&str>)`.

`RequestQueueClient::with_client_key(client_key: impl Into<String>) -> RequestQueueClient`
consumes `self` and returns a new client that sends the given `clientKey` as a query parameter on
most request-level operations (`list_head`, `add_request`, `update_request`, `delete_request`,
`list_and_lock_head`, the batch add/delete calls, `list_requests`, `prolong_request_lock`,
`delete_request_lock`, `unlock_requests`). It is *not* sent on the queue-level metadata methods
`get`, `update`, and `delete`, nor on `get_request` — matching the JS reference client, whose
`getRequest` is the one request-level method that builds its params from bare `this._params()`
instead of merging in `clientKey: this.clientKey` the way its siblings do. `client_key` should be
a stable, unique identifier for *this* consumer (e.g. one value per crawler process), reused
across calls so the API can attribute locks to their owner. It has no effect on unlocked
request-level operations (`add_request`, `list_requests`, …) beyond being sent along; it matters
specifically for the locking methods below, whose lock ownership and `unlock_requests`'s scope
are both keyed on it. See
[locking requests](#locking-requests) below for a worked example.

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<RequestQueue>` | Queue metadata. |
| `update(fields)` | `&impl Serialize` | `RequestQueue` | Updates metadata. |
| `delete()` | — | `()` | Deletes the queue. |
| `list_head(limit)` | `Option<i64>` | `RequestQueueHead` | Requests at the head. |
| `add_request(request, forefront)` | `&RequestQueueRequest`, `bool` | `RequestQueueOperationInfo` | Adds a request. |
| `get_request(id)` | `&str` | `Option<RequestQueueRequest>` | Reads a request. |
| `update_request(request, forefront)` | `&RequestQueueRequest`, `bool` | `RequestQueueOperationInfo` | Updates a request. |
| `delete_request(id)` | `&str` | `()` | Deletes a request. |
| `list_and_lock_head(lock_secs, limit)` | `i64`, `Option<i64>` | `Value` | Locks head requests. |
| `batch_add_requests(requests, forefront)` | `&[RequestQueueRequest]`, `bool` | `Value` | Batch add with reference-matching defaults (see below). |
| `batch_add_requests_with_options(requests, options)` | `&[RequestQueueRequest]`, `BatchAddRequestsOptions` | `Value` | Batch add with explicit retry/parallelism control (see below). |
| `batch_delete_requests(requests)` | `&[impl Serialize]` | `Value` | Batch delete. |
| `list_requests(options)` | `ListRequestsOptions { limit: Option<i64>, exclusive_start_id: Option<String>, cursor: Option<String>, filter: Option<Vec<String>> }` | `Value` | List requests (cursor/filter pagination). |
| `paginate_requests(page_limit)` | `Option<i64>` | `RequestQueueRequestsIterator` | Lazy request iterator. |
| `prolong_request_lock(id, lock_secs, forefront)` | `&str`, `i64`, `bool` | `Value` | Extend a lock. |
| `delete_request_lock(id, forefront)` | `&str`, `bool` | `()` | Release a lock. |
| `unlock_requests()` | — | `Value` | Release all this client's locks. |

`paginate_requests(page_limit)` returns a `RequestQueueRequestsIterator` — a lazy, page-fetching
iterator (parity with the Store iterator in [Store, users and logs](misc.md#apify-store--clientstore)).
It is named `paginate_requests` (rather than an `iterate_*` verb like the dataset/key-value-store
iterators) to mirror the reference JavaScript client's `paginateRequests` method, keeping the
public interface consistent across the two clients.
Its `next()` is `async` and fallible, returning
`ApifyClientResult<Option<RequestQueueRequest>>`, fetching the next page on demand and yielding
`Ok(None)` once the queue is exhausted. `page_limit` bounds the requests fetched per page (`None`
uses the server default). Drive it with `.await?`:

```rust,no_run
use apify_client::models::RequestQueueRequest;
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let queue = client.request_queues().get_or_create(None).await?;
let queue_client = client.request_queue(&queue.id);

// Add a request first so the iteration below has something to yield.
queue_client
    .add_request(
        &RequestQueueRequest {
            id: None,
            url: "https://example.com/".to_string(),
            unique_key: Some("example".to_string()),
            method: Some("GET".to_string()),
            user_data: None,
            extra: Default::default(),
        },
        false,
    )
    .await?;

let mut iter = queue_client.paginate_requests(None);
while let Some(request) = iter.next().await? {
    println!("{:?}: {}", request.id, request.url);
}
# Ok(())
# }
```

The `forefront` boolean (on `add_request`, `update_request`, `batch_add_requests`,
`prolong_request_lock`, `delete_request_lock`) controls queue ordering: `true` puts the
request(s) at the **front** of the queue so they are handled before the existing backlog;
`false` (the usual choice) appends them at the **back**.

### Batch-adding requests — `batch_add_requests` / `batch_add_requests_with_options`

`batch_add_requests(requests, forefront)` is the common case: it delegates to
`batch_add_requests_with_options` with reference-matching defaults, so most callers never need
the options form. Both handle an arbitrarily large `requests` slice safely and never fail merely
because part of a large batch was rate-limited:

- **Chunking.** The API's `requests/batch` endpoint accepts at most 25 requests per call and a
  limited payload size. `requests` is split into chunks of at most 25 items, further sliced so
  each chunk's serialized JSON stays under the API's byte-size limit (large `user_data` payloads
  can make even a handful of requests exceed it).
- **Parallelism.** Chunks are sent concurrently, up to `options.max_parallel` calls in flight at
  once (default `5`).
- **Unprocessed-request retries.** A chunk call can report some of its requests as
  `unprocessedRequests` (typically transient rate-limiting). These are automatically retried, up
  to `options.max_unprocessed_requests_retries` times (default `3`), with exponential backoff
  starting at `options.min_delay_between_unprocessed_requests_retries` (default `500ms`).
- **No throw on partial failure.** If a chunk call fails outright (not just reports
  `unprocessedRequests`), the method does not return an `Err` for the whole batch: the requests in
  that chunk are folded into the returned `unprocessedRequests` instead, matching the JS reference
  client. Always check the returned `unprocessedRequests` array, not just `Ok`/`Err`, to detect a
  partially-submitted batch.

`BatchAddRequestsOptions` fields (all optional):

- `forefront` — add all requests to the front of the queue (default `false`).
- `max_unprocessed_requests_retries` — retry attempts for a chunk's `unprocessedRequests` (default
  `3`).
- `max_parallel` — maximum concurrent `requests/batch` calls (default `5`).
- `min_delay_between_unprocessed_requests_retries` — base backoff delay before the first retry,
  doubling (with jitter) on each subsequent one (default `500ms`).

```rust,no_run
use apify_client::{models::RequestQueueRequest, BatchAddRequestsOptions};
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let queue = client.request_queues().get_or_create(None).await?;
let requests: Vec<RequestQueueRequest> = (0..100)
    .map(|i| RequestQueueRequest {
        id: None,
        url: format!("https://example.com/{i}"),
        unique_key: Some(format!("page-{i}")),
        method: Some("GET".to_string()),
        user_data: None,
        extra: Default::default(),
    })
    .collect();

// Explicit options: cap concurrency and retries beyond the defaults.
let result = client
    .request_queue(&queue.id)
    .batch_add_requests_with_options(
        &requests,
        BatchAddRequestsOptions {
            max_parallel: Some(2),
            ..Default::default()
        },
    )
    .await?;
println!(
    "processed {}, unprocessed {}",
    result["processedRequests"].as_array().map_or(0, Vec::len),
    result["unprocessedRequests"].as_array().map_or(0, Vec::len),
);
# Ok(())
# }
```

Some request-queue methods return an untyped `serde_json::Value` because the API responses are
open-ended and most callers do not consume them structurally. Their shapes (read fields with
`value.get("...")`):

- `list_and_lock_head` → an object with `items` (the locked head requests), `limit`,
  `queueModifiedAt`, `hadMultipleClients`, and the granted `lockSecs`.
- `batch_add_requests` / `batch_delete_requests` → an object with `processedRequests` and
  `unprocessedRequests` arrays.
- `list_requests` → an object with `items` (the page of requests), `count`, `limit`, and
  `exclusiveStartId` for cursor continuation.
- `unlock_requests` → an object reporting how many locks were released (`unlockedCount`).

### Locking requests

`list_and_lock_head`, `prolong_request_lock`, `delete_request_lock`, and `unlock_requests` let
several consumers share one queue without processing the same request twice: a consumer locks a
batch of requests off the head, processes them, and either deletes them (done) or releases the
lock (so another consumer can pick them up). Give each consumer its own `RequestQueueClient` via
`with_client_key` so the API can tell locks apart by owner — reuse the *same* client (and key)
for the whole lock/prolong/unlock lifecycle of that consumer:

```rust,no_run
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let queue = client.request_queues().get_or_create(None).await?;
// One client per consumer; `client_key` should be stable and unique per consumer process.
let queue_client = client
    .request_queue(&queue.id)
    .with_client_key("worker-1");

// Lock up to 10 requests from the head for 60 seconds.
let locked = queue_client.list_and_lock_head(60, Some(10)).await?;
let items = locked["items"].as_array().cloned().unwrap_or_default();
println!("locked {} request(s)", items.len());

for item in &items {
    // Skip anything without a usable ID rather than issuing a request against an empty path.
    let Some(id) = item["id"].as_str() else {
        continue;
    };

    // ... process the request here ...

    // If processing takes longer than expected, extend the lock instead of losing it:
    // queue_client.prolong_request_lock(id, 60, false).await?;

    // Or, to give up on a request without deleting it, release just its lock:
    // queue_client.delete_request_lock(id, false).await?;

    // Done: delete it. A missing/already-deleted request errors (see error handling).
    queue_client.delete_request(id).await?;
}

// Release every lock still held by this client (e.g. on shutdown):
queue_client.unlock_requests().await?;
# Ok(())
# }
```

### `RequestQueueRequest` and request-queue return types

`RequestQueueRequest` (from `apify_client::models`) is the value passed to `add_request` /
`update_request` and returned by `get_request` / inside `RequestQueueHead`. Its fields:

| Field | Type | Description |
|---|---|---|
| `id` | `Option<String>` | Request ID assigned by the API; leave `None` when adding a new request. |
| `url` | `String` | The URL to process (required). |
| `unique_key` | `Option<String>` | Dedup key (defaults to `url` server-side when omitted). |
| `method` | `Option<String>` | HTTP method (defaults to `GET`). |
| `user_data` | `Option<serde_json::Value>` | Arbitrary user data attached to the request. |
| `extra` | `Extra` | Any other fields returned by the API; use `Default::default()` when constructing. |

Construct one and add it to a queue:

```rust,no_run
use apify_client::models::RequestQueueRequest;
# use apify_client::ApifyClient;
# async fn run(client: ApifyClient) -> Result<(), Box<dyn std::error::Error>> {
let queue = client.request_queues().get_or_create(None).await?;
let queue_client = client.request_queue(&queue.id);

let request = RequestQueueRequest {
    id: None,
    url: "https://example.com/".to_string(),
    unique_key: Some("example".to_string()),
    method: Some("GET".to_string()),
    user_data: None,
    extra: Default::default(),
};
let info = queue_client.add_request(&request, false).await?;
println!("added request {}", info.request_id);

let head = queue_client.list_head(Some(10)).await?;
println!("{} request(s) at the head", head.items.len());
# Ok(())
# }
```

Relevant return-type fields:

- `RequestQueueOperationInfo`: `request_id: String`, `was_already_present: bool`,
  `was_already_handled: bool`.
- `RequestQueueHead`: `limit: i64`, `had_multiple_clients: bool`,
  `items: Vec<RequestQueueRequest>`, `extra: Extra` (any other fields returned by the API).
- `KeyValueStoreKeysPage`: `limit: i64`, `is_truncated: bool`, `exclusive_start_key`,
  `next_exclusive_start_key` (both `Option<String>`), `items: Vec<KeyValueStoreKey>`.

## Common list container — `PaginationList<T>`

Offset/limit-paginated list methods (`list_items`, the various collection `list` methods, …)
return `PaginationList<T>`. Re-exported at the crate root (`apify_client::PaginationList`). Fields:

| Field | Type | Description |
|---|---|---|
| `total` | `i64` | Total items available across all pages. |
| `offset` | `i64` | Items skipped at the start. |
| `limit` | `i64` | Max items the API would return for this request. |
| `count` | `i64` | Items actually returned in this page. |
| `desc` | `bool` | Whether the items are in descending order. |
| `items` | `Vec<T>` | The items of this page. |

The storage clients are also reachable from a run via `run.dataset()`,
`run.key_value_store()` and `run.request_queue()`.
