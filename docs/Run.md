# Run

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Unique identifier of the Actor run. | 
**act_id** | **String** | ID of the Actor that was run. | 
**user_id** | **String** | ID of the user who started the run. | 
**actor_task_id** | Option<**String**> | ID of the Actor task, if the run was started from a task. | [optional]
**started_at** | **chrono::DateTime<chrono::FixedOffset>** | Time when the Actor run started. | 
**finished_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Time when the Actor run finished. | [optional]
**status** | [**models::ActorJobStatus**](ActorJobStatus.md) | Current status of the Actor run. | 
**status_message** | Option<**String**> | Detailed message about the run status. | [optional]
**is_status_message_terminal** | Option<**bool**> | Whether the status message is terminal (final). | [optional]
**meta** | [**models::RunMeta**](RunMeta.md) | Metadata about the Actor run. | 
**pricing_info** | Option<[**models::ActorRunPricingInfo**](ActorRunPricingInfo.md)> | Pricing information for the Actor. | [optional]
**stats** | [**models::RunStats**](RunStats.md) | Statistics of the Actor run. | 
**charged_event_counts** | Option<**std::collections::HashMap<String, i32>**> | A map of charged event types to their counts. The keys are event type identifiers defined by the Actor's pricing model (pay-per-event), and the values are the number of times each event was charged during this run. | [optional]
**options** | [**models::RunOptions**](RunOptions.md) | Configuration options for the Actor run. | 
**build_id** | **String** | ID of the Actor build used for this run. | 
**exit_code** | Option<**i32**> | Exit code of the Actor run process. | [optional]
**general_access** | [**models::GeneralAccess**](GeneralAccess.md) | General access level for the Actor run. | 
**default_key_value_store_id** | **String** | ID of the default key-value store for this run. | 
**default_dataset_id** | **String** | ID of the default dataset for this run. | 
**default_request_queue_id** | **String** | ID of the default request queue for this run. | 
**storage_ids** | Option<[**models::RunStorageIds**](RunStorageIds.md)> |  | [optional]
**build_number** | Option<**String**> | Build number of the Actor build used for this run. | [optional]
**container_url** | Option<**String**> | URL of the container running the Actor. | [optional]
**is_container_server_ready** | Option<**bool**> | Whether the container's HTTP server is ready to accept requests. | [optional]
**git_branch_name** | Option<**String**> | Name of the git branch used for the Actor build. | [optional]
**usage** | Option<[**models::RunUsage**](RunUsage.md)> | Resource usage statistics for the run. | [optional]
**usage_total_usd** | Option<**f64**> | Total cost in USD for this run. Represents what you actually pay. For run owners: includes platform usage (compute units) and/or event costs depending on the Actor's pricing model. For run non-owners: only available for Pay-Per-Event Actors (event costs only). Requires authentication token to access. | [optional]
**usage_usd** | Option<[**models::RunUsageUsd**](RunUsageUsd.md)> | Platform usage costs breakdown in USD. Only present if you own the run AND are paying for platform usage (Pay-Per-Usage, Rental, or Pay-Per-Event with usage costs like standby Actors). Not available for standard Pay-Per-Event Actors. Requires authentication token to access. | [optional]
**metamorphs** | Option<[**Vec<models::Metamorph>**](Metamorph.md)> | List of metamorph events that occurred during the run. | [optional]
**platform_usage_billing_model** | Option<**String**> | Indicates which party covers platform usage costs for this run. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


