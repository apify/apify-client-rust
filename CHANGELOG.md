# Changelog

All notable changes to the Rust Apify API client are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres
to [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-06-18

Initial release of the official Rust client for the Apify API.

### Added
- Resource-oriented async client (`ApifyClient` + `ApifyClientBuilder`) mirroring the
  official JavaScript and Python clients.
- Resource clients for Actors, Actor builds, Actor runs, Actor tasks, datasets, key-value
  stores, request queues, schedules, webhooks, webhook dispatches, the Apify Store, users,
  Actor versions and environment variables, and logs.
- Convenience helpers: `actor.call` / `task.call` (start + wait + run), `run.wait_for_finish`
  and `build.wait_for_finish`, dataset push/list items, key-value store records, request
  queue operations, log retrieval and streaming (log redirection), and lazy iteration of
  Apify Store Actors (`store().iterate()`).
- Replaceable HTTP transport via the `HttpBackend` trait with a default reqwest backend.
- Automatic Bearer authentication, the mandated `User-Agent` header, and transparent
  retries with exponential backoff on `429`/`5xx`/network errors.
- Public version constants `CLIENT_VERSION` and `API_SPEC_VERSION`
  (`v2-2026-06-16T064758Z`).
- Request-queue lock lifecycle: `list_requests`, `list_and_lock_head`, `prolong_request_lock`,
  `delete_request_lock`, `unlock_requests`, plus `batch_add_requests` / `batch_delete_requests`.
- Dataset `download_items` (JSON/CSV/XLSX/XML/RSS/HTML export) and `get_statistics`.
- Build `get_openapi_definition`.
- `ApifyClient::set_status_message` for setting the current run's status from inside an Actor.
- Public, shareable resource URLs with HMAC-SHA256 signing for private resources:
  `DatasetClient::create_items_public_url`, `KeyValueStoreClient::get_record_public_url` /
  `create_keys_public_url`, plus a configurable `public_base_url`.
- Format-specific dataset export options via `DatasetDownloadOptions`
  (`attachment`, `bom`, `delimiter`, `skip_header_row`, `xml_root`, `xml_row`).
- Full parameter coverage on list/start endpoints (`my`/`sortBy` for Actors; `webhooks`,
  `restartOnError`, `forcePermissionLevel` for run start; `unwind`/`flatten`/`view`/etc. for
  dataset items; `includeUnrunnableActors` for store; `attachment` for KVS records).
- Offline unit tests (mock `HttpBackend`) covering retry counting, 429/5xx retry vs. 4xx
  no-retry, network-error retry, backoff, error-envelope parsing and 404→None mapping.
- Integration test suite covering simple GETs and full CRUD flows for each resource.
- GitHub Actions workflow running formatting, clippy, build and integration tests.

### Fixed
- Percent-encode URL path segments (key-value-store record keys, request-queue request IDs)
  so keys containing `/`, `?`, `#`, spaces or non-ASCII no longer produce malformed URLs.
- `User-Agent` `isAtHome` flag now reads the platform variable `APIFY_IS_AT_HOME` (matching
  the JS reference) instead of the literal `isAtHome`, so it is correct on the Apify platform.
- `get_record` now sends `attachment=true`, matching the reference client's `getRecord`
  (which sends `attachment=true` unconditionally). `get_record_with_options` takes a
  `GetRecordOptions { attachment, signature }`; `attachment` defaults to `true` when unset.

### Changed
- Added previously-missing spec query parameters: dataset items `outputFields` (list/download)
  and `feedTitle`/`feedDescription` (download); key-value-store keys `collection`/`signature`;
  key-value-store record `signature` (via `GetRecordOptions`); request-queue requests
  `cursor`/`filter` (via `ListRequestsOptions`); storage collection list `unnamed`/`ownership`
  (via `StorageListOptions`); run collection list `startedAfter`/`startedBefore` (via
  `RunListOptions`).
- `get_record_with_options` signature changed to take `GetRecordOptions` (was `attachment: bool`),
  exposing the spec `signature` param for reading records from private stores.
- Backoff doubling factor extracted to a named constant.

### Notes
- A few documented endpoints are intentionally not exposed (matching the JS reference):
  synchronous run endpoints, `/tools/*`, `/browser-info`, and the keyed-`POST` create variants
  for Actor versions and version env-vars (creation is via `POST` to the collection, upsert via
  `PUT` on the keyed path).
