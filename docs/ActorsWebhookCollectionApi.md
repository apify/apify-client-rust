# \ActorsWebhookCollectionApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_webhooks_get**](ActorsWebhookCollectionApi.md#act_webhooks_get) | **GET** /v2/actors/{actorId}/webhooks | Get list of webhooks



## act_webhooks_get

> models::ListOfWebhooksResponse act_webhooks_get(actor_id, offset, limit, desc)
Get list of webhooks

Gets the list of webhooks of a specific Actor. The response is a JSON with the list of objects, where each object contains basic information about a single webhook.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 records.  By default, the records are sorted by the `createdAt` field in ascending order, to sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |

### Return type

[**models::ListOfWebhooksResponse**](ListOfWebhooksResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

