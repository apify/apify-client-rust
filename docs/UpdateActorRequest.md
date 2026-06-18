# UpdateActorRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | Option<**String**> |  | [optional]
**description** | Option<**String**> |  | [optional]
**is_public** | Option<**bool**> |  | [optional]
**actor_permission_level** | Option<[**models::ActorPermissionLevel**](ActorPermissionLevel.md)> |  | [optional]
**seo_title** | Option<**String**> |  | [optional]
**seo_description** | Option<**String**> |  | [optional]
**title** | Option<**String**> |  | [optional]
**restart_on_error** | Option<**bool**> |  | [optional]
**versions** | Option<[**Vec<models::CreateOrUpdateVersionRequest>**](CreateOrUpdateVersionRequest.md)> |  | [optional]
**pricing_infos** | Option<[**Vec<models::ActorRunPricingInfo>**](ActorRunPricingInfo.md)> |  | [optional]
**categories** | Option<**Vec<String>**> |  | [optional]
**default_run_options** | Option<[**models::DefaultRunOptions**](DefaultRunOptions.md)> |  | [optional]
**tagged_builds** | Option<[**std::collections::HashMap<String, models::BuildTag>**](BuildTag.md)> | An object to modify tags on the Actor's builds. The key is the tag name (e.g., _latest_), and the value is either an object with a `buildId` or `null`.  This operation is a patch; any existing tags that you omit from this object will be preserved.  - **To create or reassign a tag**, provide the tag name with a `buildId`. e.g., to assign the _latest_ tag:    &nbsp;    ```json   {     \"latest\": {       \"buildId\": \"z2EryhbfhgSyqj6Hn\"     }   }   ```  - **To remove a tag**, provide the tag name with a `null` value. e.g., to remove the _beta_ tag:    &nbsp;    ```json   {     \"beta\": null   }   ```  - **To perform multiple operations**, combine them. The following reassigns _latest_ and removes _beta_, while preserving any other existing tags.    &nbsp;    ```json   {     \"latest\": {       \"buildId\": \"z2EryhbfhgSyqj6Hn\"     },     \"beta\": null   }   ```  | [optional]
**actor_standby** | Option<[**models::ActorStandby**](ActorStandby.md)> |  | [optional]
**example_run_input** | Option<[**models::ExampleRunInput**](ExampleRunInput.md)> |  | [optional]
**is_deprecated** | Option<**bool**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


