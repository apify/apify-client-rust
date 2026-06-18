# KeyValueStore

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**name** | Option<**String**> |  | [optional]
**user_id** | Option<**String**> |  | [optional]
**username** | Option<**String**> |  | [optional]
**created_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**modified_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**accessed_at** | **chrono::DateTime<chrono::FixedOffset>** |  | 
**act_id** | Option<**String**> |  | [optional]
**act_run_id** | Option<**String**> |  | [optional]
**console_url** | Option<**String**> |  | [optional]
**keys_public_url** | Option<**String**> | A public link to access keys of the key-value store directly. | [optional]
**records_public_url** | Option<**String**> | A public link to access records of the key-value store directly. | [optional]
**schema** | Option<**serde_json::Value**> | Optional JSON schema describing the keys stored in the key-value store. | [optional]
**url_signing_secret_key** | Option<**String**> | A secret key for generating signed public URLs. It is only provided to clients with WRITE permission for the key-value store. | [optional]
**general_access** | Option<[**models::GeneralAccess**](GeneralAccess.md)> |  | [optional]
**stats** | Option<[**models::KeyValueStoreStats**](KeyValueStoreStats.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


