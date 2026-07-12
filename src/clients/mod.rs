//! Resource clients.
//!
//! Each submodule provides a strongly-typed client for one Apify API resource. Clients
//! are obtained from [`ApifyClient`](crate::ApifyClient) accessor methods rather than
//! constructed directly, so their constructors are crate-private.

pub(crate) mod base;

pub mod actor;
pub mod actor_collection;
pub mod actor_env_var;
pub mod actor_env_var_collection;
pub mod actor_version;
pub mod actor_version_collection;
pub mod build;
pub mod build_collection;
pub mod dataset;
pub mod dataset_collection;
pub mod key_value_store;
pub mod key_value_store_collection;
pub mod log;
pub mod pagination;
pub mod request_queue;
pub mod request_queue_collection;
pub mod run;
pub mod run_collection;
pub mod schedule;
pub mod schedule_collection;
pub mod store_collection;
pub mod task;
pub mod task_collection;
pub mod user;
pub mod webhook;
pub mod webhook_collection;
pub mod webhook_dispatch;
pub mod webhook_dispatch_collection;
