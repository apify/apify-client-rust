# RequestQueueHead

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**limit** | **i32** | The maximum number of requests returned. | 
**queue_modified_at** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp when the request queue was last modified. Modifications include adding, updating, or removing requests, as well as locking or unlocking requests in the request queue. | 
**had_multiple_clients** | **bool** | Whether the request queue has been accessed by multiple different clients. | 
**items** | [**Vec<models::HeadRequest>**](HeadRequest.md) | The array of requests from the request queue head. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


