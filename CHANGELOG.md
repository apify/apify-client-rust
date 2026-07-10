# Changelog

All notable changes to the Rust Apify API client are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres
to [Semantic Versioning](https://semver.org/).

## [0.6.0] - 2026-07-10

### Added
- Lazy async pagination iterators on every collection client, via a shared generic
  `ListIterator<T>` (exported at the crate root). New `iterate()` methods on the actor, actor
  version, environment-variable, build, run, dataset, key-value-store, request-queue, schedule,
  task, webhook, and webhook-dispatch collection clients, plus `DatasetClient::iterate_items()`
  for dataset items. Each yields one item at a time, fetching pages on demand — the idiomatic
  counterpart to the reference client's async-iterable list results. The options' `limit` caps
  the total number of items yielded (matching the reference client), and `ListIterator::with_chunk_size`
  sets the per-request page size.
- `KeyValueStoreClient::iterate_keys()`, returning a cursor-based `KeyValueStoreKeysIterator`
  that auto-paginates a store's keys via `exclusiveStartKey`/`nextExclusiveStartKey` (the
  auto-paginating counterpart to `list_keys`, matching the reference client's `listKeys()`
  async-iterable).
- Re-exported `StoreActorIterator` at the crate root.

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-07-10T105921Z`. The spec delta (added `401`/`402`
  error responses and relaxed field nullability/optionality) needs no code change: error
  responses are handled generically and response models are forward-compatible.
- `StoreCollectionClient::iterate` now uses the shared `ListIterator`, and `StoreActorIterator`
  is a type alias for `ListIterator<ActorStoreListItem>`. As part of this, `store().iterate()`'s
  `options.limit` changed from a per-page size (0.5.0) to a cap on the total number of items
  yielded, for consistency with the reference client and the other `iterate()` methods; set the
  per-page size with `ListIterator::with_chunk_size` instead. The `StoreActorIterator` type alias
  itself is unchanged.
- Corrected the `src/models.rs` module doc to describe forward-compatibility accurately.
- Bumped crate version to `0.6.0`.

### Removed
- `KeyValueStoreClient::get_records` and `GetRecordsOptions`. The `GET /v2/key-value-stores/{storeId}/records`
  endpoint is not implemented by the reference JS client, so it is out of scope; its removal
  corrects an earlier scope violation.

## [0.5.0] - 2026-07-10

### Added
- `RequestCompression` enum and `ApifyClientBuilder::request_compression` to select the
  request-body encoding: brotli (`Content-Encoding: br`, default) or gzip
  (`Content-Encoding: gzip`).

### Changed
- The `User-Agent` OS token now matches the reference client's `os.platform()` value
  (`darwin`, `win32`, `sunos`) instead of Rust-native spellings (`macos`, `windows`, `solaris`).
- Bumped crate version to `0.5.0`.

## [0.4.8] - 2026-07-09

### Added
- Request bodies of at least 1024 bytes are now brotli-compressed and sent with
  `Content-Encoding: br`.

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-07-08T143931Z`.
- Bumped crate version to `0.4.8`.

## [0.4.7] - 2026-07-07

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-07-07T132551Z`.
- Bumped crate version to `0.4.7`.

### Documentation
- Corrected the `last_run`/`LastRunOptions` rustdoc comments: `origin` is now a documented query
  parameter on the last-run endpoints in the OpenAPI spec (the comments previously said it was
  undeclared).
- Fixed the `README.md` `monthly_usage` link to point at the Users section of `docs/misc.md`
  instead of the Logs section.

## [0.4.6] - 2026-07-07

### Changed
- Rewrote earlier `CHANGELOG.md` entries to satisfy the changelog requirements: condensed
  narrative prose into short change bullets and removed cross-client references to sibling
  implementations, references to requirement-tracking issues, and out-of-scope / not-implemented
  notes.
- Bumped crate version to `0.4.6`.

## [0.4.5] - 2026-07-03

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-07-02T131926Z`.
- Bumped crate version to `0.4.5`.

