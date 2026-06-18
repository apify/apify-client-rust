# \ActorsActorVersionsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_version_delete**](ActorsActorVersionsApi.md#act_version_delete) | **DELETE** /v2/actors/{actorId}/versions/{versionNumber} | Delete version
[**act_version_env_var_delete**](ActorsActorVersionsApi.md#act_version_env_var_delete) | **DELETE** /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName} | Delete environment variable
[**act_version_env_var_get**](ActorsActorVersionsApi.md#act_version_env_var_get) | **GET** /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName} | Get environment variable
[**act_version_env_var_post**](ActorsActorVersionsApi.md#act_version_env_var_post) | **POST** /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName} | Update environment variable (POST)
[**act_version_env_var_put**](ActorsActorVersionsApi.md#act_version_env_var_put) | **PUT** /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName} | Update environment variable
[**act_version_env_vars_get**](ActorsActorVersionsApi.md#act_version_env_vars_get) | **GET** /v2/actors/{actorId}/versions/{versionNumber}/env-vars | Get list of environment variables
[**act_version_env_vars_post**](ActorsActorVersionsApi.md#act_version_env_vars_post) | **POST** /v2/actors/{actorId}/versions/{versionNumber}/env-vars | Create environment variable
[**act_version_get**](ActorsActorVersionsApi.md#act_version_get) | **GET** /v2/actors/{actorId}/versions/{versionNumber} | Get version
[**act_version_post**](ActorsActorVersionsApi.md#act_version_post) | **POST** /v2/actors/{actorId}/versions/{versionNumber} | Update version (POST)
[**act_version_put**](ActorsActorVersionsApi.md#act_version_put) | **PUT** /v2/actors/{actorId}/versions/{versionNumber} | Update version
[**act_versions_get**](ActorsActorVersionsApi.md#act_versions_get) | **GET** /v2/actors/{actorId}/versions | Get list of versions
[**act_versions_post**](ActorsActorVersionsApi.md#act_versions_post) | **POST** /v2/actors/{actorId}/versions | Create version



## act_version_delete

> serde_json::Value act_version_delete(actor_id, version_number)
Delete version

Deletes a specific version of Actor's source code. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_env_var_delete

> serde_json::Value act_version_env_var_delete(actor_id, version_number, env_var_name)
Delete environment variable

Deletes a specific environment variable.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |
**env_var_name** | **String** | The name of the environment variable | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_env_var_get

> models::EnvVarResponse act_version_env_var_get(actor_id, version_number, env_var_name)
Get environment variable

Gets a [EnvVar object](#/reference/actors/environment-variable-object) that contains all the details about a specific environment variable of an Actor.  If `isSecret` is set to `true`, then `value` will never be returned. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |
**env_var_name** | **String** | The name of the environment variable | [required] |

### Return type

[**models::EnvVarResponse**](EnvVarResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_env_var_post

> models::EnvVarResponse act_version_env_var_post(actor_id, version_number, env_var_name, env_var_request)
Update environment variable (POST)

Updates Actor environment variable using values specified by a [EnvVar object](#/reference/actors/environment-variable-object) passed as JSON in the POST payload. This endpoint is an alias for the [`PUT` update environment variable](#tag/ActorsEnvironment-variable-object/operation/act_version_envVar_put) method and behaves identically. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |
**env_var_name** | **String** | The name of the environment variable | [required] |
**env_var_request** | [**EnvVarRequest**](EnvVarRequest.md) |  | [required] |

### Return type

[**models::EnvVarResponse**](EnvVarResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_env_var_put

> models::EnvVarResponse act_version_env_var_put(actor_id, version_number, env_var_name, env_var_request)
Update environment variable

Updates Actor environment variable using values specified by a [EnvVar object](#/reference/actors/environment-variable-object) passed as JSON in the POST payload. If the object does not define a specific property, its value will not be updated.  The request needs to specify the `Content-Type: application/json` HTTP header!  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. ([More info](#/introduction/authentication)).  The response is the [EnvVar object](#/reference/actors/environment-variable-object) as returned by the [Get environment variable](#/reference/actors/environment-variable-object/get-environment-variable) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |
**env_var_name** | **String** | The name of the environment variable | [required] |
**env_var_request** | [**EnvVarRequest**](EnvVarRequest.md) |  | [required] |

### Return type

[**models::EnvVarResponse**](EnvVarResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_env_vars_get

> models::ListOfEnvVarsResponse act_version_env_vars_get(actor_id, version_number)
Get list of environment variables

Gets the list of environment variables for a specific version of an Actor. The response is a JSON object with the list of [EnvVar objects](#/reference/actors/environment-variable-object), where each contains basic information about a single environment variable. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |

### Return type

[**models::ListOfEnvVarsResponse**](ListOfEnvVarsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_env_vars_post

> models::EnvVarResponse act_version_env_vars_post(actor_id, version_number, env_var_request)
Create environment variable

Creates an environment variable of an Actor using values specified in a [EnvVar object](#/reference/actors/environment-variable-object) passed as JSON in the POST payload.  The request must specify `name` and `value` parameters (as strings) in the JSON payload and a `Content-Type: application/json` HTTP header.  ``` {     \"name\": \"ENV_VAR_NAME\",     \"value\": \"my-env-var\" } ```  The response is the [EnvVar object](#/reference/actors/environment-variable-object) as returned by the [Get environment variable](#/reference/actors/environment-variable-object/get-environment-variable) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |
**env_var_request** | [**EnvVarRequest**](EnvVarRequest.md) |  | [required] |

### Return type

[**models::EnvVarResponse**](EnvVarResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_get

> models::VersionResponse act_version_get(actor_id, version_number)
Get version

Gets a [Version object](#/reference/actors/version-object) that contains all the details about a specific version of an Actor. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |

### Return type

[**models::VersionResponse**](VersionResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_post

> models::VersionResponse act_version_post(actor_id, version_number, create_or_update_version_request)
Update version (POST)

Updates Actor version using values specified by a [Version object](#/reference/actors/version-object) passed as JSON in the POST payload. This endpoint is an alias for the [`PUT` update version](#tag/ActorsVersion-object/operation/act_version_put) method and behaves identically. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |
**create_or_update_version_request** | [**CreateOrUpdateVersionRequest**](CreateOrUpdateVersionRequest.md) |  | [required] |

### Return type

[**models::VersionResponse**](VersionResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_version_put

> models::VersionResponse act_version_put(actor_id, version_number, create_or_update_version_request)
Update version

Updates Actor version using values specified by a [Version object](#/reference/actors/version-object) passed as JSON in the POST payload.  If the object does not define a specific property, its value will not be updated.  The request needs to specify the `Content-Type: application/json` HTTP header!  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. ([More info](#/introduction/authentication)).  The response is the [Version object](#/reference/actors/version-object) as returned by the [Get version](#/reference/actors/version-object/get-version) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version_number** | **String** | Actor version. | [required] |
**create_or_update_version_request** | [**CreateOrUpdateVersionRequest**](CreateOrUpdateVersionRequest.md) |  | [required] |

### Return type

[**models::VersionResponse**](VersionResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_versions_get

> models::ListOfVersionsResponse act_versions_get(actor_id)
Get list of versions

Gets the list of versions of a specific Actor. The response is a JSON object with the list of [Version objects](#/reference/actors/version-object), where each contains basic information about a single version. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |

### Return type

[**models::ListOfVersionsResponse**](ListOfVersionsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_versions_post

> models::VersionResponse act_versions_post(actor_id, create_or_update_version_request)
Create version

Creates a version of an Actor using values specified in a [Version object](#/reference/actors/version-object) passed as JSON in the POST payload.  The request must specify `versionNumber` and `sourceType` parameters (as strings) in the JSON payload and a `Content-Type: application/json` HTTP header.  Each `sourceType` requires its own additional properties to be passed to the JSON payload object. These are outlined in the [Version object](#/reference/actors/version-object) table below and in more detail in the [Apify documentation](https://docs.apify.com/platform/actors/development/deployment/source-types).  For example, if an Actor's source code is stored in a [GitHub repository](https://docs.apify.com/platform/actors/development/deployment/source-types#git-repository), you will set the `sourceType` to `GIT_REPO` and pass the repository's URL in the `gitRepoUrl` property.  ``` {     \"versionNumber\": \"0.1\",     \"sourceType\": \"GIT_REPO\",     \"gitRepoUrl\": \"https://github.com/my-github-account/actor-repo\" } ```  The response is the [Version object](#/reference/actors/version-object) as returned by the [Get version](#/reference/actors/version-object/get-version) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**create_or_update_version_request** | [**CreateOrUpdateVersionRequest**](CreateOrUpdateVersionRequest.md) |  | [required] |

### Return type

[**models::VersionResponse**](VersionResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

