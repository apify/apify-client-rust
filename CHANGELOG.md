# Changelog

All notable changes to the Rust Apify API client are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres
to [Semantic Versioning](https://semver.org/).

## [0.2.3] - 2026-06-22

Publishing compliance for the updated client requirements (apify-client-orchestration PR #9),
which added: "Manual release workflow also creates a tagged GitHub release." No changes to the
public interface; release-workflow behaviour only.

### Changed
- CI: the manually triggered `Publish Rust client to crates.io` workflow
  (`.github/workflows/rust-publish.yml`) now also tags the released commit and creates a matching
  GitHub release, in addition to publishing to crates.io. The release tag (`vX.Y.Z`) is derived
  from the single source of truth (the `version` field in `Cargo.toml`, which is also what
  `CLIENT_VERSION` exposes via `CARGO_PKG_VERSION`), validated to be bare semver, and checked
  against existing local/remote tags so a release can never silently clobber a prior one. The
  workflow now requires the `master` branch, requests `contents: write` permission, and creates
  the GitHub release via `gh` using the default `GITHUB_TOKEN` repository secret. The release notes
  are extracted from the matching `CHANGELOG.md` section (falling back to a one-liner if absent).
  The tag and release are created before `cargo publish` so the immutable git tag/release stay
  consistent with the crate version even if the (unrepeatable) publish step fails. The GitHub
  release step is idempotent (updates an existing release rather than failing), and the README
  documents the "delete the tag and re-run" recovery procedure for a post-tag/pre-publish failure.
  The `dry_run` input now also skips tag and release creation. This mirrors the Go client's
  `go-publish.yml`.

## [0.2.2] - 2026-06-22

Publishing compliance for the updated client requirements (apify-client-orchestration PR #7).
No changes to the public interface; packaging metadata and a release workflow only.

### Added
- CI: a manually triggered (`workflow_dispatch`) `Publish Rust client to crates.io` workflow
  (`.github/workflows/rust-publish.yml`) that publishes the crate to crates.io — the
  language-specific distribution standard for Rust. It runs the format/clippy/build quality gate,
  performs a `cargo publish --dry-run` packaging check, then `cargo publish`. The registry token
  is read exclusively from the `CARGO_REGISTRY_TOKEN` repository secret, and the run fails early
  with a clear message if that secret is missing. A `dry_run` input allows a packaging-only run
  with no actual release.

### Changed
- Packaging: added the strongly recommended `homepage` field (`https://apify.com`) to
  `Cargo.toml` so the crates.io listing carries complete publishing metadata.
- Documentation: added the required "experimental, AI-generated and AI-maintained" disclaimer to
  the crate-level rustdoc (`src/lib.rs`), `README.md` and `docs/README.md`, and softened the
  "official" wording accordingly. The crates.io package `description` in `Cargo.toml` was likewise
  reworded to "An experimental, AI-generated and AI-maintained Rust client …" so the published
  one-line summary matches the disclaimer.
- Documentation: documented `ApifyClientError::as_api_error` (used in the README error-handling
  example) in the `docs/README.md` error-handling section, with a runnable snippet.
- Documentation: removed dangling reference-style Markdown link brackets around
  `apify_client::models` (`docs/README.md`) and `futures_util::StreamExt` (`docs/misc.md`) so
  they render as plain inline code rather than broken links on GitHub.
- Documentation: added a "Releasing" subsection to the README "Versioning" section describing the
  crates.io distribution mechanism and the publish workflow, for parity with the Go sibling.

## [0.2.1] - 2026-06-19

Compliance fix for the updated client/test requirements (apify-client-orchestration PR #4).
No changes to the public interface; CI and documentation-testing only.

### Changed
- CI: added a standalone `Test examples` workflow step that verifies the documentation
  examples actually work — it runs the `examples/` programs end-to-end against the live API
  (the `example_*` smoke tests in `tests/examples.rs`, each invoking `cargo run --example`) and
  runs the in-documentation code snippets as doctests (`cargo test --doc`). The example smoke
  tests were previously executed as part of the `Run integration tests` step and the doctests in
  a separate `Run documentation example tests` step; they are now consolidated under the
  requirement-named `Test examples` step. `Run integration tests` now skips the `example_*`
  tests (via `--skip example_`) so the two concerns stay separate.
- Documentation testing: the external `docs/` pages (`docs/README.md`, `docs/actors.md`,
  `docs/misc.md`, `docs/storages.md`, `docs/runs.md`, `docs/builds.md`) are now compiled as
  doctests via `#[doc = include_str!]` in `src/lib.rs`, so every in-documentation `rust` code
  snippet is verified valid and runnable by `cargo test --doc`. Previously only the root
  `README.md` snippets were doctest-checked.
- Documentation: added response-model field tables for the types the README Quick start and the
  examples read but which were previously undocumented — `ActorRun` (incl. `id`, `status`,
  `default_dataset_id`/`default_key_value_store_id`/`default_request_queue_id`) in
  `docs/runs.md`; `Actor` (`id`, …) and `Build` (`id`, `status`, …) in `docs/actors.md` (with a
  cross-reference from `docs/builds.md`); the shared storage-metadata fields of `Dataset` /
  `KeyValueStore` / `RequestQueue` (incl. `id`) in `docs/storages.md`; and `User` (`id`,
  `username`) in `docs/misc.md`. Each new section carries a runnable `no_run` doctest exercising
  the documented fields.

## [0.2.0] - 2026-06-19

Updated to Apify OpenAPI specification `v2-2026-06-18T095846Z` (previously
`v2-2026-06-16T064758Z`). The spec delta is small and fully additive; no breaking changes to
the public interface.

### Added
- `ActorClient::validate_input_for_build` — validates input against the input schema of a
  specific Actor build, exposing the spec's optional `build` query parameter on
  `POST /v2/actors/{actorId}/validate-input`. The existing `validate_input` is unchanged and now
  delegates to it with `None` (default build).
- `UserClient::monthly_usage_for_date` — fetches monthly usage for the month containing a given
  `YYYY-MM-DD` date, exposing the spec's optional `date` query parameter on
  `GET /v2/users/me/usage/monthly`. The existing `monthly_usage` is unchanged and now delegates
  to it with `None` (current month).
- Integration tests: `get_monthly_usage_for_date` (user) and a `validate_input_for_build` call
  added to `build_actor_flow` (where a real `latest` build exists to validate against).

### Changed
- `API_SPEC_VERSION` bumped to `v2-2026-06-18T095846Z`.

### Fixed
- `ActorClient::validate_input` (and the new `validate_input_for_build`) no longer fail to parse
  the response. The `validate-input` endpoint returns a bare `{ "valid": ... }` object rather than
  the usual `{ "data": ... }` envelope, so it now skips `data`-envelope unwrapping (new internal
  `post_action_raw` helper). Previously any call returned a deserialization error
  (`missing field 'data'`). Exercised by the `validate_input_for_build` assertion in the
  `build_actor_flow` integration test.
- `UserClient::monthly_usage`'s `me`-only guard error now names `monthly_usage` instead of the
  delegated `monthly_usage_for_date`, so a non-`me` caller sees the method they actually called.

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
- `KeyValueStoreClient::get_records` — downloads all records from a store as a ZIP archive
  (`GET /v2/key-value-stores/{storeId}/records`), with `collection`/`prefix`/`signature`
  filtering via `GetRecordsOptions`. Reachable on run/task default stores via the nested
  `key_value_store()` accessor.
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
- `User-Agent` `isAtHome` flag now reads **both** the platform variable `APIFY_IS_AT_HOME`
  (matching the JS reference) and the bare `isAtHome` name from `client_requirements.md`; either
  being set marks the client "at home". These two same-priority requirements conflicted, so the
  client honours both (consistent with the Go sibling). The flag is rendered lowercase
  (`true`/`false`) to stay byte-consistent with the JS reference.
- `get_record` now sends `attachment=true`, matching the reference client's `getRecord`
  (which sends `attachment=true` unconditionally). `get_record_with_options` takes a
  `GetRecordOptions { attachment, signature }`; `attachment` defaults to `true` when unset.
- Request-queue request pagination (`paginate_requests` / `RequestQueueRequestsIterator`) now
  feeds the opaque `nextCursor` back as the `cursor` query parameter on subsequent pages
  (matching the JS reference) instead of misusing it as `exclusiveStartId`. Previously
  pagination broke past the first page (duplicate/missing items or premature stop).
- `User-Agent` `{language version}` now reports the real compiler version captured at build
  time (via `build.rs` running `rustc --version`) instead of rendering `Rust/unknown` (it
  previously read the unset MSRV field `CARGO_PKG_RUST_VERSION`).

### Changed
- Added previously-missing spec query parameters: dataset items `outputFields` (list/download)
  and `feedTitle`/`feedDescription` (download); key-value-store keys `collection`/`signature`;
  key-value-store record `signature` (via `GetRecordOptions`); request-queue requests
  `cursor`/`filter` (via `ListRequestsOptions`); storage collection list `unnamed`/`ownership`
  (via `StorageListOptions`); run collection list `startedAfter`/`startedBefore` (via
  `RunListOptions`).
- Added the `signature` query parameter to dataset items (`DatasetListItemsOptions`, inherited
  by `DatasetDownloadOptions`), so `list_items` / `download_items` can fetch items from a
  private dataset using a pre-shared signature (spec + JS reference both expose it).
- `get_record_with_options` signature changed to take `GetRecordOptions` (was `attachment: bool`),
  exposing the spec `signature` param for reading records from private stores.
- `ActorClient::default_build` now takes a `wait_for_finish: Option<i64>` argument (matching the
  reference client's `defaultBuild(options)`), optionally bounding how long the API waits for the
  default build to finish.
- Backoff doubling factor extracted to a named constant.
- Cross-client consistency with the JS reference and the OpenAPI spec (aligning with the Go
  sibling):
  - `RunClient::charge` now takes `RunChargeOptions { event_name, count, idempotency_key }` and
    always sends an `idempotency-key` header (auto-generated as
    `{runId}-{eventName}-{millis}-{random}` when omitted), so a transport-retried charge is
    applied at most once. (Was `charge(event_name, count)` with no idempotency key.)
  - `RunClient::metamorph` now takes `RunMetamorphOptions { build, content_type }`, letting the
    caller set the input body content type (defaults to `application/json`). (Was
    `metamorph(target, input, build)`.)
  - `RunResurrectOptions` gained `max_items`, `max_total_charge_usd`, and `restart_on_error`
    (all declared by `POST /v2/actor-runs/{runId}/resurrect` and supported by the JS reference).
  - `RunListOptions::status` is now `Vec<String>` (sent comma-separated), so multiple run
    statuses can be filtered in one call, matching the spec's array `status` parameter. (Was a
    single `Option<String>`.)
  - `RunClient::abort` now takes `gracefully: Option<bool>` instead of `bool`. Passing `None`
    omits the `gracefully` query parameter (letting the server apply its default, immediate
    abort), matching the reference client's optional `gracefully` option and the Go sibling.
  - `RequestQueueClient::batch_add_requests` now splits inputs larger than the API's 25-per-call
    limit into chunks and merges the per-chunk `processedRequests`/`unprocessedRequests` results,
    matching the reference client's client-side chunking. (Was a single raw POST of the whole
    slice, which could exceed API limits for large batches.)

### Notes
- A few documented endpoints are intentionally not exposed (matching the JS reference):
  synchronous run endpoints, `/tools/*`, `/browser-info`, and the keyed-`POST` create variants
  for Actor versions and version env-vars (creation is via `POST` to the collection, upsert via
  `PUT` on the keyed path).
- The JS `listItems` `chunkSize` option is intentionally not exposed: it is a client-side
  hint controlling the per-request page size of the JS async-iterator, not an API query
  parameter. The Rust `list_items` returns a single `PaginationList` page (the caller controls
  the page size via `limit`), so `chunkSize` has no analogue here.
- The `POST` store-record alias `POST /v2/key-value-stores/{storeId}/records/{recordKey}` is
  intentionally not exposed: the spec defines it as behaving identically to the covered `PUT`
  variant, and the reference client stores records via `PUT` only. Records are stored with
  `set_record_raw` / `set_record_json`.
