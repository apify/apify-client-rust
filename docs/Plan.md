# Plan

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**description** | **String** |  | 
**is_enabled** | **bool** |  | 
**monthly_base_price_usd** | **f64** |  | 
**monthly_usage_credits_usd** | **f64** |  | 
**usage_discount_percent** | Option<**f64**> |  | [optional]
**enabled_platform_features** | **Vec<String>** |  | 
**max_monthly_usage_usd** | **f64** |  | 
**max_actor_memory_gbytes** | **f64** |  | 
**max_monthly_actor_compute_units** | **f64** |  | 
**max_monthly_residential_proxy_gbytes** | **f64** |  | 
**max_monthly_proxy_serps** | **i32** |  | 
**max_monthly_external_data_transfer_gbytes** | **f64** |  | 
**max_actor_count** | **i32** |  | 
**max_actor_task_count** | **i32** |  | 
**data_retention_days** | **i32** |  | 
**available_proxy_groups** | **std::collections::HashMap<String, i32>** | A dictionary mapping proxy group names to the number of available proxies in each group. The keys are proxy group names (e.g., \"RESIDENTIAL\", \"DATACENTER\") and values are the count of available proxies.  | 
**team_account_seat_count** | **i32** |  | 
**support_level** | **String** |  | 
**available_add_ons** | **Vec<String>** |  | 
**tier** | Option<**String**> |  | [optional]
**api_rate_limit_boosts** | Option<**i32**> |  | [optional]
**max_schedule_count** | Option<**i32**> |  | [optional]
**max_concurrent_actor_runs** | Option<**i32**> |  | [optional]
**plan_pricing** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Pricing details for this plan. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


