//! Data models for Apify API resources.
//!
//! Each resource is modelled with the fields most commonly used by clients, mirroring
//! the reference JavaScript client. To remain forward-compatible with additive changes to
//! the API, none of the models set `deny_unknown_fields`, so unknown API fields are ignored
//! rather than breaking deserialization. Most resource models additionally capture any unknown
//! fields in an `extra` map via `#[serde(flatten)]` so they remain accessible to callers.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Convenience alias for the catch-all map of unmodelled JSON fields.
pub type Extra = HashMap<String, Value>;

/// An Actor on the Apify platform.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    /// Unique Actor ID.
    pub id: String,
    /// ID of the user who owns the Actor.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Technical name of the Actor (used in API paths).
    #[serde(default)]
    pub name: Option<String>,
    /// Username of the Actor's owner.
    #[serde(default)]
    pub username: Option<String>,
    /// Human-readable title shown in the UI.
    #[serde(default)]
    pub title: Option<String>,
    /// Description of what the Actor does.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether the Actor is publicly available in Apify Store.
    #[serde(default)]
    pub is_public: Option<bool>,
    /// When the Actor was created.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// When the Actor was last modified.
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A single execution of an Actor (an Actor run).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorRun {
    /// Unique run ID.
    pub id: String,
    /// ID of the Actor that produced this run.
    #[serde(default)]
    pub act_id: Option<String>,
    /// ID of the task that started this run, if any.
    #[serde(default)]
    pub actor_task_id: Option<String>,
    /// ID of the user who owns the run.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Current run status, e.g. `READY`, `RUNNING`, `SUCCEEDED`, `FAILED`, `ABORTED`, `TIMED-OUT`.
    #[serde(default)]
    pub status: Option<String>,
    /// Optional human-readable status message.
    #[serde(default)]
    pub status_message: Option<String>,
    /// When the run started.
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    /// When the run finished (absent while still running).
    #[serde(default)]
    pub finished_at: Option<DateTime<Utc>>,
    /// ID of the build used for the run.
    #[serde(default)]
    pub build_id: Option<String>,
    /// Default dataset ID associated with the run.
    #[serde(default)]
    pub default_dataset_id: Option<String>,
    /// Default key-value store ID associated with the run.
    #[serde(default)]
    pub default_key_value_store_id: Option<String>,
    /// Default request queue ID associated with the run.
    #[serde(default)]
    pub default_request_queue_id: Option<String>,
    /// URL of the run's container, if running.
    #[serde(default)]
    pub container_url: Option<String>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Terminal run/build statuses, used by wait-for-finish helpers.
pub(crate) const TERMINAL_STATUSES: &[&str] =
    &["SUCCEEDED", "FAILED", "ABORTED", "TIMED-OUT", "TIMED_OUT"];

impl ActorRun {
    /// Returns `true` if the run has reached a terminal state.
    pub fn is_terminal(&self) -> bool {
        self.status
            .as_deref()
            .map(|s| TERMINAL_STATUSES.contains(&s))
            .unwrap_or(false)
    }
}

/// A build of an Actor.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Build {
    /// Unique build ID.
    pub id: String,
    /// ID of the Actor that was built.
    #[serde(default)]
    pub act_id: Option<String>,
    /// Current build status.
    #[serde(default)]
    pub status: Option<String>,
    /// When the build started.
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    /// When the build finished.
    #[serde(default)]
    pub finished_at: Option<DateTime<Utc>>,
    /// Build number, e.g. `0.1.2`.
    #[serde(default)]
    pub build_number: Option<String>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

impl Build {
    /// Returns `true` if the build has reached a terminal state.
    pub fn is_terminal(&self) -> bool {
        self.status
            .as_deref()
            .map(|s| TERMINAL_STATUSES.contains(&s))
            .unwrap_or(false)
    }
}

/// An Actor task (a saved, reusable Actor configuration).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Unique task ID.
    pub id: String,
    /// ID of the Actor this task runs.
    #[serde(default)]
    pub act_id: Option<String>,
    /// ID of the user who owns the task.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Technical name of the task.
    #[serde(default)]
    pub name: Option<String>,
    /// Human-readable title.
    #[serde(default)]
    pub title: Option<String>,
    /// When the task was created.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// When the task was last modified.
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
    /// Whether the task is published on its public landing page. Derived from
    /// `public_config.published_at`; set it via [`TaskClient::update`](crate::clients::task::TaskClient::update)
    /// (or the [`publish`](crate::clients::task::TaskClient::publish)/
    /// [`unpublish`](crate::clients::task::TaskClient::unpublish) wrappers) to change it.
    #[serde(default)]
    pub is_public: Option<bool>,
    /// The task's public landing page display configuration, or `None` if never configured.
    #[serde(default)]
    pub public_config: Option<TaskPublicConfig>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Public-facing display configuration of a task's public landing page.
///
/// The task is published when `published_at` is set and unpublished when it is `None`.
/// `published_at` is server-controlled (read-only) - use
/// [`TaskClient::publish`](crate::clients::task::TaskClient::publish) /
/// [`TaskClient::unpublish`](crate::clients::task::TaskClient::unpublish) to change the
/// publication state.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPublicConfig {
    /// When the task was published, or `None` if it isn't published. Read-only.
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
    /// Name shown by search engines. Defaults to the task title when unset.
    #[serde(default)]
    pub seo_title: Option<String>,
    /// Description shown by search engines. Defaults to the task description when unset.
    #[serde(default)]
    pub seo_description: Option<String>,
    /// Names of the task input fields displayed on the public task page.
    #[serde(default)]
    pub input_schema_fields: Option<Vec<String>>,
    /// Name of the Actor dataset schema entry whose results are displayed. `None` uses the
    /// Actor's default dataset.
    #[serde(default)]
    pub dataset_name: Option<String>,
    /// Key of the dataset view (from the Actor's dataset schema) used to display results.
    /// Required to publish the task.
    #[serde(default)]
    pub dataset_view: Option<String>,
}

