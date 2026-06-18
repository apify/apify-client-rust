# BuildShort

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**act_id** | Option<**String**> |  | [optional]
**user_id** | Option<**String**> |  | [optional]
**status** | [**models::ActorJobStatus**](ActorJobStatus.md) |  | 
**started_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**finished_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**usage_total_usd** | **f64** |  | 
**build_number** | **String** |  | 
**build_number_int** | Option<**i32**> |  | [optional]
**meta** | Option<[**models::BuildsMeta**](BuildsMeta.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


