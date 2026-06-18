# WebhookCreate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**is_ad_hoc** | Option<**bool**> |  | [optional]
**event_types** | [**Vec<models::WebhookEventType>**](WebhookEventType.md) |  | 
**condition** | [**models::WebhookCondition**](WebhookCondition.md) |  | 
**idempotency_key** | Option<**String**> |  | [optional]
**ignore_ssl_errors** | Option<**bool**> |  | [optional]
**do_not_retry** | Option<**bool**> |  | [optional]
**request_url** | **String** |  | 
**payload_template** | Option<**String**> |  | [optional]
**headers_template** | Option<**String**> |  | [optional]
**description** | Option<**String**> |  | [optional]
**should_interpolate_strings** | Option<**bool**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


