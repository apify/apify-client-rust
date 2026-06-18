# CreateTaskRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**act_id** | **String** |  | 
**name** | Option<**String**> |  | [optional]
**options** | Option<[**models::TaskOptions**](TaskOptions.md)> |  | [optional]
**input** | Option<**std::collections::HashMap<String, serde_json::Value>**> | The input configuration for the Actor task. This is a user-defined JSON object that will be passed to the Actor when the task is run.  | [optional]
**title** | Option<**String**> |  | [optional]
**actor_standby** | Option<[**models::ActorStandby**](ActorStandby.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