### Documentation
- Added `chrono` to the extra-dependency install instructions.
- Expanded the `DatasetListItemsOptions`/`DatasetDownloadOptions` and `ActorBuildOptions` field lists into per-field descriptions, and corrected the `output_fields` description.
- Removed a stray `get_statistics` bullet from the request-queue return-types list.

## [0.4.4] - 2026-07-01

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-07-01T115402Z`.
- Bumped crate version to `0.4.4`.

## [0.4.3] - 2026-06-30

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-06-30T091455Z`.
- Bumped crate version to `0.4.3`.

### Documentation
- `examples/get_account.rs`: import `chrono::Utc` explicitly so the snippet is self-contained.
- `RunCollectionClient::list`: added a rustdoc example for the two-argument `list(ListOptions, RunListOptions)` call.

## [0.4.2] - 2026-06-29

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-06-29T142258Z` (no in-scope API surface change).

## [0.4.1] - 2026-06-29

### Changed
- Documentation now states the client is "official, but experimental" and AI-generated/AI-maintained (crate-level rustdoc `src/lib.rs`, `README.md`, `docs/README.md`, and the `Cargo.toml` `description`).
- The crates.io publish workflow (`.github/workflows/rust-publish.yml`) now authenticates with Trusted Publishing (OIDC) via `rust-lang/crates-io-auth-action@v1` instead of a stored `CARGO_REGISTRY_TOKEN` secret; the job gains `id-token: write` permission to mint the short-lived token.

## [0.4.0] - 2026-06-26

### Added
- `ActorClient::last_run_with_options` and `TaskClient::last_run_with_options`, plus `LastRunOptions { status, origin }` (re-exported at the crate root), adding an `origin` filter alongside `status` on the last-run endpoints. The existing `last_run(status)` delegates to it (additive).
- `LogClient::get_with_options` / `stream_with_options` and `RunClient::get_streamed_log_with_options`, plus `LogOptions { raw }` (re-exported), exposing the spec's optional `raw` query parameter on the log endpoints. The existing no-argument methods delegate with default options (additive).
- `examples/raw_log.rs` (covered by the `Test examples` CI step) exercising the raw-log path end-to-end.
- Tests: `log_get_sends_raw_query_param`; a raw-log assertion in the `run_actor_and_read_outputs` integration flow.

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-06-25T142310Z`.
- Bumped crate version `0.3.0` → `0.4.0` (minor; additive APIs above).
- Tightened the `build_user_agent` `isAtHome` comment in `src/common.rs` (no behaviour change).

### Documentation
- Documented the accepted `last_run` `origin` values (`DEVELOPMENT`, `WEB`, `API`, `SCHEDULER`) in `docs/runs.md` and referenced them from `docs/actors.md` / `docs/tasks.md`; corrected the `TaskClient::last_run` docstring example to `"SCHEDULER"`; showed an `origin`-filtered call in `examples/run_and_last_run_storages.rs`.

## [0.3.0] - 2026-06-25

### Fixed
- `ListRequestsOptions.filter` (for `GET /v2/request-queues/{queueId}/requests`) is now `Option<Vec<String>>` serialized comma-joined, matching the spec (an array of the enum values `locked`/`pending`). Breaking change to that field's type.
- The `User-Agent` `isAtHome` flag is now based solely on the `APIFY_IS_AT_HOME` environment variable.

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-06-24T105326Z`.
- Bumped crate version `0.2.4` → `0.3.0` (minor; breaking `filter` type change).
- `RunMetamorphOptions` and `RunChargeOptions` are now re-exported at the crate root (additive).
- Documentation: the `log_redirection` example now redirects a separate Actor's run log; added model field tables for `Task`/`Schedule`/`Webhook`/`WebhookDispatch`; aligned `monthly_usage` docs on the non-panicking `Value::get` idiom; hardened the `last_run` example against eventual consistency; documented the crate-root import surface and the `APIFY_TOKEN` convention.

## [0.2.4] - 2026-06-23

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-06-23T113219Z`.
- Bumped crate version `0.2.3` → `0.2.4`.

## [0.2.3] - 2026-06-22

