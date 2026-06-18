//! # apify-client
//!
//! The official, idiomatic Rust client for the [Apify API](https://docs.apify.com/api/v2).
//!
//! It provides a resource-oriented interface that mirrors the official
//! [JavaScript](https://github.com/apify/apify-client-js) and Python clients: start from
//! an [`ApifyClient`], then drill down into resources (Actors, runs, datasets, key-value
//! stores, request queues, tasks, schedules, webhooks, the store, users and logs).
//!
//! ## Quick start
//!
//! ```no_run
//! use apify_client::ApifyClient;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ApifyClient::new("my-api-token");
//!
//! // Start an Actor and wait for it to finish.
//! let run = client
//!     .actor("apify/hello-world")
//!     .call::<serde_json::Value>(None, Default::default(), None)
//!     .await?;
//!
//! // Read items from the run's default dataset.
//! if let Some(dataset_id) = &run.default_dataset_id {
//!     let items = client
//!         .dataset(dataset_id)
//!         .list_items::<serde_json::Value>(Default::default())
//!         .await?;
//!     println!("Got {} items", items.items.len());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! - **Public interface**: [`ApifyClient`] and the resource clients in [`clients`].
//! - **Replaceable transport**: the [`http_client::HttpBackend`] trait, with a default
//!   [`http_client::ReqwestBackend`]. Swap it via
//!   [`ApifyClientBuilder::http_backend`].
//! - **Cross-cutting behaviour** (auth, `User-Agent`, retries with exponential backoff,
//!   timeouts) lives in [`http_client::HttpClient`] and is applied to every request.

#![warn(missing_docs)]

mod client;
pub mod clients;
pub mod common;
pub mod error;
pub mod http_client;
pub mod models;
mod version;

pub use client::{ApifyClient, ApifyClientBuilder};
pub use common::{ListOptions, PaginationList, QueryParams, StorageListOptions};
pub use error::{ApiError, ApifyClientError, ApifyClientResult};
pub use version::{API_SPEC_VERSION, CLIENT_VERSION};

// Re-export the most commonly used option/parameter types for ergonomic access.
pub use clients::actor::{ActorBuildOptions, ActorStartOptions};
pub use clients::actor_collection::ActorListOptions;
pub use clients::dataset::{DatasetDownloadOptions, DatasetListItemsOptions, DownloadItemsFormat};
pub use clients::key_value_store::{GetRecordOptions, GetRecordsOptions, ListKeysOptions};
pub use clients::request_queue::ListRequestsOptions;
pub use clients::run::RunResurrectOptions;
pub use clients::run_collection::RunListOptions;
pub use clients::store_collection::StoreListOptions;

// Compile-test the code snippets in the README so the documentation stays valid.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;
