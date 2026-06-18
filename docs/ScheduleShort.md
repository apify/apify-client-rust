# ScheduleShort

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**user_id** | **String** |  | 
**name** | **String** |  | 
**cron_expression** | **String** |  | 
**timezone** | **String** |  | 
**is_enabled** | **bool** |  | 
**is_exclusive** | **bool** |  | 
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**modified_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**next_run_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**last_run_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**actions** | [**Vec<models::ScheduleActionShort>**](ScheduleActionShort.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


