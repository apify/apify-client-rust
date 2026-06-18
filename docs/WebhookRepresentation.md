# WebhookRepresentation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_types** | [**Vec<models::WebhookEventType>**](WebhookEventType.md) |  | 
**request_url** | **String** | The URL to which the webhook sends its payload. | 
**payload_template** | Option<**String**> | Optional template for the JSON payload sent by the webhook. | [optional]
**headers_template** | Option<**String**> | Optional template for the HTTP headers sent by the webhook. | [optional]
**should_interpolate_strings** | Option<**bool**> | Flag to also interpolate `{{...}}` variables inside string values of the payload and headers templates. | [optional]
**idempotency_key** | Option<**String**> | Key that prevents creating duplicate webhooks, e.g. when the run-starting request is retried. | [optional]
**ignore_ssl_errors** | Option<**bool**> | Flag to ignore SSL errors when the webhook sends the request. | [optional]
**do_not_retry** | Option<**bool**> | Flag to skip retrying the webhook request on failure. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


