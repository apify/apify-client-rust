# ActorDefinition

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**actor_specification** | Option<**ActorSpecification**> | The Actor specification version that this Actor follows. This property must be set to 1. (enum: 1) | [optional]
**name** | Option<**String**> | The name of the Actor. | [optional]
**version** | Option<**String**> | The version of the Actor, typically a dot-separated sequence of numbers (e.g., `0.1`, `1.0`, or `0.0.1`). | [optional]
**build_tag** | Option<**String**> | The tag name to be applied to a successful build of the Actor. Defaults to 'latest' if not specified. | [optional]
**environment_variables** | Option<**std::collections::HashMap<String, String>**> | A map of environment variables to be used during local development and deployment. | [optional]
**dockerfile** | Option<**String**> | The path to the Dockerfile used for building the Actor on the platform. | [optional]
**docker_context_dir** | Option<**String**> | The path to the directory used as the Docker context when building the Actor. | [optional]
**readme** | Option<**String**> | The path to the README file for the Actor. | [optional]
**input** | Option<**serde_json::Value**> | The input schema object, the full specification can be found in [Apify docs](https://docs.apify.com/platform/actors/development/actor-definition/input-schema) | [optional]
**changelog** | Option<**String**> | The path to the CHANGELOG file displayed in the Actor's information tab. | [optional]
**storages** | Option<[**models::ActorDefinitionStorages**](ActorDefinitionStorages.md)> |  | [optional]
**default_memory_mbytes** | Option<[**models::ActorDefinitionDefaultMemoryMbytes**](ActorDefinitionDefaultMemoryMbytes.md)> |  | [optional]
**min_memory_mbytes** | Option<**i32**> | Specifies the minimum amount of memory in megabytes required by the Actor. | [optional]
**max_memory_mbytes** | Option<**i32**> | Specifies the maximum amount of memory in megabytes required by the Actor. | [optional]
**uses_standby_mode** | Option<**bool**> | Specifies whether Standby mode is enabled for the Actor. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


