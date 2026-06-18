# \ToolsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**tools_browser_info_delete**](ToolsApi.md#tools_browser_info_delete) | **DELETE** /v2/browser-info | Get browser info
[**tools_browser_info_get**](ToolsApi.md#tools_browser_info_get) | **GET** /v2/browser-info | Get browser info
[**tools_browser_info_post**](ToolsApi.md#tools_browser_info_post) | **POST** /v2/browser-info | Get browser info
[**tools_browser_info_put**](ToolsApi.md#tools_browser_info_put) | **PUT** /v2/browser-info | Get browser info
[**tools_decode_and_verify_post**](ToolsApi.md#tools_decode_and_verify_post) | **POST** /v2/tools/decode-and-verify | Decode and verify object
[**tools_encode_and_sign_post**](ToolsApi.md#tools_encode_and_sign_post) | **POST** /v2/tools/encode-and-sign | Encode and sign object



## tools_browser_info_delete

> models::BrowserInfoResponse tools_browser_info_delete(skip_headers, raw_headers)
Get browser info

Returns information about the HTTP request, including the client IP address, country code, request headers, and body length.  This endpoint is designed for proxy testing. It accepts any HTTP method so you can verify that your proxy correctly forwards requests of any type and that client IP addresses are anonymized. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**skip_headers** | Option<**bool**> | If `true` or `1`, the response omits the `headers` field. |  |
**raw_headers** | Option<**bool**> | If `true` or `1`, the response includes the `rawHeaders` field with the raw request headers. |  |

### Return type

[**models::BrowserInfoResponse**](BrowserInfoResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tools_browser_info_get

> models::BrowserInfoResponse tools_browser_info_get(skip_headers, raw_headers)
Get browser info

Returns information about the HTTP request, including the client IP address, country code, request headers, and body length.  This endpoint is designed for proxy testing. It accepts any HTTP method so you can verify that your proxy correctly forwards requests of any type and that client IP addresses are anonymized. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**skip_headers** | Option<**bool**> | If `true` or `1`, the response omits the `headers` field. |  |
**raw_headers** | Option<**bool**> | If `true` or `1`, the response includes the `rawHeaders` field with the raw request headers. |  |

### Return type

[**models::BrowserInfoResponse**](BrowserInfoResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tools_browser_info_post

> models::BrowserInfoResponse tools_browser_info_post(skip_headers, raw_headers)
Get browser info

Returns information about the HTTP request, including the client IP address, country code, request headers, and body length.  This endpoint is designed for proxy testing. It accepts any HTTP method so you can verify that your proxy correctly forwards requests of any type and that client IP addresses are anonymized. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**skip_headers** | Option<**bool**> | If `true` or `1`, the response omits the `headers` field. |  |
**raw_headers** | Option<**bool**> | If `true` or `1`, the response includes the `rawHeaders` field with the raw request headers. |  |

### Return type

[**models::BrowserInfoResponse**](BrowserInfoResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tools_browser_info_put

> models::BrowserInfoResponse tools_browser_info_put(skip_headers, raw_headers)
Get browser info

Returns information about the HTTP request, including the client IP address, country code, request headers, and body length.  This endpoint is designed for proxy testing. It accepts any HTTP method so you can verify that your proxy correctly forwards requests of any type and that client IP addresses are anonymized. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**skip_headers** | Option<**bool**> | If `true` or `1`, the response omits the `headers` field. |  |
**raw_headers** | Option<**bool**> | If `true` or `1`, the response includes the `rawHeaders` field with the raw request headers. |  |

### Return type

[**models::BrowserInfoResponse**](BrowserInfoResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tools_decode_and_verify_post

> models::DecodeAndVerifyResponse tools_decode_and_verify_post(decode_and_verify_request)
Decode and verify object

Decodes and verifies an encoded value previously created by the encode-and-sign endpoint. Returns the original decoded object along with information about the user who encoded it and whether that user is verified.  **Important**: The request must specify the `Content-Type: application/json` HTTP header. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**decode_and_verify_request** | [**DecodeAndVerifyRequest**](DecodeAndVerifyRequest.md) |  | [required] |

### Return type

[**models::DecodeAndVerifyResponse**](DecodeAndVerifyResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tools_encode_and_sign_post

> models::EncodeAndSignResponse tools_encode_and_sign_post(body)
Encode and sign object

Encodes and signs any JSON object. The encoded value includes a signature tied to the authenticated user's ID, which can later be verified using the decode-and-verify endpoint.  **Important**: The request must specify the `Content-Type: application/json` HTTP header. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body** | **serde_json::Value** |  | [required] |

### Return type

[**models::EncodeAndSignResponse**](EncodeAndSignResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

