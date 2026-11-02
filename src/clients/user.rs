//! Client for a user (`/v2/users/{userId}` or `/v2/users/me`).

use serde::Serialize;
use serde_json::Value;

use crate::clients::base::{get_resource, get_resource_required, ResourceContext};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, HttpMethod};
use crate::models::User;
use crate::timeouts::Timeout;

/// Client for a specific user (or the current user via [`ApifyClient::me`]).
///
/// [`ApifyClient::me`]: crate::ApifyClient::me
#[derive(Debug, Clone)]
pub struct UserClient {
    ctx: ResourceContext,
}

crate::clients::base::impl_with_timeout!(UserClient);

impl UserClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "users", id),
        }
    }

    /// Fetches the user account information, or `None` if the user does not exist.
    ///
    /// For the current user (`me`) this returns private account details; for other users
    /// it returns the public profile.
    pub async fn get(&self) -> ApifyClientResult<Option<User>> {
        get_resource(&self.ctx, None, &QueryParams::new(), Timeout::Short).await
    }

    /// Returns the user's monthly usage for the current month. To fetch usage for a specific
    /// month, use [`UserClient::monthly_usage_for_date`].
    ///
    /// For a user other than the current one, only your own account and the organization your API
    /// token belongs to can be accessed.
    pub async fn monthly_usage(&self) -> ApifyClientResult<Value> {
        self.monthly_usage_for_date(None).await
    }

    /// Returns the user's monthly usage, optionally for the month containing `date`.
    ///
    /// `date` is an optional `YYYY-MM-DD` string selecting the month to report (the spec's
    /// optional `date` query parameter on `GET /v2/users/{userId}/usage/monthly`); passing `None`
    /// returns the current month, which is equivalent to [`UserClient::monthly_usage`].
    pub async fn monthly_usage_for_date(&self, date: Option<&str>) -> ApifyClientResult<Value> {
        let mut params = QueryParams::new();
        params.add_str("date", date);
        get_resource_required(&self.ctx, Some("usage/monthly"), &params, Timeout::Short).await
    }

    /// Returns the user's account and usage limits.
    ///
    /// For a user other than the current one, only your own account and the organization your API
    /// token belongs to can be accessed.
    pub async fn limits(&self) -> ApifyClientResult<Value> {
        get_resource_required(
            &self.ctx,
            Some("limits"),
            &QueryParams::new(),
            Timeout::Short,
        )
        .await
    }

    /// Updates the user's limits.
    ///
    /// For a user other than the current one, only your own account and the organization your API
    /// token belongs to can be accessed.
    pub async fn update_limits<T: Serialize>(&self, new_limits: &T) -> ApifyClientResult<()> {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        // The endpoint's response body carries nothing, so it is ignored rather than parsed.
        self.ctx
            .send_with(
                HttpMethod::Put,
                self.ctx.url(Some("limits")),
                headers,
                Some(serde_json::to_vec(new_limits)?),
                self.ctx.timeout(Timeout::Short),
            )
            .await?;
        Ok(())
    }
}
