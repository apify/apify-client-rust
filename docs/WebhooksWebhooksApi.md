# \WebhooksWebhooksApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**webhook_delete**](WebhooksWebhooksApi.md#webhook_delete) | **DELETE** /v2/webhooks/{webhookId} | Delete webhook
[**webhook_get**](WebhooksWebhooksApi.md#webhook_get) | **GET** /v2/webhooks/{webhookId} | Get webhook
[**webhook_put**](WebhooksWebhooksApi.md#webhook_put) | **PUT** /v2/webhooks/{webhookId} | Update webhook
[**webhook_test_post**](WebhooksWebhooksApi.md#webhook_test_post) | **POST** /v2/webhooks/{webhookId}/test | Test webhook
[**webhook_webhook_dispatches_get**](WebhooksWebhooksApi.md#webhook_webhook_dispatches_get) | **GET** /v2/webhooks/{webhookId}/dispatches | Get collection
[**webhooks_get**](WebhooksWebhooksApi.md#webhooks_get) | **GET** /v2/webhooks | Get list of webhooks
[**webhooks_post**](WebhooksWebhooksApi.md#webhooks_post) | **POST** /v2/webhooks | Create webhook



## webhook_delete

> serde_json::Value webhook_delete(webhook_id)
Delete webhook

Deletes a webhook.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**webhook_id** | **String** | Webhook ID. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhook_get

> models::WebhookResponse webhook_get(webhook_id)
Get webhook

Gets webhook object with all details.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**webhook_id** | **String** | Webhook ID. | [required] |

### Return type

[**models::WebhookResponse**](WebhookResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhook_put

> models::WebhookResponse webhook_put(webhook_id, webhook_update)
Update webhook

Updates a webhook using values specified by a webhook object passed as JSON in the POST payload. If the object does not define a specific property, its value will not be updated.  The response is the full webhook object as returned by the [Get webhook](#/reference/webhooks/webhook-object/get-webhook) endpoint.  The request needs to specify the `Content-Type: application/json` HTTP header!  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. ([More info](#/introduction/authentication)). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**webhook_id** | **String** | Webhook ID. | [required] |
**webhook_update** | [**WebhookUpdate**](WebhookUpdate.md) |  | [required] |

### Return type

[**models::WebhookResponse**](WebhookResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhook_test_post

> models::TestWebhookResponse webhook_test_post(webhook_id)
Test webhook

Tests a webhook. Creates a webhook dispatch with a dummy payload.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**webhook_id** | **String** | Webhook ID. | [required] |

### Return type

[**models::TestWebhookResponse**](TestWebhookResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhook_webhook_dispatches_get

> models::ListOfWebhookDispatchesResponse webhook_webhook_dispatches_get(webhook_id)
Get collection

Gets a given webhook's list of dispatches.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**webhook_id** | **String** | Webhook ID. | [required] |

### Return type

[**models::ListOfWebhookDispatchesResponse**](ListOfWebhookDispatchesResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webhooks_get

> models::ListOfWebhooksResponse webhooks_get(offset, limit, desc)
Get list of webhooks

Gets the list of webhooks that the user created.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 records. By default, the records are sorted by the `createdAt` field in ascending order. To sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
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


## webhooks_post

> models::WebhookResponse webhooks_post(webhook_create)
Create webhook

Creates a new webhook with settings provided by the webhook object passed as JSON in the payload. The response is the created webhook object.  To avoid duplicating a webhook, use the `idempotencyKey` parameter in the request body. Multiple calls to create a webhook with the same `idempotencyKey` will only create the webhook with the first call and return the existing webhook on subsequent calls. Idempotency keys must be unique, so use a UUID or another random string with enough entropy.  To assign the new webhook to an Actor or task, the request body must contain `requestUrl`, `eventTypes`, and `condition` properties.  * `requestUrl` is the webhook's target URL, to which data is sent as a POST request with a JSON payload. * `eventTypes` is a list of events that will trigger the webhook, e.g. when the Actor run succeeds. * `condition` should be an object containing the ID of the Actor or task to which the webhook will be assigned. * `payloadTemplate` is a JSON-like string, whose syntax is extended with the use of variables. * `headersTemplate` is a JSON-like string, whose syntax is extended with the use of variables. Following values will be re-written to defaults: \"host\", \"Content-Type\", \"X-Apify-Webhook\", \"X-Apify-Webhook-Dispatch-Id\", \"X-Apify-Request-Origin\" * `description` is an optional string. * `shouldInterpolateStrings` is a boolean indicating whether to interpolate variables contained inside strings in the `payloadTemplate`  ```     \"isAdHoc\" : false,     \"requestUrl\" : \"https://example.com\",     \"eventTypes\" : [         \"ACTOR.RUN.SUCCEEDED\",         \"ACTOR.RUN.ABORTED\"     ],     \"condition\" : {         \"actorId\": \"5sTMwDQywwsLzKRRh\",         \"actorTaskId\" : \"W9bs9JE9v7wprjAnJ\"     },     \"payloadTemplate\": \"\",     \"headersTemplate\": \"\",     \"description\": \"my awesome webhook\",     \"shouldInterpolateStrings\": false, ```  **Important**: The request must specify the `Content-Type: application/json` HTTP header. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**webhook_create** | [**WebhookCreate**](WebhookCreate.md) |  | [required] |

### Return type

[**models::WebhookResponse**](WebhookResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

