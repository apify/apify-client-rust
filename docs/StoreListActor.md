# StoreListActor

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**title** | **String** |  | 
**name** | **String** |  | 
**username** | **String** |  | 
**user_full_name** | **String** |  | 
**description** | **String** |  | 
**categories** | Option<**Vec<String>**> |  | [optional]
**notice** | Option<**String**> |  | [optional]
**picture_url** | Option<**String**> |  | [optional]
**user_picture_url** | Option<**String**> |  | [optional]
**url** | Option<**String**> |  | [optional]
**stats** | [**models::ActorStats**](ActorStats.md) |  | 
**current_pricing_info** | [**models::CurrentPricingInfo**](CurrentPricingInfo.md) |  | 
**is_white_listed_for_agentic_payments** | Option<**bool**> | Whether the Actor is whitelisted for agentic payment processing. | [optional]
**actor_review_count** | Option<**i32**> |  | [optional]
**actor_review_rating** | Option<**f64**> |  | [optional]
**bookmark_count** | Option<**i32**> |  | [optional]
**badge** | Option<**String**> |  | [optional]
**readme_summary** | Option<**String**> | A brief, LLM-generated readme summary | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


