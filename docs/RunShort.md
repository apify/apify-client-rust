# RunShort

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**act_id** | **String** |  | 
**user_id** | Option<**String**> |  | [optional]
**actor_task_id** | Option<**String**> |  | [optional]
**status** | [**models::ActorJobStatus**](ActorJobStatus.md) |  | 
**started_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**finished_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**build_id** | **String** |  | 
**build_number** | Option<**String**> |  | [optional]
**build_number_int** | Option<**i32**> |  | [optional]
**meta** | [**models::RunMeta**](RunMeta.md) |  | 
**usage_total_usd** | **f64** |  | 
**default_key_value_store_id** | **String** |  | 
**default_dataset_id** | **String** |  | 
**default_request_queue_id** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


