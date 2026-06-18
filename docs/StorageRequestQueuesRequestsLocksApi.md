# \StorageRequestQueuesRequestsLocksApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**request_queue_head_get**](StorageRequestQueuesRequestsLocksApi.md#request_queue_head_get) | **GET** /v2/request-queues/{queueId}/head | Get head
[**request_queue_head_lock_post**](StorageRequestQueuesRequestsLocksApi.md#request_queue_head_lock_post) | **POST** /v2/request-queues/{queueId}/head/lock | Get head and lock
[**request_queue_request_lock_delete**](StorageRequestQueuesRequestsLocksApi.md#request_queue_request_lock_delete) | **DELETE** /v2/request-queues/{queueId}/requests/{requestId}/lock | Delete request lock
[**request_queue_request_lock_put**](StorageRequestQueuesRequestsLocksApi.md#request_queue_request_lock_put) | **PUT** /v2/request-queues/{queueId}/requests/{requestId}/lock | Prolong request lock
[**request_queue_requests_unlock_post**](StorageRequestQueuesRequestsLocksApi.md#request_queue_requests_unlock_post) | **POST** /v2/request-queues/{queueId}/requests/unlock | Unlock requests



## request_queue_head_get

> models::HeadResponse request_queue_head_get(queue_id, limit, client_key)
Get head

Returns given number of first requests from the queue.  The response contains the `hadMultipleClients` boolean field which indicates that the queue was accessed by more than one client (with unique or empty `clientKey`). This field is used by [Apify SDK](https://sdk.apify.com) to determine whether the local cache is consistent with the request queue, and thus optimize performance of certain operations. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**limit** | Option<**f64**> | How many items from queue should be returned. |  |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

[**models::HeadResponse**](HeadResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_head_lock_post

> models::HeadAndLockResponse request_queue_head_lock_post(queue_id, lock_secs, limit, client_key)
Get head and lock

Returns the given number of first requests from the queue and locks them for the given time.  If this endpoint locks the request, no other client or run will be able to get and lock these requests.  The response contains the `hadMultipleClients` boolean field which indicates that the queue was accessed by more than one client (with unique or empty `clientKey`). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**lock_secs** | **f64** | How long the requests will be locked for (in seconds). | [required] |
**limit** | Option<**f64**> | How many items from the queue should be returned. |  |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

[**models::HeadAndLockResponse**](HeadAndLockResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_request_lock_delete

> request_queue_request_lock_delete(queue_id, request_id, client_key, forefront)
Delete request lock

Deletes a request lock. The request lock can be deleted only by the client that has locked it using [Get and lock head operation](#/request-queue-head-lock-post).  The clientKey identifier is used for locking and unlocking requests. You can delete or prolong the lock only for requests that were locked by the same client key or from the same Actor run. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**request_id** | **String** | Request ID. | [required] |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |
**forefront** | Option<**String**> | Determines if request should be added to the head of the queue or to the end after lock was removed.  |  |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_request_lock_put

> models::ProlongRequestLockResponse request_queue_request_lock_put(queue_id, request_id, lock_secs, client_key, forefront)
Prolong request lock

Prolongs request lock. The request lock can be prolonged only by the client that has locked it using [Get and lock head operation](#/request-queue-head-lock-post).  The clientKey identifier is used for locking and unlocking requests. You can delete or prolong the lock only for requests that were locked by the same client key or from the same Actor run. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**request_id** | **String** | Request ID. | [required] |
**lock_secs** | **f64** | How long the requests will be locked for (in seconds). | [required] |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |
**forefront** | Option<**String**> | Determines if request should be added to the head of the queue or to the end after lock expires.  |  |

### Return type

[**models::ProlongRequestLockResponse**](ProlongRequestLockResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_requests_unlock_post

> models::UnlockRequestsResponse request_queue_requests_unlock_post(queue_id, client_key)
Unlock requests

Unlocks requests in the queue that are currently locked by the client.  * If the client is within an Actor run, it unlocks all requests locked by that specific run plus all requests locked by the same clientKey. * If the client is outside of an Actor run, it unlocks all requests locked using the same clientKey. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

[**models::UnlockRequestsResponse**](UnlockRequestsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

