# Request

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**unique_key** | Option<**String**> | A unique key used for request de-duplication. Requests with the same unique key are considered identical. | [optional]
**url** | Option<**String**> | The URL of the request. | [optional]
**method** | Option<[**models::HttpMethod**](HttpMethod.md)> |  | [optional]
**retry_count** | Option<**i32**> | The number of times this request has been retried. | [optional]
**loaded_url** | Option<**String**> | The final URL that was loaded, after redirects (if any). | [optional]
**payload** | Option<[**models::RequestBasePayload**](RequestBasePayload.md)> |  | [optional]
**headers** | Option<**serde_json::Value**> | HTTP headers sent with the request. | [optional]
**user_data** | Option<**std::collections::HashMap<String, serde_json::Value>**> | Custom user data attached to the request. Can contain arbitrary fields. | [optional]
**no_retry** | Option<**bool**> | Indicates whether the request should not be retried if processing fails. | [optional]
**error_messages** | Option<**Vec<String>**> | Error messages recorded from failed processing attempts. | [optional]
**handled_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | The timestamp when the request was marked as handled, if applicable. | [optional]
**id** | Option<**String**> | A unique identifier assigned to the request. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