/// A dataset storage.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    /// Unique dataset ID.
    pub id: String,
    /// Technical name of the dataset, if named.
    #[serde(default)]
    pub name: Option<String>,
    /// ID of the owner.
    #[serde(default)]
    pub user_id: Option<String>,
    /// When the dataset was created.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// When the dataset was last modified.
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
    /// Total number of items in the dataset.
    #[serde(default)]
    pub item_count: Option<i64>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A key-value store storage.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValueStore {
    /// Unique store ID.
    pub id: String,
    /// Technical name of the store, if named.
    #[serde(default)]
    pub name: Option<String>,
    /// ID of the owner.
    #[serde(default)]
    pub user_id: Option<String>,
    /// When the store was created.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// When the store was last modified.
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Metadata about a single key in a key-value store.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValueStoreKey {
    /// The record key.
    pub key: String,
    /// Size of the record value in bytes.
    #[serde(default)]
    pub size: Option<i64>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Result of listing keys in a key-value store (key-based pagination).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValueStoreKeysPage {
    /// Maximum number of keys returned for this request.
    #[serde(default)]
    pub limit: i64,
    /// Whether there are more keys to fetch.
    #[serde(default)]
    pub is_truncated: bool,
    /// The key the listing started after.
    #[serde(default)]
    pub exclusive_start_key: Option<String>,
    /// The value to use as `exclusive_start_key` for the next page.
    #[serde(default)]
    pub next_exclusive_start_key: Option<String>,
    /// The keys of this page.
    #[serde(default)]
    pub items: Vec<KeyValueStoreKey>,
}

/// A record (key + value + content type) in a key-value store.
#[derive(Debug, Clone)]
pub struct KeyValueStoreRecord {
    /// The record key.
    pub key: String,
    /// The raw value bytes.
    pub value: Vec<u8>,
    /// The MIME content type of the value, if reported by the API.
    pub content_type: Option<String>,
}

impl KeyValueStoreRecord {
    /// Interprets the value as UTF-8 text.
    pub fn as_text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.value.clone())
    }

    /// Deserializes the value as JSON into `T`.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.value)
    }
}

