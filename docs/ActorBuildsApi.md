# \ActorBuildsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actor_build_abort_post**](ActorBuildsApi.md#actor_build_abort_post) | **POST** /v2/actor-builds/{buildId}/abort | Abort build
[**actor_build_delete**](ActorBuildsApi.md#actor_build_delete) | **DELETE** /v2/actor-builds/{buildId} | Delete build
[**actor_build_get**](ActorBuildsApi.md#actor_build_get) | **GET** /v2/actor-builds/{buildId} | Get build
[**actor_build_log_get**](ActorBuildsApi.md#actor_build_log_get) | **GET** /v2/actor-builds/{buildId}/log | Get build's Log
[**actor_build_openapi_json_get**](ActorBuildsApi.md#actor_build_openapi_json_get) | **GET** /v2/actor-builds/{buildId}/openapi.json | Get OpenAPI definition
[**actor_builds_get**](ActorBuildsApi.md#actor_builds_get) | **GET** /v2/actor-builds | Get user builds list



## actor_build_abort_post

> models::BuildResponse actor_build_abort_post(build_id)
Abort build

Aborts an Actor build and returns an object that contains all the details about the build.  Only builds that are starting or running are aborted. For builds with status `FINISHED`, `FAILED`, `ABORTING` and `TIMED-OUT` this call does nothing. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**build_id** | **String** | ID of the build, found in the build's Info tab. | [required] |

### Return type

[**models::BuildResponse**](BuildResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_build_delete

> actor_build_delete(build_id)
Delete build

Delete the build. The build that is the current default build for the Actor cannot be deleted.  Only users with build permissions for the Actor can delete builds. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**build_id** | **String** | ID of the build, found in the build's Info tab. | [required] |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_build_get

> models::BuildResponse actor_build_get(build_id, wait_for_finish)
Get build

Gets an object that contains all the details about a specific build of an Actor.  By passing the optional `waitForFinish` parameter the API endpoint will synchronously wait for the build to finish. This is useful to avoid periodic polling when waiting for an Actor build to finish.  This endpoint does not require the authentication token. Instead, calls are authenticated using a hard-to-guess ID of the build. However, if you access the endpoint without the token, certain attributes, such as `usageUsd` and `usageTotalUsd`, will be hidden. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**build_id** | **String** | ID of the build, found in the build's Info tab. | [required] |
**wait_for_finish** | Option<**f64**> | The maximum number of seconds the server waits for the build to finish. By default it is `0`, the maximum value is `60`. <!-- MAX_ACTOR_JOB_ASYNC_WAIT_SECS --> If the build finishes in time then the returned build object will have a terminal status (e.g. `SUCCEEDED`), otherwise it will have a transitional status (e.g. `RUNNING`).  |  |

### Return type

[**models::BuildResponse**](BuildResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_build_log_get

> String actor_build_log_get(build_id, stream, download)
Get build's Log

Retrieves Actor build's logs.  This endpoint is a shortcut for getting the build's log. Same as [Get log](/api/v2/log-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**build_id** | **String** | ID of the build, found in the build's Info tab. | [required] |
**stream** | Option<**bool**> | If `true` or `1` then the logs will be streamed as long as the run or build is running.  |  |
**download** | Option<**bool**> | If `true` or `1` then the web browser will download the log file rather than open it in a tab.  |  |

### Return type

**String**

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/plain, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_build_openapi_json_get

> serde_json::Value actor_build_openapi_json_get(build_id)
Get OpenAPI definition

Get the OpenAPI definition for Actor builds. Two similar endpoints are available:  - [First endpoint](/api/v2/act-openapi-json-get): Requires both `actorId` and `buildId`. Use `default` as the `buildId` to get the OpenAPI schema for the default Actor build. - [Second endpoint](/api/v2/actor-build-openapi-json-get): Requires only `buildId`.  Get the OpenAPI definition for a specific Actor build. Authentication is based on the build's unique ID. No authentication token is required.  :::note  You can also use the [`/api/v2/act-openapi-json-get`](/api/v2/act-openapi-json-get) endpoint to get the OpenAPI definition for a build.  ::: 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**build_id** | **String** | ID of the build, found in the build's Info tab. Use the special value `default` to get the OpenAPI schema for the Actor's default build.  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_builds_get

> models::ListOfBuildsResponse actor_builds_get(offset, limit, desc)
Get user builds list

Gets a list of all builds for a user. The response is a JSON array of objects, where each object contains basic information about a single build.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 records.  By default, the records are sorted by the `startedAt` field in ascending order. Therefore, you can use pagination to incrementally fetch all builds while new ones are still being started. To sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `startedAt` field in descending order. By default, they are sorted in ascending order.  |  |

### Return type

[**models::ListOfBuildsResponse**](ListOfBuildsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

