# \StorageRequestQueuesRequestsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**request_queue_request_delete**](StorageRequestQueuesRequestsApi.md#request_queue_request_delete) | **DELETE** /v2/request-queues/{queueId}/requests/{requestId} | Delete request
[**request_queue_request_get**](StorageRequestQueuesRequestsApi.md#request_queue_request_get) | **GET** /v2/request-queues/{queueId}/requests/{requestId} | Get request
[**request_queue_request_put**](StorageRequestQueuesRequestsApi.md#request_queue_request_put) | **PUT** /v2/request-queues/{queueId}/requests/{requestId} | Update request
[**request_queue_requests_get**](StorageRequestQueuesRequestsApi.md#request_queue_requests_get) | **GET** /v2/request-queues/{queueId}/requests | List requests
[**request_queue_requests_post**](StorageRequestQueuesRequestsApi.md#request_queue_requests_post) | **POST** /v2/request-queues/{queueId}/requests | Add request



## request_queue_request_delete

> request_queue_request_delete(queue_id, request_id, client_key)
Delete request

Deletes given request from queue.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**request_id** | **String** | Request ID. | [required] |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_request_get

> models::RequestResponse request_queue_request_get(queue_id, request_id)
Get request

Returns request from queue.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**request_id** | **String** | Request ID. | [required] |

### Return type

[**models::RequestResponse**](RequestResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_request_put

> models::UpdateRequestResponse request_queue_request_put(queue_id, request_id, request, forefront, client_key)
Update request

Updates a request in a queue. Mark request as handled by setting `request.handledAt = new Date()`. If `handledAt` is set, the request will be removed from head of the queue (and unlocked, if applicable). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**request_id** | **String** | Request ID. | [required] |
**request** | [**Request**](Request.md) |  | [required] |
**forefront** | Option<**String**> | Determines if request should be added to the head of the queue or to the end. Default value is `false` (end of queue).  |  |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

[**models::UpdateRequestResponse**](UpdateRequestResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_requests_get

> models::ListOfRequestsResponse request_queue_requests_get(queue_id, client_key, exclusive_start_id, limit, cursor, filter)
List requests

Returns a list of requests. This endpoint is paginated using cursor (pagination by `exclusiveStartId` is deprecated) and limit parameters. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |
**exclusive_start_id** | Option<**String**> | All requests up to this one (including) are skipped from the result. (Deprecated, use `cursor` instead.) |  |
**limit** | Option<**f64**> | Number of keys to be returned. Maximum value is `10000`. |  |
**cursor** | Option<**String**> | A cursor string for pagination, returned in the previous response as `nextCursor`. Use this to retrieve the next page of requests. |  |
**filter** | Option<[**Vec<String>**](String.md)> | Filter requests by their state. Possible values are `locked` and `pending`. You can combine multiple values separated by commas, which will mean the union of these filters – requests matching any of the specified states will be returned. (Not compatible with deprecated `exclusiveStartId` parameter.) |  |

### Return type

[**models::ListOfRequestsResponse**](ListOfRequestsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_requests_post

> models::AddRequestResponse request_queue_requests_post(queue_id, request_base, client_key, forefront)
Add request

Adds request to the queue. Response contains ID of the request and info if request was already present in the queue or handled.  If request with same `uniqueKey` was already present in the queue then returns an ID of existing request. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**request_base** | [**RequestBase**](RequestBase.md) |  | [required] |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |
**forefront** | Option<**String**> | Determines if request should be added to the head of the queue or to the end. Default value is `false` (end of queue).  |  |

### Return type

[**models::AddRequestResponse**](AddRequestResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

