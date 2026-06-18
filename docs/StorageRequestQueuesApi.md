# \StorageRequestQueuesApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**request_queue_delete**](StorageRequestQueuesApi.md#request_queue_delete) | **DELETE** /v2/request-queues/{queueId} | Delete request queue
[**request_queue_get**](StorageRequestQueuesApi.md#request_queue_get) | **GET** /v2/request-queues/{queueId} | Get request queue
[**request_queue_put**](StorageRequestQueuesApi.md#request_queue_put) | **PUT** /v2/request-queues/{queueId} | Update request queue
[**request_queue_requests_batch_delete**](StorageRequestQueuesApi.md#request_queue_requests_batch_delete) | **DELETE** /v2/request-queues/{queueId}/requests/batch | Delete requests
[**request_queue_requests_batch_post**](StorageRequestQueuesApi.md#request_queue_requests_batch_post) | **POST** /v2/request-queues/{queueId}/requests/batch | Add requests
[**request_queues_get**](StorageRequestQueuesApi.md#request_queues_get) | **GET** /v2/request-queues | Get list of request queues
[**request_queues_post**](StorageRequestQueuesApi.md#request_queues_post) | **POST** /v2/request-queues | Create request queue



## request_queue_delete

> request_queue_delete(queue_id)
Delete request queue

Deletes given queue.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_get

> models::RequestQueueResponse request_queue_get(queue_id)
Get request queue

Returns queue object for given queue ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |

### Return type

[**models::RequestQueueResponse**](RequestQueueResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_put

> models::RequestQueueResponse request_queue_put(queue_id, update_request_queue_request)
Update request queue

Updates a request queue's name and general resource access level using a value specified by a JSON object passed in the PUT payload.  The response is the updated request queue object, as returned by the [Get request queue](#/reference/request-queues/queue-collection/get-request-queue) API endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**update_request_queue_request** | [**UpdateRequestQueueRequest**](UpdateRequestQueueRequest.md) |  | [required] |

### Return type

[**models::RequestQueueResponse**](RequestQueueResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_requests_batch_delete

> models::BatchDeleteResponse request_queue_requests_batch_delete(queue_id, content_type, request_draft_delete, client_key)
Delete requests

Batch-deletes given requests from the queue. The number of requests in a batch is limited to 25. The response contains an array of unprocessed and processed requests. If any delete operation fails because the request queue rate limit is exceeded or an internal failure occurs, the failed request is returned in the `unprocessedRequests` response parameter. You can re-send these delete requests. It is recommended to use an exponential backoff algorithm for these retries. Each request is identified by its ID or uniqueKey parameter. You can use either of them to identify the request. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**content_type** | **String** |  | [required] |
**request_draft_delete** | [**Vec<models::RequestDraftDelete>**](RequestDraftDelete.md) |  | [required] |
**client_key** | Option<**String**> | A unique identifier of the client accessing the request queue. It must be a string between 1 and 32 characters long. This identifier is used to determine whether the queue was accessed by multiple clients. If `clientKey` is not provided, the system considers this API call to come from a new client. For details, see the `hadMultipleClients` field returned by the [Get head](#/reference/request-queues/queue-head) operation.  |  |

### Return type

[**models::BatchDeleteResponse**](BatchDeleteResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queue_requests_batch_post

> models::BatchAddResponse request_queue_requests_batch_post(queue_id, request_base, client_key, forefront)
Add requests

Adds requests to the queue in batch. The maximum requests in batch is limited to 25. The response contains an array of unprocessed and processed requests. If any add operation fails because the request queue rate limit is exceeded or an internal failure occurs, the failed request is returned in the unprocessedRequests response parameter. You can resend these requests to add. It is recommended to use an exponential backoff algorithm for these retries. If a request with the same `uniqueKey` was already present in the queue, then it returns an ID of the existing request. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue_id** | **String** | Queue ID or `username~queue-name`. | [required] |
**request_base** | [**Vec<models::RequestBase>**](RequestBase.md) |  | [required] |
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


## request_queues_get

> models::ListOfRequestQueuesResponse request_queues_get(offset, limit, desc, unnamed, ownership)
Get list of request queues

Lists all of a user's request queues. The response is a JSON array of objects, where each object contains basic information about one queue.  By default, the objects are sorted by the `createdAt` field in ascending order, therefore you can use pagination to incrementally fetch all queues while new ones are still being created. To sort them in descending order, use `desc=1` parameter. The endpoint supports pagination using `limit` and `offset` parameters and it will not return more than 1000 array elements. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |
**unnamed** | Option<**bool**> | If `true` or `1` then all the storages are returned. By default, only named storages are returned.  |  |
**ownership** | Option<[**StorageOwnership**](StorageOwnership.md)> | Filter by ownership. If this parameter is omitted, all accessible request queues are returned.  - `ownedByMe`: Return only request queues owned by the user. - `sharedWithMe`: Return only request queues shared with the user by other users.  |  |

### Return type

[**models::ListOfRequestQueuesResponse**](ListOfRequestQueuesResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_queues_post

> models::RequestQueueResponse request_queues_post(name)
Create request queue

Creates a request queue and returns its object. Keep in mind that requests stored under unnamed queue follows [data retention period](https://docs.apify.com/platform/storage#data-retention).  It creates a queue of given name if the parameter name is used. If a queue with the given name already exists then the endpoint returns its object. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | Option<**String**> | Custom unique name to easily identify the queue in the future. |  |

### Return type

[**models::RequestQueueResponse**](RequestQueueResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

