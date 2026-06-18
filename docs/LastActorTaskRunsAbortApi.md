# \LastActorTaskRunsAbortApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actor_task_runs_last_abort_post**](LastActorTaskRunsAbortApi.md#actor_task_runs_last_abort_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/abort | Abort Actor task's last run



## actor_task_runs_last_abort_post

> models::RunResponse actor_task_runs_last_abort_post(actor_task_id, status, gracefully)
Abort Actor task's last run

Aborts the last run of the specified Actor task and returns an object that contains all the details about the run.  This endpoint is a shortcut for [Abort run](#/reference/actor-runs/abort-run/abort-run) on the Actor task's last run. Only runs that are starting or running are aborted. For runs with status `FINISHED`, `FAILED`, `ABORTING` and `TIMED-OUT` this call does nothing. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**gracefully** | Option<**bool**> | If true passed, the Actor run will abort gracefully. It will send `aborting` and `persistState` event into run and force-stop the run after 30 seconds. It is helpful in cases where you plan to resurrect the run later.  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

