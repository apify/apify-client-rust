# AddedRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**request_id** | **String** | A unique identifier assigned to the request. | 
**unique_key** | **String** | A unique key used for request de-duplication. Requests with the same unique key are considered identical. | 
**was_already_present** | **bool** | Indicates whether a request with the same unique key already existed in the request queue. If true, no new request was created. | 
**was_already_handled** | **bool** | Indicates whether a request with the same unique key has already been processed by the request queue. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


