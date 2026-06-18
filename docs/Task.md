# Task

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**user_id** | **String** |  | 
**act_id** | **String** |  | 
**name** | **String** |  | 
**username** | Option<**String**> |  | [optional]
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**modified_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**removed_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**stats** | Option<[**models::TaskStats**](TaskStats.md)> |  | [optional]
**options** | Option<[**models::TaskOptions**](TaskOptions.md)> |  | [optional]
**input** | Option<**std::collections::HashMap<String, serde_json::Value>**> | The input configuration for the Actor task. This is a user-defined JSON object that will be passed to the Actor when the task is run.  | [optional]
**title** | Option<**String**> |  | [optional]
**actor_standby** | Option<[**models::ActorStandby**](ActorStandby.md)> |  | [optional]
**standby_url** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