/// A request queue storage.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestQueue {
    /// Unique queue ID.
    pub id: String,
    /// Technical name of the queue, if named.
    #[serde(default)]
    pub name: Option<String>,
    /// ID of the owner.
    #[serde(default)]
    pub user_id: Option<String>,
    /// When the queue was created.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// When the queue was last modified.
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,
    /// Total number of requests ever added.
    #[serde(default)]
    pub total_request_count: Option<i64>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A single request stored in a request queue.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestQueueRequest {
    /// Unique request ID (assigned by the API; omit when adding a new request).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The URL to be processed.
    pub url: String,
    /// Unique key used for deduplication (defaults to `url`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_key: Option<String>,
    /// HTTP method, defaults to `GET`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Arbitrary user data attached to the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_data: Option<Value>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Result of adding (or updating) a request in a queue.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestQueueOperationInfo {
    /// ID of the request that was added or updated.
    pub request_id: String,
    /// Whether the request was already present in the queue.
    #[serde(default)]
    pub was_already_present: bool,
    /// Whether the request had already been handled.
    #[serde(default)]
    pub was_already_handled: bool,
}

/// The head of a request queue (requests waiting to be processed).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestQueueHead {
    /// Maximum number of requests returned.
    #[serde(default)]
    pub limit: i64,
    /// Whether more than one client has accessed the queue.
    #[serde(default)]
    pub had_multiple_clients: bool,
    /// The requests at the head of the queue.
    #[serde(default)]
    pub items: Vec<RequestQueueRequest>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The head of a request queue with a server-side lock applied (`POST .../head/lock`).
///
/// Same shape as [`RequestQueueHead`] plus the lock metadata the API returns alongside it.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedRequestQueueHead {
    /// Maximum number of requests returned.
    #[serde(default)]
    pub limit: i64,
    /// Whether more than one client has accessed the queue.
    #[serde(default)]
    pub had_multiple_clients: bool,
    /// Number of seconds the returned requests were locked for.
    #[serde(default)]
    pub lock_secs: i64,
    /// Whether the queue has any requests locked, by this or another client.
    #[serde(default)]
    pub queue_has_locked_requests: Option<bool>,
    /// The client key the lock was acquired with.
    #[serde(default)]
    pub client_key: Option<String>,
    /// The locked requests from the head of the queue.
    #[serde(default)]
    pub items: Vec<RequestQueueRequest>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A page of requests returned by `GET /v2/request-queues/{queueId}/requests` (cursor-based
/// pagination over every request in the queue, as opposed to [`RequestQueueHead`]'s unlocked
/// peek at the head).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestQueueRequestsPage {
    /// Maximum number of requests returned for this request.
    #[serde(default)]
    pub limit: i64,
    /// ID of the last request of the previous page. Deprecated by the API in favour of `cursor`.
    #[serde(default)]
    pub exclusive_start_id: Option<String>,
    /// Cursor identifying the current page.
    #[serde(default)]
    pub cursor: Option<String>,
    /// Cursor to pass as `cursor` to fetch the next page; absent on the last page.
    #[serde(default)]
    pub next_cursor: Option<String>,
    /// The requests of this page.
    #[serde(default)]
    pub items: Vec<RequestQueueRequest>,
}

/// Result of prolonging a request's lock (`PUT .../requests/{requestId}/lock`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestLockInfo {
    /// When the (prolonged) lock expires.
    pub lock_expires_at: DateTime<Utc>,
}

