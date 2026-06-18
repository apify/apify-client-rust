# MonthlyUsage

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**usage_cycle** | [**models::UsageCycle**](UsageCycle.md) |  | 
**monthly_service_usage** | [**std::collections::HashMap<String, models::UsageItem>**](UsageItem.md) | A map of usage item names (e.g., ACTOR_COMPUTE_UNITS) to their usage details. | 
**daily_service_usages** | [**Vec<models::DailyServiceUsages>**](DailyServiceUsages.md) |  | 
**total_usage_credits_usd_before_volume_discount** | **f64** |  | 
**total_usage_credits_usd_after_volume_discount** | **f64** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


