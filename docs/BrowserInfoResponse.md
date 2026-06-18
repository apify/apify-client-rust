# BrowserInfoResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**method** | **String** | HTTP method of the request. | 
**client_ip** | Option<**String**> | IP address of the client. | 
**country_code** | Option<**String**> | Two-letter country code resolved from the client IP address. | 
**body_length** | **i32** | Length of the request body in bytes. | 
**headers** | Option<[**std::collections::HashMap<String, models::BrowserInfoResponseHeadersValue>**](BrowserInfoResponseHeadersValue.md)> | Request headers. Omitted when `skipHeaders=true`.  | [optional]
**raw_headers** | Option<**Vec<String>**> | Raw request headers as a flat list of alternating name/value strings. Included only when `rawHeaders=true`.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


