# RequestQueue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | A unique identifier assigned to the request queue. | 
**name** | Option<**String**> | The name of the request queue. | [optional]
**user_id** | **String** | The ID of the user who owns the request queue. | 
**act_id** | Option<**String**> | The ID of the Actor that created this request queue. | [optional]
**act_run_id** | Option<**String**> | The ID of the Actor run that created this request queue. | [optional]
**created_at** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp when the request queue was created. | 
**modified_at** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp when the request queue was last modified. Modifications include adding, updating, or removing requests, as well as locking or unlocking requests in the request queue. | 
**accessed_at** | **chrono::DateTime<chrono::FixedOffset>** | The timestamp when the request queue was last accessed. | 
**total_request_count** | **i32** | The total number of requests in the request queue. | 
**handled_request_count** | **i32** | The number of requests that have been handled. | 
**pending_request_count** | **i32** | The number of requests that are pending and have not been handled yet. | 
**had_multiple_clients** | **bool** | Whether the request queue has been accessed by multiple different clients. | 
**console_url** | **String** | The URL to view the request queue in the Apify console. | 
**stats** | Option<[**models::RequestQueueStats**](RequestQueueStats.md)> |  | [optional]
**general_access** | Option<[**models::GeneralAccess**](GeneralAccess.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


