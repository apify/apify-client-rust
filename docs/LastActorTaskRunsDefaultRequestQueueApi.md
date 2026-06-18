# \LastActorTaskRunsDefaultRequestQueueApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actor_task_runs_last_request_queue_delete**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue | Delete last task run's default request queue
[**actor_task_runs_last_request_queue_get**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue | Get last task run's default request queue
[**actor_task_runs_last_request_queue_head_get**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_head_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/head | Get last task run's default request queue head
[**actor_task_runs_last_request_queue_head_lock_post**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_head_lock_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/head/lock | Get and lock last task run's default request queue head
[**actor_task_runs_last_request_queue_put**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue | Update last task run's default request queue
[**actor_task_runs_last_request_queue_request_delete**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId} | Delete request from last task run's default request queue
[**actor_task_runs_last_request_queue_request_get**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId} | Get request from last task run's default request queue
[**actor_task_runs_last_request_queue_request_lock_delete**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_lock_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId}/lock | Delete lock on request in last task run's default request queue
[**actor_task_runs_last_request_queue_request_lock_put**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_lock_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId}/lock | Prolong lock on request in last task run's default request queue
[**actor_task_runs_last_request_queue_request_put**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId} | Update request in last task run's default request queue
[**actor_task_runs_last_request_queue_requests_batch_delete**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_batch_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/batch | Batch delete requests from last task run's default request queue
[**actor_task_runs_last_request_queue_requests_batch_post**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_batch_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/batch | Batch add requests to last task run's default request queue
[**actor_task_runs_last_request_queue_requests_get**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests | List last task run's default request queue's requests
[**actor_task_runs_last_request_queue_requests_post**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests | Add request to last task run's default request queue
[**actor_task_runs_last_request_queue_requests_unlock_post**](LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_unlock_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/unlock | Unlock requests in last task run's default request queue



## actor_task_runs_last_request_queue_delete

> actor_task_runs_last_request_queue_delete(actor_task_id, status)
Delete last task run's default request queue

Deletes the default request queue associated with the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Delete request queue](/api/v2/request-queue-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_request_queue_get

> models::RequestQueueResponse actor_task_runs_last_request_queue_get(actor_task_id, status)
Get last task run's default request queue

Returns the default request queue associated with the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Get request queue](/api/v2/request-queue-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::RequestQueueResponse**](RequestQueueResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_request_queue_head_get

> models::HeadResponse actor_task_runs_last_request_queue_head_get(actor_task_id, status, limit, client_key)
Get last task run's default request queue head

Returns the given number of first requests from the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Get head](/api/v2/request-queue-head-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_head_lock_post

> models::HeadAndLockResponse actor_task_runs_last_request_queue_head_lock_post(actor_task_id, lock_secs, status, limit, client_key)
Get and lock last task run's default request queue head

Returns the given number of first requests from the default request queue of the last Actor task run and locks them for the given time.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Get head and lock](/api/v2/request-queue-head-lock-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_put

> models::RequestQueueResponse actor_task_runs_last_request_queue_put(actor_task_id, update_request_queue_request, status)
Update last task run's default request queue

Updates the default request queue associated with the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Update request queue](/api/v2/request-queue-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_request_delete

> actor_task_runs_last_request_queue_request_delete(actor_task_id, request_id, status, client_key)
Delete request from last task run's default request queue

Deletes a request from the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Delete request](/api/v2/request-queue-request-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_request_get

> models::RequestResponse actor_task_runs_last_request_queue_request_get(actor_task_id, request_id, status)
Get request from last task run's default request queue

Returns a request from the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Get request](/api/v2/request-queue-request-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_request_lock_delete

> actor_task_runs_last_request_queue_request_lock_delete(actor_task_id, request_id, status, client_key, forefront)
Delete lock on request in last task run's default request queue

Deletes a request lock in the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Delete request lock](/api/v2/request-queue-request-lock-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_request_lock_put

> models::ProlongRequestLockResponse actor_task_runs_last_request_queue_request_lock_put(actor_task_id, request_id, lock_secs, status, client_key, forefront)
Prolong lock on request in last task run's default request queue

Prolongs a request lock in the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Prolong request lock](/api/v2/request-queue-request-lock-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_request_put

> models::UpdateRequestResponse actor_task_runs_last_request_queue_request_put(actor_task_id, request_id, request, status, forefront, client_key)
Update request in last task run's default request queue

Updates a request in the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Update request](/api/v2/request-queue-request-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_requests_batch_delete

> models::BatchDeleteResponse actor_task_runs_last_request_queue_requests_batch_delete(actor_task_id, content_type, request_draft_delete, status, client_key)
Batch delete requests from last task run's default request queue

Batch-deletes requests from the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Delete requests](/api/v2/request-queue-requests-batch-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_requests_batch_post

> models::BatchAddResponse actor_task_runs_last_request_queue_requests_batch_post(actor_task_id, request_base, status, client_key, forefront)
Batch add requests to last task run's default request queue

Adds requests to the default request queue of the last Actor task run in batch.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Add requests](/api/v2/request-queue-requests-batch-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_requests_get

> models::ListOfRequestsResponse actor_task_runs_last_request_queue_requests_get(actor_task_id, status, client_key, exclusive_start_id, limit, cursor, filter)
List last task run's default request queue's requests

Returns a list of requests from the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [List requests](/api/v2/request-queue-requests-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_requests_post

> models::AddRequestResponse actor_task_runs_last_request_queue_requests_post(actor_task_id, request_base, status, client_key, forefront)
Add request to last task run's default request queue

Adds a request to the default request queue of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Add request](/api/v2/request-queue-requests-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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


## actor_task_runs_last_request_queue_requests_unlock_post

> models::UnlockRequestsResponse actor_task_runs_last_request_queue_requests_unlock_post(actor_task_id, status, client_key)
Unlock requests in last task run's default request queue

Unlocks requests in the default request queue of the last Actor task run that are currently locked by the client.  This endpoint is a shortcut for getting the last task run's `defaultRequestQueueId` and then using the [Unlock requests](/api/v2/request-queue-requests-unlock-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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

