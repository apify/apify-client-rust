# \LastActorRunsLogApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_runs_last_log_get**](LastActorRunsLogApi.md#act_runs_last_log_get) | **GET** /v2/actors/{actorId}/runs/last/log | Get last Actor run's log



## act_runs_last_log_get

> String act_runs_last_log_get(actor_id, stream, download, raw)
Get last Actor run's log

Retrieves last Actor run's logs.  This endpoint is a shortcut for getting last Actor run's log. Same as [Get log](/api/v2/log-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
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

