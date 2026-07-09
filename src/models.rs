//! Data models for Apify API resources.
//!
//! Each resource is modelled with the fields most commonly used by clients, mirroring
//! the reference JavaScript client. To remain forward-compatible with additive changes
//! to the API, every model captures any unknown fields in an `extra` map via
//! `#[serde(flatten)]`, so new API fields never break deserialization.

use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Convenience alias for the catch-all map of unmodelled JSON fields.
pub type Extra = HashMap<String, Value>;

/// Lifecycle status of an Actor run or build.
///
/// Models the platform's job statuses as a typed enum rather than a bare string, so callers
/// can `match` on it exhaustively. Any status not known to this client version is preserved
/// verbatim in [`RunStatus::Other`], keeping deserialization forward-compatible with new
/// platform statuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    /// The run is initializing and has not started yet.
    Ready,
    /// The run is currently executing.
    Running,
    /// The run finished successfully.
    Succeeded,
    /// The run failed.
    Failed,
    /// The run is in the process of timing out.
    TimingOut,
    /// The run timed out.
    TimedOut,
    /// The run is in the process of being aborted.
    Aborting,
    /// The run was aborted.
    Aborted,
    /// A status not recognized by this client version, preserved verbatim.
    Other(String),
}

impl RunStatus {
    /// Returns the API wire representation of this status (e.g. `"SUCCEEDED"`, `"TIMED-OUT"`).
    pub fn as_str(&self) -> &str {
        match self {
            RunStatus::Ready => "READY",
            RunStatus::Running => "RUNNING",
            RunStatus::Succeeded => "SUCCEEDED",
            RunStatus::Failed => "FAILED",
            RunStatus::TimingOut => "TIMING-OUT",
            RunStatus::TimedOut => "TIMED-OUT",
            RunStatus::Aborting => "ABORTING",
            RunStatus::Aborted => "ABORTED",
            RunStatus::Other(value) => value.as_str(),
        }
    }

    /// Returns `true` if this is a terminal status — the run/build has finished and its status
    /// will not change further (`SUCCEEDED`, `FAILED`, `ABORTED`, `TIMED-OUT`).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            RunStatus::Succeeded | RunStatus::Failed | RunStatus::Aborted | RunStatus::TimedOut
        )
    }
}

impl From<&str> for RunStatus {
    fn from(value: &str) -> Self {
        match value {
            "READY" => RunStatus::Ready,
            "RUNNING" => RunStatus::Running,
            "SUCCEEDED" => RunStatus::Succeeded,
            "FAILED" => RunStatus::Failed,
            // Accept both the hyphenated wire form the API sends and the underscored spelling
            // used for the platform's status constant names, mapping both to one variant.
            "TIMING-OUT" | "TIMING_OUT" => RunStatus::TimingOut,
            "TIMED-OUT" | "TIMED_OUT" => RunStatus::TimedOut,
            "ABORTING" => RunStatus::Aborting,
            "ABORTED" => RunStatus::Aborted,
            other => RunStatus::Other(other.to_string()),
        }
    }
}

impl fmt::Display for RunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for RunStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RunStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(RunStatus::from(raw.as_str()))
    }
}

/// How an Actor run was started (the run's `meta.origin`).
///
/// Like [`RunStatus`], this is a typed enum with an [`RunOrigin::Other`] catch-all for values
/// not known to this client version. Used to filter the last run by origin (see
/// [`crate::clients::run::LastRunOptions`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOrigin {
    /// Started from the development UI.
    Development,
    /// Started from the Apify Console web UI.
    Web,
    /// Started via the API.
    Api,
    /// Started by a schedule.
    Scheduler,
    /// Started by an automated test.
    Test,
    /// Started by a webhook.
    Webhook,
    /// Started by another Actor.
    Actor,
    /// Started from the Apify CLI.
    Cli,
    /// Started by the Actor Standby mode.
    Standby,
    /// Started by a CI/CD pipeline.
    Ci,
    /// Started via the Model Context Protocol.
    Mcp,
    /// An origin not recognized by this client version, preserved verbatim.
    Other(String),
}

impl RunOrigin {
    /// Returns the API wire representation of this origin (e.g. `"API"`, `"SCHEDULER"`).
    pub fn as_str(&self) -> &str {
        match self {
            RunOrigin::Development => "DEVELOPMENT",
            RunOrigin::Web => "WEB",
            RunOrigin::Api => "API",
            RunOrigin::Scheduler => "SCHEDULER",
            RunOrigin::Test => "TEST",
            RunOrigin::Webhook => "WEBHOOK",
            RunOrigin::Actor => "ACTOR",
            RunOrigin::Cli => "CLI",
            RunOrigin::Standby => "STANDBY",
            RunOrigin::Ci => "CI",
            RunOrigin::Mcp => "MCP",
            RunOrigin::Other(value) => value.as_str(),
        }
    }
}

impl From<&str> for RunOrigin {
    fn from(value: &str) -> Self {
        match value {
            "DEVELOPMENT" => RunOrigin::Development,
            "WEB" => RunOrigin::Web,
            "API" => RunOrigin::Api,
            "SCHEDULER" => RunOrigin::Scheduler,
            "TEST" => RunOrigin::Test,
            "WEBHOOK" => RunOrigin::Webhook,
            "ACTOR" => RunOrigin::Actor,
            "CLI" => RunOrigin::Cli,
            "STANDBY" => RunOrigin::Standby,
            "CI" => RunOrigin::Ci,
            "MCP" => RunOrigin::Mcp,
            other => RunOrigin::Other(other.to_string()),
        }
    }
}

impl fmt::Display for RunOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for RunOrigin {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RunOrigin {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(RunOrigin::from(raw.as_str()))
    }
}

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
    /// Current run status (see [`RunStatus`]).
    #[serde(default)]
    pub status: Option<RunStatus>,
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

impl ActorRun {
    /// Returns `true` if the run has reached a terminal state (see [`RunStatus::is_terminal`]).
    pub fn is_terminal(&self) -> bool {
        self.status.as_ref().is_some_and(RunStatus::is_terminal)
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
    /// Current build status (see [`RunStatus`]).
    #[serde(default)]
    pub status: Option<RunStatus>,
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
    /// Returns `true` if the build has reached a terminal state (see [`RunStatus::is_terminal`]).
    pub fn is_terminal(&self) -> bool {
        self.status.as_ref().is_some_and(RunStatus::is_terminal)
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