### Changed
- CI: the manually triggered `Publish Rust client to crates.io` workflow (`.github/workflows/rust-publish.yml`) now also tags the released commit and creates a matching GitHub release. The tag (`vX.Y.Z`) is derived from the `Cargo.toml` `version`, validated as bare semver, and checked against existing local/remote tags; it is created before `cargo publish`, and the GitHub release step is idempotent. The `dry_run` input skips tag and release creation.

## [0.2.2] - 2026-06-22

### Added
- CI: a manually triggered (`workflow_dispatch`) `Publish Rust client to crates.io` workflow (`.github/workflows/rust-publish.yml`) that publishes the crate to crates.io. It runs the format/clippy/build gate, a `cargo publish --dry-run` check, then `cargo publish`. The registry token is read from the `CARGO_REGISTRY_TOKEN` repository secret. A `dry_run` input allows a packaging-only run.

### Changed
- Packaging: added the `homepage` field (`https://apify.com`) to `Cargo.toml`.
- Documentation: added the "experimental, AI-generated and AI-maintained" disclaimer to the crate-level rustdoc (`src/lib.rs`), `README.md` and `docs/README.md`, and reworded the `Cargo.toml` `description` accordingly; documented `ApifyClientError::as_api_error`; removed dangling reference-style Markdown link brackets in `docs/README.md` and `docs/misc.md`; added a "Releasing" subsection to the README "Versioning" section.

## [0.2.1] - 2026-06-19

### Changed
- CI: added a standalone `Test examples` step that runs the `examples/` programs end-to-end against the live API and the in-documentation snippets as doctests; `Run integration tests` now skips the `example_*` tests (via `--skip example_`).
- Documentation testing: the external `docs/` pages are now compiled as doctests via `#[doc = include_str!]` in `src/lib.rs`, so every in-documentation `rust` snippet is verified.
- Documentation: added response-model field tables for `ActorRun`, `Actor`, `Build`, the shared storage-metadata fields of `Dataset`/`KeyValueStore`/`RequestQueue`, and `User`, each with a runnable `no_run` doctest.

## [0.2.0] - 2026-06-19

### Added
- `ActorClient::validate_input_for_build` — exposes the spec's optional `build` query parameter on `POST /v2/actors/{actorId}/validate-input`. The existing `validate_input` delegates with `None`.
- `UserClient::monthly_usage_for_date` — exposes the spec's optional `date` query parameter on `GET /v2/users/me/usage/monthly`. The existing `monthly_usage` delegates with `None`.
- Integration tests: `get_monthly_usage_for_date`; a `validate_input_for_build` call added to `build_actor_flow`.

### Changed
- Bumped `API_SPEC_VERSION` to `v2-2026-06-18T095846Z`.

### Fixed
- `validate_input` / `validate_input_for_build` no longer fail to parse the bare `{ "valid": ... }` response (new internal `post_action_raw` helper skips `data`-envelope unwrapping).
- `UserClient::monthly_usage`'s `me`-only guard error now names `monthly_usage` instead of the delegated `monthly_usage_for_date`.

## [0.1.0] - 2026-06-18

Initial release of the official Rust client for the Apify API.

