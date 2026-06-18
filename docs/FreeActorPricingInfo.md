# FreeActorPricingInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**apify_margin_percentage** | **f64** | In [0, 1], fraction of pricePerUnitUsd that goes to Apify | 
**created_at** | **chrono::DateTime<chrono::FixedOffset>** | When this pricing info record has been created | 
**started_at** | **chrono::DateTime<chrono::FixedOffset>** | Since when is this pricing info record effective for a given Actor | 
**notified_about_future_change_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**notified_about_change_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> |  | [optional]
**reason_for_change** | Option<**String**> |  | [optional]
**is_price_change_notification_suppressed** | Option<**bool**> |  | [optional]
**force_contains_significant_price_change** | Option<**bool**> |  | [optional]
**pricing_model** | **PricingModel** |  (enum: FREE) | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


