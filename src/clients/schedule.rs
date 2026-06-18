//! Client for a single schedule (`/v2/schedules/{scheduleId}`).

use serde::Serialize;

use crate::clients::base::{
    delete_resource, get_raw, get_resource, update_resource, ResourceContext,
};
use crate::common::QueryParams;
use crate::error::ApifyClientResult;
use crate::http_client::HttpClient;
use crate::models::Schedule;

/// Client for a specific schedule.
#[derive(Debug, Clone)]
pub struct ScheduleClient {
    ctx: ResourceContext,
}

impl ScheduleClient {
    pub(crate) fn new(http: HttpClient, base_url: &str, id: &str) -> Self {
        Self {
            ctx: ResourceContext::single(http, base_url, "schedules", id),
        }
    }

    /// Fetches the schedule, or `None` if it does not exist.
    pub async fn get(&self) -> ApifyClientResult<Option<Schedule>> {
        get_resource(&self.ctx, None, &QueryParams::new()).await
    }

    /// Updates the schedule with the given fields.
    pub async fn update<T: Serialize>(&self, new_fields: &T) -> ApifyClientResult<Schedule> {
        update_resource(&self.ctx, None, new_fields).await
    }

    /// Deletes the schedule.
    pub async fn delete(&self) -> ApifyClientResult<()> {
        delete_resource(&self.ctx, None).await
    }

    /// Fetches the schedule's invocation log as text, or `None` if not available.
    pub async fn get_log(&self) -> ApifyClientResult<Option<String>> {
        let response = get_raw(&self.ctx, Some("log"), &QueryParams::new()).await?;
        match response {
            Some(r) => Ok(Some(String::from_utf8_lossy(&r.body).into_owned())),
            None => Ok(None),
        }
    }
}
