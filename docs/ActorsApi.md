# \ActorsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_delete**](ActorsApi.md#act_delete) | **DELETE** /v2/actors/{actorId} | Delete Actor
[**act_get**](ActorsApi.md#act_get) | **GET** /v2/actors/{actorId} | Get Actor
[**act_put**](ActorsApi.md#act_put) | **PUT** /v2/actors/{actorId} | Update Actor
[**act_validate_input_post**](ActorsApi.md#act_validate_input_post) | **POST** /v2/actors/{actorId}/validate-input | Validate Actor input
[**acts_get**](ActorsApi.md#acts_get) | **GET** /v2/actors | Get list of Actors
[**acts_post**](ActorsApi.md#acts_post) | **POST** /v2/actors | Create Actor



## act_delete

> serde_json::Value act_delete(actor_id)
Delete Actor

Deletes an Actor.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_get

> models::ActorResponse act_get(actor_id)
Get Actor

Gets an object that contains all the details about a specific Actor.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |

### Return type

[**models::ActorResponse**](ActorResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_put

> models::ActorResponse act_put(actor_id, update_actor_request)
Update Actor

Updates settings of an Actor using values specified by an Actor object passed as JSON in the POST payload. If the object does not define a specific property, its value will not be updated.  The response is the full Actor object as returned by the [Get Actor](#/reference/actors/actor-object/get-actor) endpoint.  The request needs to specify the `Content-Type: application/json` HTTP header!  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. ([More info](#/introduction/authentication)).  If you want to make your Actor [public](https://docs.apify.com/platform/actors/publishing) using `isPublic: true`, you will need to provide the Actor's `title` and the `categories` under which that Actor will be classified in Apify Store. For this, it's best to use the [constants from our `apify-shared-js` package](https://github.com/apify/apify-shared-js/blob/2d43ebc41ece9ad31cd6525bd523fb86939bf860/packages/consts/src/consts.ts#L452-L471). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**update_actor_request** | [**UpdateActorRequest**](UpdateActorRequest.md) |  | [required] |

### Return type

[**models::ActorResponse**](ActorResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_validate_input_post

> models::ActValidateInputPost200Response act_validate_input_post(actor_id, body, build)
Validate Actor input

Validates the provided input against the Actor's input schema for the specified build.  The endpoint checks whether the JSON payload conforms to the input schema defined in the Actor's build. If no `build` query parameter is provided, the `latest` build tag is used by default. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**body** | **serde_json::Value** | JSON input to validate against the Actor's input schema. | [required] |
**build** | Option<**String**> | Optional tag or number of the Actor build to use for input schema validation. By default, the `latest` build tag is used.  |  |

### Return type

[**models::ActValidateInputPost200Response**](act_validateInput_post_200_response.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## acts_get

> models::ListOfActorsResponse acts_get(my, offset, limit, desc, sort_by)
Get list of Actors

Gets the list of all Actors that the user created or used. The response is a list of objects, where each object contains a basic information about a single Actor.  To only get Actors created by the user, add the `my=1` query parameter.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 records.  By default, the records are sorted by the `createdAt` field in ascending order, therefore you can use pagination to incrementally fetch all Actors while new ones are still being created. To sort the records in descending order, use the `desc=1` parameter.  You can also sort by your last run by using the `sortBy=stats.lastRunStartedAt` query parameter. In this case, descending order means the most recently run Actor appears first. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**my** | Option<**bool**> | If `true` or `1` then the returned list only contains Actors owned by the user. The default value is `false`.  |  |
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |
**sort_by** | Option<**String**> | Field to sort the records by. The default is `createdAt`. You can also use `stats.lastRunStartedAt` to sort by the most recently ran Actors.  |  |

### Return type

[**models::ListOfActorsResponse**](ListOfActorsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## acts_post

> models::ActorResponse acts_post(create_actor_request)
Create Actor

Creates a new Actor with settings specified in an Actor object passed as JSON in the POST payload. The response is the full Actor object as returned by the [Get Actor](#/reference/actors/actor-object/get-actor) endpoint.  The HTTP request must have the `Content-Type: application/json` HTTP header!  The Actor needs to define at least one version of the source code. For more information, see [Version object](#/reference/actors/version-object).  If you want to make your Actor [public](https://docs.apify.com/platform/actors/publishing) using `isPublic: true`, you will need to provide the Actor's `title` and the `categories` under which that Actor will be classified in Apify Store. For this, it's best to use the [constants from our `apify-shared-js` package](https://github.com/apify/apify-shared-js/blob/2d43ebc41ece9ad31cd6525bd523fb86939bf860/packages/consts/src/consts.ts#L452-L471). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_actor_request** | [**CreateActorRequest**](CreateActorRequest.md) |  | [required] |

### Return type

[**models::ActorResponse**](ActorResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

