# ActorChargeEvent

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_title** | **String** | Human-readable title shown to users in the billing UI. | 
**event_description** | **String** | Human-readable description of what triggers this event. | 
**event_price_usd** | Option<**f64**> | Flat price per event in USD. Present only for non-tiered events. Mutually exclusive with `eventTieredPricingUsd`.  | [optional]
**event_tiered_pricing_usd** | Option<[**std::collections::HashMap<String, models::TieredPricingPerEventEntry>**](TieredPricingPerEventEntry.md)> | Tiered price-per-event pricing for a single charge event, keyed by subscription tier (e.g. `FREE`, `BRONZE`, `SILVER`, `GOLD`, `PLATINUM`, `DIAMOND`). The actual price applied is resolved from the user's tier.  | [optional]
**is_primary_event** | Option<**bool**> | Whether this event is the Actor's primary chargeable event. | [optional]
**is_one_time_event** | Option<**bool**> | Whether this event can only be charged once per Actor run. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