/// Result of `POST /v2/request-queues/{queueId}/requests/unlock`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockRequestsResult {
    /// Number of requests that were unlocked.
    pub unlocked_count: i64,
}

/// A request successfully processed by a request-queue batch add or batch delete operation.
///
/// The populated fields depend on the operation: a batch **add** always sets `unique_key`,
/// `request_id`, `was_already_present` and `was_already_handled` (the API's `AddedRequest`); a
/// batch **delete** sets `id` and/or `unique_key`, whichever the request was identified by (the
/// API's `DeletedRequest`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedRequest {
    /// The request's ID, when the operation identifies requests by ID (batch delete).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The request's unique key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_key: Option<String>,
    /// The request's ID, as returned by a batch **add** (mirrors the API's `requestId`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Whether the request was already present in the queue (batch add only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub was_already_present: Option<bool>,
    /// Whether the request had already been handled (batch add only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub was_already_handled: Option<bool>,
}

/// A request that a request-queue batch add operation did not process (typically due to rate
/// limiting), and which [`crate::clients::request_queue::RequestQueueClient::batch_add_requests`]
/// retries automatically.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnprocessedRequest {
    /// The request's unique key.
    pub unique_key: String,
    /// The request's URL.
    pub url: String,
    /// The request's HTTP method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

/// Result of a request-queue batch add or batch delete operation.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequestsOperationResult {
    /// Requests that were successfully processed.
    #[serde(default)]
    pub processed_requests: Vec<ProcessedRequest>,
    /// Requests that were not processed and can be retried.
    #[serde(default)]
    pub unprocessed_requests: Vec<UnprocessedRequest>,
}

/// A schedule that triggers Actor or task runs on a cron expression.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    /// Unique schedule ID.
    pub id: String,
    /// ID of the owner.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Technical name of the schedule.
    #[serde(default)]
    pub name: Option<String>,
    /// The cron expression that determines when the schedule fires.
    #[serde(default)]
    pub cron_expression: Option<String>,
    /// Whether the schedule is currently enabled.
    #[serde(default)]
    pub is_enabled: Option<bool>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A webhook that notifies an external URL on Actor events.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    /// Unique webhook ID.
    pub id: String,
    /// ID of the owner.
    #[serde(default)]
    pub user_id: Option<String>,
    /// The URL that receives the webhook POST request.
    #[serde(default)]
    pub request_url: Option<String>,
    /// Event types that trigger this webhook.
    #[serde(default)]
    pub event_types: Vec<String>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A single dispatch (invocation) of a webhook.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookDispatch {
    /// Unique dispatch ID.
    pub id: String,
    /// ID of the webhook that produced this dispatch.
    #[serde(default)]
    pub webhook_id: Option<String>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Account information about a user.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// Unique user ID.
    pub id: String,
    /// Username.
    #[serde(default)]
    pub username: Option<String>,
    /// Any other fields returned by the API (public or private depending on the call).
    #[serde(flatten)]
    pub extra: Extra,
}

/// A single Actor entry as returned by the Apify Store listing.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorStoreListItem {
    /// Unique Actor ID.
    pub id: String,
    /// Technical name of the Actor.
    #[serde(default)]
    pub name: Option<String>,
    /// Username of the Actor's owner.
    #[serde(default)]
    pub username: Option<String>,
    /// Human-readable title.
    #[serde(default)]
    pub title: Option<String>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An Actor version.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorVersion {
    /// The version number, e.g. `0.1`.
    pub version_number: String,
    /// The source type of the version, e.g. `SOURCE_FILES`, `GIT_REPO`, `TARBALL`, `GITHUB_GIST`.
    #[serde(default)]
    pub source_type: Option<String>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An environment variable attached to an Actor version.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorEnvVar {
    /// The environment variable name.
    pub name: String,
    /// The value (may be omitted for secret variables in responses).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Whether the variable is a secret.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_secret: Option<bool>,
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
}
