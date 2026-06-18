# \LastActorRunsDefaultRequestQueueApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_runs_last_request_queue_delete**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_delete) | **DELETE** /v2/actors/{actorId}/runs/last/request-queue | Delete last run's default request queue
[**act_runs_last_request_queue_get**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_get) | **GET** /v2/actors/{actorId}/runs/last/request-queue | Get last run's default request queue
[**act_runs_last_request_queue_head_get**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_head_get) | **GET** /v2/actors/{actorId}/runs/last/request-queue/head | Get last run's default request queue head
[**act_runs_last_request_queue_head_lock_post**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_head_lock_post) | **POST** /v2/actors/{actorId}/runs/last/request-queue/head/lock | Get and lock last run's default request queue head
[**act_runs_last_request_queue_put**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_put) | **PUT** /v2/actors/{actorId}/runs/last/request-queue | Update last run's default request queue
[**act_runs_last_request_queue_request_delete**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_delete) | **DELETE** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId} | Delete request from last run's default request queue
[**act_runs_last_request_queue_request_get**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_get) | **GET** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId} | Get request from last run's default request queue
[**act_runs_last_request_queue_request_lock_delete**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_lock_delete) | **DELETE** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId}/lock | Delete lock on request in last run's default request queue
[**act_runs_last_request_queue_request_lock_put**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_lock_put) | **PUT** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId}/lock | Prolong lock on request in last run's default request queue
[**act_runs_last_request_queue_request_put**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_put) | **PUT** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId} | Update request in last run's default request queue
[**act_runs_last_request_queue_requests_batch_delete**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_batch_delete) | **DELETE** /v2/actors/{actorId}/runs/last/request-queue/requests/batch | Batch delete requests from last run's default request queue
[**act_runs_last_request_queue_requests_batch_post**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_batch_post) | **POST** /v2/actors/{actorId}/runs/last/request-queue/requests/batch | Batch add requests to last run's default request queue
[**act_runs_last_request_queue_requests_get**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_get) | **GET** /v2/actors/{actorId}/runs/last/request-queue/requests | List last run's default request queue's requests
[**act_runs_last_request_queue_requests_post**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_post) | **POST** /v2/actors/{actorId}/runs/last/request-queue/requests | Add request to last run's default request queue
[**act_runs_last_request_queue_requests_unlock_post**](LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_unlock_post) | **POST** /v2/actors/{actorId}/runs/last/request-queue/requests/unlock | Unlock requests in last run's default request queue



## act_runs_last_request_queue_delete

> act_runs_last_request_queue_delete(actor_id, status)
Delete last run's default request queue

