# \LogsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**log_get**](LogsApi.md#log_get) | **GET** /v2/logs/{buildOrRunId} | Get log



## log_get

> String log_get(build_or_run_id, stream, download, raw)
Get log

Retrieves logs for a specific Actor build or run. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**build_or_run_id** | **String** | ID of the Actor build or run. | [required] |
**stream** | Option<**bool**> | If `true` or `1` then the logs will be streamed as long as the run or build is running.  |  |
**download** | Option<**bool**> | If `true` or `1` then the web browser will download the log file rather than open it in a tab.  |  |
**raw** | Option<**bool**> | If `true` or `1`, the logs will be kept verbatim. By default, the API removes ANSI escape codes from the logs, keeping only printable characters.  |  |

### Return type

**String**

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/plain, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

