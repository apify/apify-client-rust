//! Client for a user (`/v2/users/{userId}` or `/v2/users/me`).

use serde::Serialize;
use serde_json::Value;

use crate::clients::base::{get_resource, put_raw, ResourceContext};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::{HttpClient, CONTENT_TYPE_JSON};
use crate::models::User;

/// Client for a specific user (or the current user via [`ApifyClient::me`]).
///
/// [`ApifyClient::me`]: crate::ApifyClient::me
#[derive(Debug, Clone)]
pub struct UserClient {
    ctx: ResourceContext,
    is_me: bool,
}

impl UserClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "users", id),
            is_me: id == crate::client::ME_USER_PLACEHOLDER,
        }
    }

    /// Fetches the user account information.
    ///
    /// For the current user (`me`) this returns private account details; for other users
    /// it returns the public profile.
    pub async fn get(&self) -> ApifyClientResult<Option<User>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Returns the current user's monthly usage for the current month, or `None` if it is
    /// unavailable. Only valid for the `me` client. To fetch usage for a specific month, use
    /// [`UserClient::monthly_usage_for_date`].
    pub async fn monthly_usage(&self) -> ApifyClientResult<Option<Value>> {
        self.monthly_usage_for_date_named(None, "monthly_usage")
            .await
    }

    /// Returns the current user's monthly usage, optionally for the month containing `date`, or
    /// `None` if it is unavailable.
    ///
    /// `date` is an optional `YYYY-MM-DD` string selecting the month to report (the spec's
    /// optional `date` query parameter on `GET /v2/users/me/usage/monthly`); passing `None`
    /// returns the current month, which is equivalent to [`UserClient::monthly_usage`]. Only
    /// valid for the `me` client.
    pub async fn monthly_usage_for_date(
        &self,
        date: Option<&str>,
    ) -> ApifyClientResult<Option<Value>> {
        self.monthly_usage_for_date_named(date, "monthly_usage_for_date")
            .await
    }

    /// Shared implementation for [`UserClient::monthly_usage`] and
    /// [`UserClient::monthly_usage_for_date`]. `method` is the caller's own public method name,
    /// so the `me`-only guard error names the method the caller actually invoked.
    ///
    /// The spec declares no `404` response for `GET /v2/users/me/usage/monthly`, so a `404`
    /// mapping to `None` here would never actually trigger against the real API; it is used
    /// anyway (via [`get_resource`] rather than a `_required` variant) purely for JS-reference
    /// parity, since the reference client's `monthlyUsage` wraps the call in
    /// `catchNotFoundOrThrow`.
    async fn monthly_usage_for_date_named(
        &self,
        date: Option<&str>,
        method: &str,
    ) -> ApifyClientResult<Option<Value>> {
        self.require_me(method)?;
        let mut params = QueryParams::new();
        params.add_str("date", date);
        get_resource(&self.ctx, Some("usage/monthly"), &params).await
    }

    /// Returns the current user's account and usage limits, or `None` if unavailable. Only
    /// valid for the `me` client.
    ///
    /// As with [`UserClient::monthly_usage`], the spec declares no `404` for
    /// `GET /v2/users/me/limits`; the `Option` return is purely for JS-reference parity
    /// (`limits()` there also wraps the call in `catchNotFoundOrThrow`).
    pub async fn limits(&self) -> ApifyClientResult<Option<Value>> {
        self.require_me("limits")?;
        get_resource(&self.ctx, Some("limits"), &QueryParams::new()).await
    }

    /// Updates the current user's limits. Only valid for the `me` client.
    pub async fn update_limits<T: Serialize>(&self, new_limits: &T) -> ApifyClientResult<()> {
        self.require_me("update_limits")?;
        let body = serde_json::to_vec(new_limits)?;
        put_raw(
            &self.ctx,
            Some("limits"),
            &QueryParams::new(),
            body,
            CONTENT_TYPE_JSON,
        )
        .await
    }

    fn require_me(&self, method: &str) -> ApifyClientResult<()> {
        if !self.is_me {
            return Err(crate::error::ApifyClientError::InvalidArgument(format!(
                "`{method}` is only available for the current user (use ApifyClient::me())"
            )));
        }
        Ok(())
    }
}
