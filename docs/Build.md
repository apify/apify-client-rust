# Build

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**act_id** | **String** |  | 
**user_id** | **String** |  | 
**started_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**finished_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**status** | [**models::ActorJobStatus**](ActorJobStatus.md) |  | 
**meta** | [**models::BuildsMeta**](BuildsMeta.md) |  | 
**stats** | Option<[**models::BuildStats**](BuildStats.md)> |  | [optional]
**options** | Option<[**models::BuildOptions**](BuildOptions.md)> |  | [optional]
**usage** | Option<[**models::BuildUsage**](BuildUsage.md)> |  | [optional]
**usage_total_usd** | Option<**f64**> | Total cost in USD for this build. Requires authentication token to access. | [optional]
**usage_usd** | Option<[**models::BuildUsage**](BuildUsage.md)> | Platform usage costs breakdown in USD for this build. Requires authentication token to access. | [optional]
**input_schema** | Option<**String**> |  | [optional]
**readme** | Option<**String**> |  | [optional]
**build_number** | **String** |  | 
**act_version** | Option<[**models::BuildActVersion**](BuildActVersion.md)> |  | [optional]
**actor_definition** | Option<[**models::ActorDefinition**](ActorDefinition.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


