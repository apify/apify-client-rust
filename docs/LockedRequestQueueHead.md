# LockedRequestQueueHead

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**limit** | **i32** | The maximum number of requests returned. | 
**queue_modified_at** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp when the request queue was last modified. Modifications include adding, updating, or removing requests, as well as locking or unlocking requests in the request queue. | 
**queue_has_locked_requests** | Option<**bool**> | Whether the request queue contains requests locked by any client (either the one calling the endpoint or a different one). | [optional]
**client_key** | Option<**String**> | The client key used for locking the requests. | [optional]
**had_multiple_clients** | **bool** | Whether the request queue has been accessed by multiple different clients. | 
**lock_secs** | **i32** | The number of seconds the locks will be held. | 
**items** | [**Vec<models::LockedHeadRequest>**](LockedHeadRequest.md) | The array of locked requests from the request queue head. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


