//! Client for the Actor collection endpoint (`/v2/actors`).

use serde::Serialize;

use crate::clients::base::{create_resource, list_resource, ResourceContext};
use crate::common::{PaginationList, QueryParams};
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Actor;

/// Options for listing Actors (`GET /v2/actors`).
#[derive(Debug, Default, Clone)]
pub struct ActorListOptions {
    /// Number of Actors to skip.
    pub offset: Option<i64>,
    /// Maximum number of Actors to return.
    pub limit: Option<i64>,
    /// Return Actors newest-first.
    pub desc: Option<bool>,
    /// If `true`, return only Actors owned by the current user.
    pub my: Option<bool>,
    /// Sort key, e.g. `createdAt` or `stats.lastRunStartedAt`.
    pub sort_by: Option<String>,
}

/// Client for listing and creating Actors.
#[derive(Debug, Clone)]
pub struct ActorCollectionClient {
    ctx: ResourceContext,
}

impl ActorCollectionClient {
    pub(crate) fn new(http: HttpClient, base_url: &str) -> Self {
        Self {
            ctx: ResourceContext::collection(http, base_url, "actors"),
        }
    }

    /// Lists the Actors in your account.
    ///
    /// Use [`ActorListOptions::my`] to restrict the result to Actors you own.
    pub async fn list(
        &self,
        options: ActorListOptions,
    ) -> ApifyClientResult<PaginationList<Actor>> {
        let mut params = QueryParams::new();
        params
            .add_int("offset", options.offset)
            .add_int("limit", options.limit)
            .add_bool("desc", options.desc)
            .add_bool("my", options.my)
            .add_str("sortBy", options.sort_by);
        list_resource(&self.ctx, None, &params).await
    }

    /// Creates a new Actor with the given definition.
    ///
    /// `actor` is any JSON-serializable Actor definition (at minimum a `name`).
    pub async fn create<T: Serialize>(&self, actor: &T) -> ApifyClientResult<Actor> {
        create_resource(&self.ctx, &QueryParams::new(), actor).await
    }
}
