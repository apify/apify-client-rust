# Webhook

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**modified_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**user_id** | **String** |  | 
**is_ad_hoc** | Option<**bool**> |  | [optional]
**should_interpolate_strings** | Option<**bool**> |  | [optional]
**event_types** | [**Vec<models::WebhookEventType>**](WebhookEventType.md) |  | 
**condition** | [**models::WebhookCondition**](WebhookCondition.md) |  | 
**ignore_ssl_errors** | **bool** |  | 
**do_not_retry** | Option<**bool**> |  | [optional]
**request_url** | **String** |  | 
**payload_template** | Option<**String**> |  | [optional]
**headers_template** | Option<**String**> |  | [optional]
**description** | Option<**String**> |  | [optional]
**last_dispatch** | Option<[**models::ExampleWebhookDispatch**](ExampleWebhookDispatch.md)> |  | [optional]
**stats** | Option<[**models::WebhookStats**](WebhookStats.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


