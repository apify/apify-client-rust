# \StorageKeyValueStoresApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**key_value_store_delete**](StorageKeyValueStoresApi.md#key_value_store_delete) | **DELETE** /v2/key-value-stores/{storeId} | Delete store
[**key_value_store_get**](StorageKeyValueStoresApi.md#key_value_store_get) | **GET** /v2/key-value-stores/{storeId} | Get store
[**key_value_store_keys_get**](StorageKeyValueStoresApi.md#key_value_store_keys_get) | **GET** /v2/key-value-stores/{storeId}/keys | Get list of keys
[**key_value_store_put**](StorageKeyValueStoresApi.md#key_value_store_put) | **PUT** /v2/key-value-stores/{storeId} | Update store
[**key_value_store_record_delete**](StorageKeyValueStoresApi.md#key_value_store_record_delete) | **DELETE** /v2/key-value-stores/{storeId}/records/{recordKey} | Delete record
[**key_value_store_record_get**](StorageKeyValueStoresApi.md#key_value_store_record_get) | **GET** /v2/key-value-stores/{storeId}/records/{recordKey} | Get record
[**key_value_store_record_head**](StorageKeyValueStoresApi.md#key_value_store_record_head) | **HEAD** /v2/key-value-stores/{storeId}/records/{recordKey} | Check if a record exists
[**key_value_store_record_post**](StorageKeyValueStoresApi.md#key_value_store_record_post) | **POST** /v2/key-value-stores/{storeId}/records/{recordKey} | Store record (POST)
[**key_value_store_record_put**](StorageKeyValueStoresApi.md#key_value_store_record_put) | **PUT** /v2/key-value-stores/{storeId}/records/{recordKey} | Store record
[**key_value_store_records_get**](StorageKeyValueStoresApi.md#key_value_store_records_get) | **GET** /v2/key-value-stores/{storeId}/records | Download records
[**key_value_stores_get**](StorageKeyValueStoresApi.md#key_value_stores_get) | **GET** /v2/key-value-stores | Get list of key-value stores
[**key_value_stores_post**](StorageKeyValueStoresApi.md#key_value_stores_post) | **POST** /v2/key-value-stores | Create key-value store



## key_value_store_delete

> key_value_store_delete(store_id)
Delete store

Deletes a key-value store.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_store_get

> models::KeyValueStoreResponse key_value_store_get(store_id)
Get store

Gets an object that contains all the details about a specific key-value store. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |

### Return type

[**models::KeyValueStoreResponse**](KeyValueStoreResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_store_keys_get

> models::ListOfKeysResponse key_value_store_keys_get(store_id, exclusive_start_key, limit, collection, prefix, signature)
Get list of keys

Returns a list of objects describing keys of a given key-value store, as well as some information about the values (e.g. size).  This endpoint is paginated using `exclusiveStartKey` and `limit` parameters - see [Pagination](/api/v2#using-key) for more details. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |
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


## key_value_store_put

> models::KeyValueStoreResponse key_value_store_put(store_id, update_store_request)
Update store

Updates a key-value store's name and general resource access level using a value specified by a JSON object passed in the PUT payload.  The response is the updated key-value store object, as returned by the [Get store](#/reference/key-value-stores/store-object/get-store) API endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |
**update_store_request** | [**UpdateStoreRequest**](UpdateStoreRequest.md) |  | [required] |

### Return type

[**models::KeyValueStoreResponse**](KeyValueStoreResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_store_record_delete

> key_value_store_record_delete(store_id, record_key)
Delete record

Removes a record specified by a key from the key-value store.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |
**record_key** | **String** | Key of the record. | [required] |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_store_record_get

> std::collections::HashMap<String, serde_json::Value> key_value_store_record_get(store_id, record_key, attachment, signature)
Get record

Gets a value stored in the key-value store under a specific key.  The response body has the same `Content-Encoding` header as it was set in [Put record](#tag/Key-value-storesRecord/operation/keyValueStore_record_put).  If the request does not define the `Accept-Encoding` HTTP header with the right encoding, the record will be decompressed.  Most HTTP clients support decompression by default. After using the HTTP client with decompression support, the `Accept-Encoding` header is set by the client and body is decompressed automatically.  Please note that for security reasons, Apify API can perform small modifications to HTML documents before they are served via this endpoint. To fetch the raw HTML content without any modifications, use the `attachment` query parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |
**record_key** | **String** | Key of the record. | [required] |
**attachment** | Option<**bool**> | If `true` or `1`, the response will be served with `Content-Disposition: attachment` header, causing web browsers to offer downloading HTML records instead of displaying them.  |  |
**signature** | Option<**String**> | Signature used for the access. |  |

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, */*

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_store_record_head

> key_value_store_record_head(store_id, record_key)
Check if a record exists

Check if a value is stored in the key-value store under a specific key. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |
**record_key** | **String** | Key of the record. | [required] |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_store_record_post

> serde_json::Value key_value_store_record_post(store_id, record_key, request_body, content_encoding)
Store record (POST)

Stores a value under a specific key to the key-value store.  This endpoint is an alias for the [`PUT` record](#tag/Key-value-storesRecord/operation/keyValueStore_record_put) method and behaves identically. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |
**record_key** | **String** | Key of the record. | [required] |
**request_body** | [**std::collections::HashMap<String, serde_json::Value>**](SerdeJson__Value.md) |  | [required] |
**content_encoding** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_store_record_put

> serde_json::Value key_value_store_record_put(store_id, record_key, request_body, content_encoding)
Store record

Stores a value under a specific key to the key-value store.  The value is passed as the PUT payload and it is stored with a MIME content type defined by the `Content-Type` header and with encoding defined by the `Content-Encoding` header.  To save bandwidth, storage, and speed up your upload, send the request payload compressed with Gzip compression and add the `Content-Encoding: gzip` header. It is possible to set up another compression type with `Content-Encoding` request header.  Below is a list of supported `Content-Encoding` types.  * Gzip compression: `Content-Encoding: gzip` * Deflate compression: `Content-Encoding: deflate` * Brotli compression: `Content-Encoding: br` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |
**record_key** | **String** | Key of the record. | [required] |
**request_body** | [**std::collections::HashMap<String, serde_json::Value>**](SerdeJson__Value.md) |  | [required] |
**content_encoding** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_store_records_get

> std::path::PathBuf key_value_store_records_get(store_id, collection, prefix, signature)
Download records

Downloads all records from the key-value store as a ZIP archive. Each record is stored as a separate file in the archive, with the filename equal to the record key.  You can optionally filter the records by `collection` or `prefix` to download only a subset of the store. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**store_id** | **String** | Key-value store ID or `username~store-name`. | [required] |
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


## key_value_stores_get

> models::ListOfKeyValueStoresResponse key_value_stores_get(offset, limit, desc, unnamed, ownership)
Get list of key-value stores

Gets the list of key-value stores owned by the user.  The response is a list of objects, where each objects contains a basic information about a single key-value store.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 array elements.  By default, the records are sorted by the `createdAt` field in ascending order, therefore you can use pagination to incrementally fetch all key-value stores while new ones are still being created. To sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |
**unnamed** | Option<**bool**> | If `true` or `1` then all the storages are returned. By default, only named storages are returned.  |  |
**ownership** | Option<[**StorageOwnership**](StorageOwnership.md)> | Filter by ownership. If this parameter is omitted, all accessible key-value stores are returned.  - `ownedByMe`: Return only key-value stores owned by the user. - `sharedWithMe`: Return only key-value stores shared with the user by other users.  |  |

### Return type

[**models::ListOfKeyValueStoresResponse**](ListOfKeyValueStoresResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## key_value_stores_post

> models::KeyValueStoreResponse key_value_stores_post(name)
Create key-value store

Creates a key-value store and returns its object. The response is the same object as returned by the [Get store](#/reference/key-value-stores/store-object/get-store) endpoint.  Keep in mind that data stored under unnamed store follows [data retention period](https://docs.apify.com/platform/storage#data-retention).  It creates a store with the given name if the parameter name is used. If there is another store with the same name, the endpoint does not create a new one and returns the existing object instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | Option<**String**> | Custom unique name to easily identify the store in the future. |  |

### Return type

[**models::KeyValueStoreResponse**](KeyValueStoreResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

