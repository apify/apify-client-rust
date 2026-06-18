# Dataset

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**name** | Option<**String**> |  | [optional]
**user_id** | **String** |  | 
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**modified_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**accessed_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**item_count** | **i32** |  | 
**clean_item_count** | **i32** |  | 
**act_id** | Option<**String**> |  | [optional]
**act_run_id** | Option<**String**> |  | [optional]
**fields** | Option<**Vec<String>**> |  | [optional]
**schema** | Option<**serde_json::Value**> | Defines the schema of items in your dataset, the full specification can be found in [Apify docs](/platform/actors/development/actor-definition/dataset-schema) | [optional]
**console_url** | **String** |  | 
**items_public_url** | Option<**String**> | A public link to access the dataset items directly. | [optional]
**url_signing_secret_key** | Option<**String**> | A secret key for generating signed public URLs. It is only provided to clients with WRITE permission for the dataset. | [optional]
**general_access** | Option<[**models::GeneralAccess**](GeneralAccess.md)> |  | [optional]
**stats** | Option<[**models::DatasetStats**](DatasetStats.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


