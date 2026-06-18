# \ActorsActorBuildsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_build_abort_post**](ActorsActorBuildsApi.md#act_build_abort_post) | **POST** /v2/actors/{actorId}/builds/{buildId}/abort | Abort build
[**act_build_default_get**](ActorsActorBuildsApi.md#act_build_default_get) | **GET** /v2/actors/{actorId}/builds/default | Get default build
[**act_build_get**](ActorsActorBuildsApi.md#act_build_get) | **GET** /v2/actors/{actorId}/builds/{buildId} | Get build
[**act_builds_get**](ActorsActorBuildsApi.md#act_builds_get) | **GET** /v2/actors/{actorId}/builds | Get list of builds
[**act_builds_post**](ActorsActorBuildsApi.md#act_builds_post) | **POST** /v2/actors/{actorId}/builds | Build Actor
[**act_openapi_json_get**](ActorsActorBuildsApi.md#act_openapi_json_get) | **GET** /v2/actors/{actorId}/builds/{buildId}/openapi.json | Get OpenAPI definition



## act_build_abort_post

> models::BuildResponse act_build_abort_post(actor_id, build_id)
Abort build

**[DEPRECATED]** API endpoints related to build of the Actor were moved under new namespace [`actor-builds`](#/reference/actor-builds). Aborts an Actor build and returns an object that contains all the details about the build.  Only builds that are starting or running are aborted. For builds with status `FINISHED`, `FAILED`, `ABORTING` and `TIMED-OUT` this call does nothing. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**build_id** | **String** | ID of the build, found in the build's Info tab. | [required] |

### Return type

[**models::BuildResponse**](BuildResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_build_default_get

> models::BuildResponse act_build_default_get(actor_id, wait_for_finish)
Get default build

Get the default build for an Actor.  Use the optional `waitForFinish` parameter to synchronously wait for the build to finish. This avoids the need for periodic polling when waiting for the build to complete.  This endpoint does not require an authentication token. Instead, calls are authenticated using the Actor's unique ID. However, if you access the endpoint without a token, certain attributes (e.g., `usageUsd` and `usageTotalUsd`) will be hidden. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**wait_for_finish** | Option<**f64**> | The maximum number of seconds the server waits for the build to finish. By default it is `0`, the maximum value is `60`. <!-- MAX_ACTOR_JOB_ASYNC_WAIT_SECS --> If the build finishes in time then the returned build object will have a terminal status (e.g. `SUCCEEDED`), otherwise it will have a transitional status (e.g. `RUNNING`).  |  |

### Return type

[**models::BuildResponse**](BuildResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_build_get

> models::BuildResponse act_build_get(actor_id, build_id, wait_for_finish)
Get build

By passing the optional `waitForFinish` parameter the API endpoint will synchronously wait for the build to finish. This is useful to avoid periodic polling when waiting for an Actor build to finish.  This endpoint does not require the authentication token. Instead, calls are authenticated using a hard-to-guess ID of the build. However, if you access the endpoint without the token, certain attributes, such as `usageUsd` and `usageTotalUsd`, will be hidden. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
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


## act_builds_get

> models::ListOfBuildsResponse act_builds_get(actor_id, offset, limit, desc)
Get list of builds

Gets the list of builds of a specific Actor. The response is a JSON with the list of objects, where each object contains basic information about a single build.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 records.  By default, the records are sorted by the `startedAt` field in ascending order, therefore you can use pagination to incrementally fetch all builds while new ones are still being started. To sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
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


## act_builds_post

> models::BuildResponse act_builds_post(actor_id, version, use_cache, beta_packages, tag, wait_for_finish)
Build Actor

Builds an Actor. The response is the build object as returned by the [Get build](#/reference/actors/build-object/get-build) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**version** | **String** | Actor version number to be built. | [required] |
**use_cache** | Option<**bool**> | If `true` or `1`, the system will use a cache to speed up the build process. By default, cache is not used.  |  |
**beta_packages** | Option<**bool**> | If `true` or `1` then the Actor is built with beta versions of Apify NPM packages. By default, the build uses `latest` packages.  |  |
**tag** | Option<**String**> | Tag to be applied to the build on success. By default, the tag is taken from Actor version's `buildTag` property.  |  |
**wait_for_finish** | Option<**f64**> | The maximum number of seconds the server waits for the build to finish. By default it is `0`, the maximum value is `60`. <!-- MAX_ACTOR_JOB_ASYNC_WAIT_SECS --> If the build finishes in time then the returned build object will have a terminal status (e.g. `SUCCEEDED`), otherwise it will have a transitional status (e.g. `RUNNING`).  |  |

### Return type

[**models::BuildResponse**](BuildResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_openapi_json_get

> serde_json::Value act_openapi_json_get(actor_id, build_id)
Get OpenAPI definition

Get the OpenAPI definition for Actor builds. Two similar endpoints are available:  - [First endpoint](/api/v2/act-openapi-json-get): Requires both `actorId` and `buildId`. Use `default` as the `buildId` to get the OpenAPI schema for the default Actor build.  - [Second endpoint](/api/v2/actor-build-openapi-json-get): Requires only `buildId`.  Get the OpenAPI definition for a specific Actor build.  To fetch the default Actor build, simply pass `default` as the `buildId`. Authentication is based on the build's unique ID. No authentication token is required.  :::note  You can also use the [`/api/v2/actor-build-openapi-json-get`](/api/v2/actor-build-openapi-json-get) endpoint to get the OpenAPI definition for a build.  ::: 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**build_id** | **String** | ID of the build, found in the build's Info tab. Use the special value `default` to get the OpenAPI schema for the Actor's default build.  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

