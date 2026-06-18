# Actor

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**user_id** | **String** |  | 
**name** | **String** |  | 
**username** | **String** |  | 
**description** | Option<**String**> |  | [optional]
**restart_on_error** | Option<**bool**> |  | [optional]
**is_public** | **bool** |  | 
**actor_permission_level** | Option<[**models::ActorPermissionLevel**](ActorPermissionLevel.md)> |  | [optional]
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**modified_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**stats** | [**models::ActorStats**](ActorStats.md) |  | 
**versions** | [**Vec<models::Version>**](Version.md) |  | 
**pricing_infos** | Option<[**Vec<models::ActorRunPricingInfo>**](ActorRunPricingInfo.md)> |  | [optional]
**default_run_options** | [**models::DefaultRunOptions**](DefaultRunOptions.md) |  | 
**example_run_input** | Option<[**models::ExampleRunInput**](ExampleRunInput.md)> |  | [optional]
**is_deprecated** | Option<**bool**> |  | [optional]
**deployment_key** | Option<**String**> |  | [optional]
**title** | Option<**String**> |  | [optional]
**tagged_builds** | Option<[**std::collections::HashMap<String, models::TaggedBuildsValue>**](TaggedBuildsValue.md)> | A dictionary mapping build tag names (e.g., \"latest\", \"beta\") to their build information. | [optional]
**actor_standby** | Option<[**models::ActorStandby**](ActorStandby.md)> |  | [optional]
**readme_summary** | Option<**String**> | A brief, LLM-generated readme summary | [optional]
**seo_title** | Option<**String**> |  | [optional]
**seo_description** | Option<**String**> |  | [optional]
**picture_url** | Option<**String**> |  | [optional]
**standby_url** | Option<**String**> |  | [optional]
**notice** | Option<**String**> |  | [optional]
**categories** | Option<**Vec<String>**> |  | [optional]
**is_critical** | Option<**bool**> |  | [optional]
**is_generic** | Option<**bool**> |  | [optional]
**is_source_code_hidden** | Option<**bool**> |  | [optional]
**has_no_dataset** | Option<**bool**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


