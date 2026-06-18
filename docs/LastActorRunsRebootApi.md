# \LastActorRunsRebootApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_runs_last_reboot_post**](LastActorRunsRebootApi.md#act_runs_last_reboot_post) | **POST** /v2/actors/{actorId}/runs/last/reboot | Reboot Actor's last run



## act_runs_last_reboot_post

> models::RunResponse act_runs_last_reboot_post(actor_id, status)
Reboot Actor's last run

Reboots the last run of the specified Actor and returns an object that contains all the details about the rebooted run.  This endpoint is a shortcut for [Reboot run](#/reference/actor-runs/reboot-run/reboot-run) on the Actor's last run. Only runs with status `RUNNING` can be rebooted. The run's container will be restarted, so any data not persisted in the key-value store, dataset, or request queue will be lost. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

