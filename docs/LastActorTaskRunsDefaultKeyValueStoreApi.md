# \LastActorTaskRunsDefaultKeyValueStoreApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actor_task_runs_last_key_value_store_delete**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store | Delete last task run's default store
[**actor_task_runs_last_key_value_store_get**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store | Get last task run's default store
[**actor_task_runs_last_key_value_store_keys_get**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_keys_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/keys | Get last task run's default store's list of keys
[**actor_task_runs_last_key_value_store_put**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store | Update last task run's default store
[**actor_task_runs_last_key_value_store_record_delete**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_record_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records/{recordKey} | Delete last task run's default store's record
[**actor_task_runs_last_key_value_store_record_get**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_record_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records/{recordKey} | Get last task run's default store's record
[**actor_task_runs_last_key_value_store_record_post**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_record_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records/{recordKey} | Store record in last task run's default store (POST)
[**actor_task_runs_last_key_value_store_record_put**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_record_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records/{recordKey} | Store record in last task run's default store
[**actor_task_runs_last_key_value_store_records_get**](LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_records_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records | Download last task run's default store's records



## actor_task_runs_last_key_value_store_delete

> actor_task_runs_last_key_value_store_delete(actor_task_id, status)
Delete last task run's default store

Deletes the default key-value store of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Delete store](/api/v2/key-value-store-delete) endpoint. 

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


## actor_task_runs_last_key_value_store_get

> models::KeyValueStoreResponse actor_task_runs_last_key_value_store_get(actor_task_id, status)
Get last task run's default store

Gets an object that contains all the details about the default key-value store of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Get store](/api/v2/key-value-store-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::KeyValueStoreResponse**](KeyValueStoreResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_key_value_store_keys_get

> models::ListOfKeysResponse actor_task_runs_last_key_value_store_keys_get(actor_task_id, status, exclusive_start_key, limit, collection, prefix, signature)
Get last task run's default store's list of keys

Returns a list of keys for the default key-value store of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Get list of keys](/api/v2/key-value-store-keys-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**exclusive_start_key** | Option<**String**> | All keys up to this one (including) are skipped from the result. |  |
**limit** | Option<**f64**> | Number of keys to be returned. |  |[default to 1000]
**collection** | Option<**String**> | Limit the results to keys that belong to a specific collection from the key-value store schema. The key-value store need to have a schema defined for this parameter to work. |  |
**prefix** | Option<**String**> | Limit the results to keys that start with a specific prefix. |  |
**signature** | Option<**String**> | Signature used for the access. |  |

### Return type

[**models::ListOfKeysResponse**](ListOfKeysResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_key_value_store_put

> models::KeyValueStoreResponse actor_task_runs_last_key_value_store_put(actor_task_id, update_store_request, status)
Update last task run's default store

Updates the default key-value store associated with the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Update store](/api/v2/key-value-store-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**update_store_request** | [**UpdateStoreRequest**](UpdateStoreRequest.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::KeyValueStoreResponse**](KeyValueStoreResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_key_value_store_record_delete

> actor_task_runs_last_key_value_store_record_delete(actor_task_id, record_key, status)
Delete last task run's default store's record

Removes a record specified by a key from the default key-value store of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Delete record](/api/v2/key-value-store-record-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**record_key** | **String** | Key of the record. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_key_value_store_record_get

> std::collections::HashMap<String, serde_json::Value> actor_task_runs_last_key_value_store_record_get(actor_task_id, record_key, status, signature, attachment)
Get last task run's default store's record

Gets a value stored under a specific key in the default key-value store of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Get record](/api/v2/key-value-store-record-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**record_key** | **String** | Key of the record. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**signature** | Option<**String**> | Signature used for the access. |  |
**attachment** | Option<**bool**> | If `true` or `1`, the response will be served with `Content-Disposition: attachment` header, causing web browsers to offer downloading HTML records instead of displaying them.  |  |

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, */*

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_key_value_store_record_post

> serde_json::Value actor_task_runs_last_key_value_store_record_post(actor_task_id, record_key, request_body, status, content_encoding)
Store record in last task run's default store (POST)

Stores a value under a specific key in the default key-value store of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Store record](/api/v2/key-value-store-record-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**record_key** | **String** | Key of the record. | [required] |
**request_body** | [**std::collections::HashMap<String, serde_json::Value>**](SerdeJson__Value.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**content_encoding** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_key_value_store_record_put

> serde_json::Value actor_task_runs_last_key_value_store_record_put(actor_task_id, record_key, request_body, status, content_encoding)
Store record in last task run's default store

Stores a value under a specific key in the default key-value store of the last Actor task run.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Store record](/api/v2/key-value-store-record-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**record_key** | **String** | Key of the record. | [required] |
**request_body** | [**std::collections::HashMap<String, serde_json::Value>**](SerdeJson__Value.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**content_encoding** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_key_value_store_records_get

> std::path::PathBuf actor_task_runs_last_key_value_store_records_get(actor_task_id, status, collection, prefix, signature)
Download last task run's default store's records

Downloads all records from the default key-value store of the last Actor task run as a ZIP archive.  This endpoint is a shortcut for getting the last task run's `defaultKeyValueStoreId` and then using the [Download records](/api/v2/key-value-store-records-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**collection** | Option<**String**> | If specified, only records belonging to a specific collection from the key-value store schema. The key-value store need to have a schema defined for this parameter to work.  |  |
**prefix** | Option<**String**> | If specified, only records whose key starts with the given prefix are included in the archive.  |  |
**signature** | Option<**String**> | Signature used for the access. |  |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/zip, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

