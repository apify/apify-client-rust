# PricePerDatasetItemActorPricingInfo

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
**pricing_model** | **PricingModel** |  (enum: PRICE_PER_DATASET_ITEM) | 
**unit_name** | **String** | Name of the unit that is being charged | 
**price_per_unit_usd** | Option<**f64**> | Price per unit in USD. Mutually exclusive with `tieredPricing` - exactly one of the two is present on a pricing record.  | [optional]
**tiered_pricing** | Option<[**std::collections::HashMap<String, models::TieredPricingPerDatasetItemEntry>**](TieredPricingPerDatasetItemEntry.md)> | Tiered price-per-dataset-item pricing, keyed by subscription tier (e.g. `FREE`, `BRONZE`, `SILVER`, `GOLD`, `PLATINUM`, `DIAMOND`). The actual price applied to a run is resolved from the user's tier.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


