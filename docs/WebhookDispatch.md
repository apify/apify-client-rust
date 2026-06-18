# WebhookDispatch

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**user_id** | **String** |  | 
**webhook_id** | **String** |  | 
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**status** | [**models::WebhookDispatchStatus**](WebhookDispatchStatus.md) |  | 
**event_type** | [**models::WebhookEventType**](WebhookEventType.md) |  | 
**event_data** | Option<[**models::EventData**](EventData.md)> |  | [optional]
**webhook** | Option<[**models::WebhookDispatchWebhookSummary**](WebhookDispatchWebhookSummary.md)> |  | [optional]
**calls** | Option<[**Vec<models::CallsInner>**](CallsInner.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


