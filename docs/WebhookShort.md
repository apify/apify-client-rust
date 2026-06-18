# WebhookShort

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**modified_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**user_id** | **String** |  | 
**is_ad_hoc** | Option<**bool**> |  | [optional]
**is_apify_integration** | Option<**bool**> |  | [optional]
**is_enabled** | Option<**bool**> |  | [optional]
**action_type** | Option<**String**> |  | [optional]
**should_interpolate_strings** | Option<**bool**> |  | [optional]
**event_types** | [**Vec<models::WebhookEventType>**](WebhookEventType.md) |  | 
**condition** | [**models::WebhookCondition**](WebhookCondition.md) |  | 
**ignore_ssl_errors** | **bool** |  | 
**do_not_retry** | **bool** |  | 
**request_url** | **String** |  | 
**last_dispatch** | Option<[**models::ExampleWebhookDispatch**](ExampleWebhookDispatch.md)> |  | [optional]
**stats** | Option<[**models::WebhookStats**](WebhookStats.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


