# \LastActorTaskRunsMetamorphApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actor_task_runs_last_metamorph_post**](LastActorTaskRunsMetamorphApi.md#actor_task_runs_last_metamorph_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/metamorph | Metamorph Actor task's last run



## actor_task_runs_last_metamorph_post

> models::RunResponse actor_task_runs_last_metamorph_post(actor_task_id, target_actor_id, status, build)
Metamorph Actor task's last run

Transforms the last run of the specified Actor task into a run of another Actor with a new input.  This endpoint is a shortcut for [Metamorph run](#/reference/actor-runs/metamorph-run/metamorph-run) on the Actor task's last run. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**target_actor_id** | **String** | ID of a target Actor that the run should be transformed into. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**build** | Option<**String**> | Optional build of the target Actor.  It can be either a build tag or build number. By default, the run uses the build specified in the default run configuration for the target Actor (typically `latest`).  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