Deletes the default request queue associated with the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Delete request queue](/api/v2/request-queue-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_request_queue_get

> models::RequestQueueResponse act_runs_last_request_queue_get(actor_id, status)
Get last run's default request queue

Returns the default request queue associated with the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Get request queue](/api/v2/request-queue-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::RequestQueueResponse**](RequestQueueResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_request_queue_head_get

> models::HeadResponse act_runs_last_request_queue_head_get(actor_id, status, limit, client_key)
Get last run's default request queue head

Returns the given number of first requests from the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Get head](/api/v2/request-queue-head-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
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


## act_runs_last_request_queue_head_lock_post

> models::HeadAndLockResponse act_runs_last_request_queue_head_lock_post(actor_id, lock_secs, status, limit, client_key)
Get and lock last run's default request queue head

Returns the given number of first requests from the default request queue of the last Actor run and locks them for the given time.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Get head and lock](/api/v2/request-queue-head-lock-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**lock_secs** | **f64** | How long the requests will be locked for (in seconds). | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
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


## act_runs_last_request_queue_put

> models::RequestQueueResponse act_runs_last_request_queue_put(actor_id, update_request_queue_request, status)
Update last run's default request queue

Updates the default request queue associated with the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Update request queue](/api/v2/request-queue-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**update_request_queue_request** | [**UpdateRequestQueueRequest**](UpdateRequestQueueRequest.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::RequestQueueResponse**](RequestQueueResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_request_queue_request_delete

> act_runs_last_request_queue_request_delete(actor_id, request_id, status, client_key)
Delete request from last run's default request queue

Deletes a request from the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Delete request](/api/v2/request-queue-request-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**request_id** | **String** | Request ID. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_request_queue_request_get

> models::RequestResponse act_runs_last_request_queue_request_get(actor_id, request_id, status)
Get request from last run's default request queue

Returns a request from the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Get request](/api/v2/request-queue-request-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**request_id** | **String** | Request ID. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::RequestResponse**](RequestResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_request_queue_request_lock_delete

> act_runs_last_request_queue_request_lock_delete(actor_id, request_id, status, client_key, forefront)
Delete lock on request in last run's default request queue

Deletes a request lock in the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Delete request lock](/api/v2/request-queue-request-lock-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**request_id** | **String** | Request ID. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
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


## act_runs_last_request_queue_request_lock_put

> models::ProlongRequestLockResponse act_runs_last_request_queue_request_lock_put(actor_id, request_id, lock_secs, status, client_key, forefront)
Prolong lock on request in last run's default request queue

Prolongs a request lock in the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Prolong request lock](/api/v2/request-queue-request-lock-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**request_id** | **String** | Request ID. | [required] |
**lock_secs** | **f64** | How long the requests will be locked for (in seconds). | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
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


## act_runs_last_request_queue_request_put

> models::UpdateRequestResponse act_runs_last_request_queue_request_put(actor_id, request_id, request, status, forefront, client_key)
Update request in last run's default request queue

Updates a request in the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Update request](/api/v2/request-queue-request-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**request_id** | **String** | Request ID. | [required] |
**request** | [**Request**](Request.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
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


## act_runs_last_request_queue_requests_batch_delete

> models::BatchDeleteResponse act_runs_last_request_queue_requests_batch_delete(actor_id, content_type, request_draft_delete, status, client_key)
Batch delete requests from last run's default request queue

Batch-deletes requests from the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Delete requests](/api/v2/request-queue-requests-batch-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**content_type** | **String** |  | [required] |
**request_draft_delete** | [**Vec<models::RequestDraftDelete>**](RequestDraftDelete.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

[**models::BatchDeleteResponse**](BatchDeleteResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_request_queue_requests_batch_post

> models::BatchAddResponse act_runs_last_request_queue_requests_batch_post(actor_id, request_base, status, client_key, forefront)
Batch add requests to last run's default request queue

Adds requests to the default request queue of the last Actor run in batch.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Add requests](/api/v2/request-queue-requests-batch-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**request_base** | [**Vec<models::RequestBase>**](RequestBase.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |
**forefront** | Option<**String**> | Determines if request should be added to the head of the queue or to the end. Default value is `false` (end of queue).  |  |

### Return type

[**models::BatchAddResponse**](BatchAddResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_request_queue_requests_get

> models::ListOfRequestsResponse act_runs_last_request_queue_requests_get(actor_id, status, client_key, exclusive_start_id, limit, cursor, filter)
List last run's default request queue's requests

Returns a list of requests from the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [List requests](/api/v2/request-queue-requests-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
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


## act_runs_last_request_queue_requests_post

> models::AddRequestResponse act_runs_last_request_queue_requests_post(actor_id, request_base, status, client_key, forefront)
Add request to last run's default request queue

Adds a request to the default request queue of the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Add request](/api/v2/request-queue-requests-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**request_base** | [**RequestBase**](RequestBase.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
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


## act_runs_last_request_queue_requests_unlock_post

> models::UnlockRequestsResponse act_runs_last_request_queue_requests_unlock_post(actor_id, status, client_key)
Unlock requests in last run's default request queue

Unlocks requests in the default request queue of the last Actor run that are currently locked by the client.  This endpoint is a shortcut for getting the last run's `defaultRequestQueueId` and then using the [Unlock requests](/api/v2/request-queue-requests-unlock-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

[**models::UnlockRequestsResponse**](UnlockRequestsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

