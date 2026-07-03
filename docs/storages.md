# Storages: datasets, key-value stores, request queues

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
| `push_items(items)` | `&impl Serialize` | `()` | Appends items (object or array). |
| `get_statistics()` | — | `Option<Value>` | Field statistics. |
| `download_items(format, options)` | `DownloadItemsFormat`, `DatasetDownloadOptions` | `Vec<u8>` | Export items as JSON/CSV/XLSX/XML/RSS/HTML. |
| `create_items_public_url(options, expires)` | `DatasetListItemsOptions`, `Option<i64>` | `String` | Shareable (HMAC-signed for private) items URL. |

`DatasetListItemsOptions` (all optional):

- `offset` / `limit` / `desc` — pagination window and reverse (newest-first) ordering.
- `fields` — comma-separated allow-list of top-level fields to keep in each item.
- `output_fields` — positionally renames the fields selected by `fields` in the output; requires
  `fields`, and the two lists must have equal length (the i-th `output_fields` name becomes the
  output name of the i-th `fields` entry).
- `omit` — comma-separated fields to drop from each item.
- `skip_empty` — omit items that are empty after field filtering.
- `skip_hidden` — omit hidden fields (those whose names start with `#`).
- `clean` — shorthand for `skip_hidden` + `skip_empty` (only non-empty, non-hidden items).
- `unwind` — comma-separated fields whose array values are expanded into separate items.
- `flatten` — comma-separated fields whose nested objects are flattened into dotted keys.
- `view` — name of a dataset view to apply.
- `simplified` — return the simplified form of the items.
- `skip_failed_pages` — skip pages that failed to be scraped (crawler datasets).

`DatasetDownloadOptions` adds format-specific export controls (all optional):

- `attachment` — set the `Content-Disposition: attachment` header so browsers download the file.
- `bom` — prepend a UTF-8 byte-order mark (useful for CSV opened in Excel).
- `delimiter` — CSV field delimiter (default `,`).
- `skip_header_row` — omit the CSV header row.
- `xml_root` / `xml_row` — element names for the XML root and per-item rows.
- `feed_title` / `feed_description` — title and description for RSS output.

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

`KeyValueStoreCollectionClient`: `list(options: StorageListOptions)`, `get_or_create(name: Option<&str>)`.

`KeyValueStoreClient`:

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<KeyValueStore>` | Store metadata. |
| `update(fields)` | `&impl Serialize` | `KeyValueStore` | Updates metadata. |
| `delete()` | — | `()` | Deletes the store. |
| `list_keys(options)` | `ListKeysOptions` | `KeyValueStoreKeysPage` | Lists keys (key-based pagination). |
| `get_records(options)` | `GetRecordsOptions { collection, prefix, signature }` | `Vec<u8>` | Downloads all records as a ZIP archive (raw bytes). |
| `record_exists(key)` | `&str` | `bool` | Whether a record exists (HEAD). |
| `get_record(key)` | `&str` | `Option<KeyValueStoreRecord>` | Reads a record's raw value. |
| `set_record_raw(key, bytes, content_type)` | `&str`, `Vec<u8>`, `&str` | `()` | Stores a raw record. |
| `set_record_json(key, value)` | `&str`, `&impl Serialize` | `()` | Stores a JSON record. |
| `delete_record(key)` | `&str` | `()` | Deletes a record. |
| `get_record_with_options(key, options)` | `&str`, `GetRecordOptions { attachment, signature }` | `Option<KeyValueStoreRecord>` | Reads a record with explicit attachment/signature options. |
| `get_record_public_url(key)` | `&str` | `String` | Shareable (HMAC-signed for private) record URL. |
| `create_keys_public_url(expires)` | `Option<i64>` | `String` | Shareable keys-list URL. |

`ListKeysOptions`: `limit`, `exclusive_start_key`, `prefix`, `collection`, `signature`.
`KeyValueStoreRecord` exposes `value: Vec<u8>`, `content_type`, plus `as_text()` and
`json::<T>()` helpers.

## Request queues — `client.request_queues()` / `client.request_queue(id)`

`RequestQueueCollectionClient`: `list(options: StorageListOptions)`, `get_or_create(name: Option<&str>)`.

`RequestQueueClient` (chainable `with_client_key(key)` for lock coordination):

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
| `batch_add_requests(requests, forefront)` | `&[RequestQueueRequest]`, `bool` | `Value` | Batch add. |
| `batch_delete_requests(requests)` | `&[impl Serialize]` | `Value` | Batch delete. |
| `list_requests(options)` | `ListRequestsOptions { limit, exclusive_start_id, cursor, filter }` | `Value` | List requests (cursor/filter pagination). |
| `paginate_requests(page_limit)` | `Option<i64>` | `RequestQueueRequestsIterator` | Lazy request iterator. |
| `prolong_request_lock(id, lock_secs, forefront)` | `&str`, `i64`, `bool` | `Value` | Extend a lock. |
| `delete_request_lock(id, forefront)` | `&str`, `bool` | `()` | Release a lock. |
| `unlock_requests()` | — | `Value` | Release all this client's locks. |

The `forefront` boolean (on `add_request`, `update_request`, `batch_add_requests`,
`prolong_request_lock`, `delete_request_lock`) controls queue ordering: `true` puts the
request(s) at the **front** of the queue so they are handled before the existing backlog;
`false` (the usual choice) appends them at the **back**.

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
