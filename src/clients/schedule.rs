//! Client for a single schedule (`/v2/schedules/{scheduleId}`).

use serde::Serialize;

use crate::clients::base::{
    delete_resource, get_raw_required, get_resource, update_resource, ResourceContext,
};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Schedule;
use crate::timeouts::Timeout;

/// Client for a specific schedule.
#[derive(Debug, Clone)]
pub struct ScheduleClient {
    ctx: ResourceContext,
}

crate::clients::base::impl_with_timeout!(ScheduleClient);

impl ScheduleClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "schedules", id),
        }
    }

    /// Fetches the schedule, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Schedule>> {
        get_resource(&self.ctx, None, &QueryParams::new(), Timeout::Short).await
    }

    /// Updates the schedule with the given fields.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<Schedule> {
        update_resource(&self.ctx, None, new_fields, Timeout::Short).await
    }

    /// Deletes the schedule.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None, Timeout::Short).await
    }

    /// Fetches the schedule's invocation log as text.
    ///
    /// A schedule that does not exist is a [`NotFound`](crate::ApiError::is_not_found) error:
    /// the log is a fixed sub-path of the schedule, so a `404` cannot be told apart from a
    /// missing log.
    pub async fn get_log(&self) -> ApifyClientResult<String> {
        let response =
            get_raw_required(&self.ctx, Some("log"), &QueryParams::new(), Timeout::Medium).await?;
        Ok(String::from_utf8_lossy(&response.body).into_owned())
    }
}