### Added
- Resource-oriented async client (`ApifyClient` + `ApifyClientBuilder`) mirroring the JavaScript reference client.
- Resource clients for Actors, Actor builds, Actor runs, Actor tasks, datasets, key-value stores, request queues, schedules, webhooks, webhook dispatches, the Apify Store, users, Actor versions and environment variables, and logs.
- Convenience helpers: `actor.call` / `task.call` (start + wait + run), `run.wait_for_finish` and `build.wait_for_finish`, dataset push/list items, key-value store records, request queue operations, log retrieval and streaming (log redirection), and lazy iteration of Apify Store Actors (`store().iterate()`).
- Replaceable HTTP transport via the `HttpBackend` trait with a default reqwest backend.
- Automatic Bearer authentication, the mandated `User-Agent` header, and transparent retries with exponential backoff on `429`/`5xx`/network errors.
- Public version constants `CLIENT_VERSION` and `API_SPEC_VERSION` (`v2-2026-06-16T064758Z`).
- Request-queue lock lifecycle: `list_requests`, `list_and_lock_head`, `prolong_request_lock`, `delete_request_lock`, `unlock_requests`, plus `batch_add_requests` / `batch_delete_requests`.
- Dataset `download_items` (JSON/CSV/XLSX/XML/RSS/HTML export) and `get_statistics`.
- `KeyValueStoreClient::get_records` — downloads all records from a store as a ZIP archive, with `collection`/`prefix`/`signature` filtering via `GetRecordsOptions`.
- Build `get_openapi_definition`.
- `ApifyClient::set_status_message` for setting the current run's status from inside an Actor.
- Public, shareable resource URLs with HMAC-SHA256 signing for private resources: `DatasetClient::create_items_public_url`, `KeyValueStoreClient::get_record_public_url` / `create_keys_public_url`, plus a configurable `public_base_url`.
- Format-specific dataset export options via `DatasetDownloadOptions` (`attachment`, `bom`, `delimiter`, `skip_header_row`, `xml_root`, `xml_row`).
- Full parameter coverage on list/start endpoints (`my`/`sortBy` for Actors; `webhooks`, `restartOnError`, `forcePermissionLevel` for run start; `unwind`/`flatten`/`view`/etc. for dataset items; `includeUnrunnableActors` for store; `attachment` for KVS records).
- Offline unit tests (mock `HttpBackend`) covering retry counting, 429/5xx retry vs. 4xx no-retry, network-error retry, backoff, error-envelope parsing and 404→None mapping.
- Integration test suite covering simple GETs and full CRUD flows for each resource.
- GitHub Actions workflow running formatting, clippy, build and integration tests.

### Fixed
- Percent-encode URL path segments (key-value-store record keys, request-queue request IDs) so keys containing `/`, `?`, `#`, spaces or non-ASCII no longer produce malformed URLs.
- `get_record` now sends `attachment=true`, matching the reference client's `getRecord`. `get_record_with_options` takes a `GetRecordOptions { attachment, signature }`; `attachment` defaults to `true` when unset.
- Request-queue request pagination (`paginate_requests` / `RequestQueueRequestsIterator`) now feeds the opaque `nextCursor` back as the `cursor` query parameter on subsequent pages, matching the reference client (previously pagination broke past the first page).
- `User-Agent` `{language version}` now reports the real compiler version captured at build time (via `build.rs` running `rustc --version`) instead of `Rust/unknown`.

### Changed
- Added previously-missing spec query parameters: dataset items `outputFields` (list/download) and `feedTitle`/`feedDescription` (download); key-value-store keys `collection`/`signature`; key-value-store record `signature` (via `GetRecordOptions`); request-queue requests `cursor`/`filter` (via `ListRequestsOptions`); storage collection list `unnamed`/`ownership` (via `StorageListOptions`); run collection list `startedAfter`/`startedBefore` (via `RunListOptions`).
- `get_record_with_options` signature changed to take `GetRecordOptions` (was `attachment: bool`), exposing the spec `signature` param for reading records from private stores.
- `ActorClient::default_build` now takes a `wait_for_finish: Option<i64>` argument, matching the reference client's `defaultBuild(options)`.
- Backoff doubling factor extracted to a named constant.
- `RunClient::charge` now takes `RunChargeOptions { event_name, count, idempotency_key }` and always sends an `idempotency-key` header (auto-generated when omitted), so a transport-retried charge is applied at most once.
- `RunClient::metamorph` now takes `RunMetamorphOptions { build, content_type }`.
- `RunResurrectOptions` gained `max_items`, `max_total_charge_usd`, and `restart_on_error`.
- `RunListOptions::status` is now `Vec<String>` (sent comma-separated).
- `RunClient::abort` now takes `gracefully: Option<bool>` instead of `bool` (`None` omits the query parameter).
- `RequestQueueClient::batch_add_requests` now splits inputs larger than the API's 25-per-call limit into chunks and merges the per-chunk `processedRequests`/`unprocessedRequests` results.
