# CurrentPricingInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**pricing_model** | **String** |  | 
**apify_margin_percentage** | Option<**f64**> |  | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**started_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**notified_about_change_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**notified_about_future_change_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**is_price_change_notification_suppressed** | Option<**bool**> |  | [optional]
**force_contains_significant_price_change** | Option<**bool**> |  | [optional]
**is_ppe_platform_usage_paid_by_user** | Option<**bool**> |  | [optional]
**reason_for_change** | Option<**String**> |  | [optional]
**trial_minutes** | Option<**i32**> |  | [optional]
**unit_name** | Option<**String**> |  | [optional]
**price_per_unit_usd** | Option<**f64**> |  | [optional]
**minimal_max_total_charge_usd** | Option<**f64**> |  | [optional]
**pricing_per_event** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Per-event pricing configuration for pay-per-event Actors. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


