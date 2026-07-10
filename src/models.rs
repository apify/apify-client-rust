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
    /// Any other fields returned by the API.
    #[serde(flatten)]
    pub extra: Extra,
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
