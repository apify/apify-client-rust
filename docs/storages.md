# Storages: datasets, key-value stores, request queues

## Datasets — `client.datasets()` / `client.dataset(id)`

`DatasetCollectionClient`: `list(options: StorageListOptions)`, `get_or_create(name)`.
`StorageListOptions`: `offset`, `limit`, `desc`, `unnamed`, `ownership`.

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

`DatasetListItemsOptions`: `offset`, `limit`, `desc`, `fields`, `output_fields`, `omit`,
`skip_empty`, `skip_hidden`, `clean`, `unwind`, `flatten`, `view`, `simplified`,
`skip_failed_pages`. `DatasetDownloadOptions` adds `attachment`, `bom`, `delimiter`,
`skip_header_row`, `xml_root`, `xml_row`, `feed_title`, `feed_description`.

## Key-value stores — `client.key_value_stores()` / `client.key_value_store(id)`

`KeyValueStoreCollectionClient`: `list(options: StorageListOptions)`, `get_or_create(name)`.

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

`RequestQueueCollectionClient`: `list(options: StorageListOptions)`, `get_or_create(name)`.

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
| `list_and_lock_head(lock_secs, limit)` | `i64`, `Option<i64>` | `Value` | Lock head requests. |
| `prolong_request_lock(id, lock_secs, forefront)` | `&str`, `i64`, `bool` | `Value` | Extend a lock. |
| `delete_request_lock(id, forefront)` | `&str`, `bool` | `()` | Release a lock. |
| `unlock_requests()` | — | `Value` | Release all this client's locks. |

The storage clients are also reachable from a run via `run.dataset()`,
`run.key_value_store()` and `run.request_queue()`.
