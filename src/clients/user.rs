//! Client for a user (`/v2/users/{userId}` or `/v2/users/me`).

use serde::Serialize;
use serde_json::Value;

use crate::clients::base::{get_resource, get_resource_required, ResourceContext};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
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

    /// Returns the current user's monthly usage. Only valid for the `me` client.
    pub async fn monthly_usage(&self) -> ApifyClientResult<Value> {
        self.require_me("monthly_usage")?;
        get_resource_required(&self.ctx, Some("usage/monthly"), &QueryParams::new()).await
    }

    /// Returns the current user's account and usage limits. Only valid for the `me` client.
    pub async fn limits(&self) -> ApifyClientResult<Value> {
        self.require_me("limits")?;
        get_resource_required(&self.ctx, Some("limits"), &QueryParams::new()).await
    }

    /// Updates the current user's limits. Only valid for the `me` client.
    pub async fn update_limits<T: Serialize>(&self, new_limits: &T) -> ApifyClientResult<()> {
        self.require_me("update_limits")?;
        let body = serde_json::to_vec(new_limits)?;
        let url = self.ctx.url(Some("limits"));
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.ctx
            .http
            .call(crate::http_client::HttpRequest {
                method: crate::http_client::HttpMethod::Put,
                url,
                headers,
                body: Some(body),
                timeout: crate::clients::base::DEFAULT_REQUEST_TIMEOUT,
            })
            .await?;
        Ok(())
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
