# BatchDeleteResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**processed_requests** | [**Vec<models::DeletedRequest>**](DeletedRequest.md) | Requests that were successfully deleted from the request queue. | 
**unprocessed_requests** | [**Vec<models::RequestDraft>**](RequestDraft.md) | Requests that failed to be deleted and can be retried. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


