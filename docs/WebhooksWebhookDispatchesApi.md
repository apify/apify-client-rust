# \WebhooksWebhookDispatchesApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**webhook_dispatch_get**](WebhooksWebhookDispatchesApi.md#webhook_dispatch_get) | **GET** /v2/webhook-dispatches/{dispatchId} | Get webhook dispatch
[**webhook_dispatches_get**](WebhooksWebhookDispatchesApi.md#webhook_dispatches_get) | **GET** /v2/webhook-dispatches | Get list of webhook dispatches



## webhook_dispatch_get

> models::WebhookDispatchResponse webhook_dispatch_get(dispatch_id)
Get webhook dispatch

Gets webhook dispatch object with all details.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dispatch_id** | **String** | Webhook dispatch ID. | [required] |

### Return type

[**models::WebhookDispatchResponse**](WebhookDispatchResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhook_dispatches_get

> models::ListOfWebhookDispatchesResponse webhook_dispatches_get(offset, limit, desc)
Get list of webhook dispatches

Gets the list of webhook dispatches that the user have.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 records. By default, the records are sorted by the `createdAt` field in ascending order. To sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |

### Return type

[**models::ListOfWebhookDispatchesResponse**](ListOfWebhookDispatchesResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

