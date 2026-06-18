# LockedHeadRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | A unique identifier assigned to the request. | 
**unique_key** | **String** | A unique key used for request de-duplication. Requests with the same unique key are considered identical. | 
**url** | **String** | The URL of the request. | 
**method** | Option<[**models::HttpMethod**](HttpMethod.md)> |  | [optional]
**retry_count** | Option<**i32**> | The number of times this request has been retried. | [optional]
**lock_expires_at** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp when the lock on this request expires. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


