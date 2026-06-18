# \ActorRunsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actor_run_abort_post**](ActorRunsApi.md#actor_run_abort_post) | **POST** /v2/actor-runs/{runId}/abort | Abort run
[**actor_run_delete**](ActorRunsApi.md#actor_run_delete) | **DELETE** /v2/actor-runs/{runId} | Delete run
[**actor_run_get**](ActorRunsApi.md#actor_run_get) | **GET** /v2/actor-runs/{runId} | Get run
[**actor_run_log_get**](ActorRunsApi.md#actor_run_log_get) | **GET** /v2/actor-runs/{runId}/log | Get run's log
[**actor_run_metamorph_post**](ActorRunsApi.md#actor_run_metamorph_post) | **POST** /v2/actor-runs/{runId}/metamorph | Metamorph run
[**actor_run_put**](ActorRunsApi.md#actor_run_put) | **PUT** /v2/actor-runs/{runId} | Update run
[**actor_run_reboot_post**](ActorRunsApi.md#actor_run_reboot_post) | **POST** /v2/actor-runs/{runId}/reboot | Reboot run
[**actor_runs_get**](ActorRunsApi.md#actor_runs_get) | **GET** /v2/actor-runs | Get user runs list
[**post_charge_run**](ActorRunsApi.md#post_charge_run) | **POST** /v2/actor-runs/{runId}/charge | Charge events in run
[**post_resurrect_run**](ActorRunsApi.md#post_resurrect_run) | **POST** /v2/actor-runs/{runId}/resurrect | Resurrect run



## actor_run_abort_post

> models::RunResponse actor_run_abort_post(run_id, gracefully)
Abort run

Aborts an Actor run and returns an object that contains all the details about the run.  Only runs that are starting or running are aborted. For runs with status `FINISHED`, `FAILED`, `ABORTING` and `TIMED-OUT` this call does nothing. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |
**gracefully** | Option<**bool**> | If true passed, the Actor run will abort gracefully. It will send `aborting` and `persistState` event into run and force-stop the run after 30 seconds. It is helpful in cases where you plan to resurrect the run later.  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_run_delete

> actor_run_delete(run_id)
Delete run

Delete the run. Only finished runs can be deleted. Only the person or organization that initiated the run can delete it. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_run_get

> models::RunResponse actor_run_get(run_id, wait_for_finish)
Get run

This is not a single endpoint, but an entire group of endpoints that lets you retrieve the run or any of its default storages.  ##### Convenience endpoints for Actor run default storages  * [Dataset](/api/v2/default-dataset)  * [Key-value store](/api/v2/default-key-value-store)  * [Request queue](/api/v2/default-request-queue)  Gets an object that contains all the details about a specific run of an Actor.  By passing the optional `waitForFinish` parameter the API endpoint will synchronously wait for the run to finish. This is useful to avoid periodic polling when waiting for Actor run to complete. Note that the first response after completion can still show preliminary `stats`, costs, and event counts. For stable figures, wait about 10 seconds and call the endpoint again.  This endpoint does not require the authentication token. Instead, calls are authenticated using a hard-to-guess ID of the run. However, if you access the endpoint without the token, certain attributes, such as `usageUsd` and `usageTotalUsd`, will be hidden. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |
**wait_for_finish** | Option<**f64**> | The maximum number of seconds the server waits for the run to finish. By default it is `0`, the maximum value is `60`. <!-- MAX_ACTOR_JOB_ASYNC_WAIT_SECS --> If the run finishes in time then the returned run object will have a terminal status (e.g. `SUCCEEDED`), otherwise it will have a transitional status (e.g. `RUNNING`).  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_run_log_get

> String actor_run_log_get(run_id, stream, download, raw)
Get run's log

Retrieves Actor run's logs.  This endpoint is a shortcut for getting the run's log. Same as [Get log](/api/v2/log-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |
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


## actor_run_metamorph_post

> models::RunResponse actor_run_metamorph_post(run_id, target_actor_id, build)
Metamorph run

Transforms an Actor run into a run of another Actor with a new input.  This is useful if you want to use another Actor to finish the work of your current Actor run, without the need to create a completely new run and waiting for its finish.  For the users of your Actors, the metamorph operation is transparent, they will just see your Actor got the work done.  Internally, the system stops the Docker container corresponding to the Actor run and starts a new container using a different Docker image.  All the default storages are preserved and the new input is stored under the `INPUT-METAMORPH-1` key in the same default key-value store.  For more information, see the [Actor docs](https://docs.apify.com/platform/actors/development/programming-interface/metamorph). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |
**target_actor_id** | **String** | ID of a target Actor that the run should be transformed into. | [required] |
**build** | Option<**String**> | Optional build of the target Actor.  It can be either a build tag or build number. By default, the run uses the build specified in the default run configuration for the target Actor (typically `latest`).  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_run_put

> models::RunResponse actor_run_put(run_id, update_run_request)
Update run

This endpoint can be used to update both the run's status message and to configure its general resource access level.  **Status message:**  You can set a single status message on your run that will be displayed in the Apify Console UI. During an Actor run, you will typically do this in order to inform users of your Actor about the Actor's progress.  The request body must contain `runId` and `statusMessage` properties. The `isStatusMessageTerminal` property is optional and it indicates if the status message is the very last one. In the absence of a status message, the platform will try to substitute sensible defaults.  **General resource access:**  You can also update the run's general resource access setting, which determines who can view the run and its related data.  Allowed values:  * `FOLLOW_USER_SETTING` - The run inherits the general access setting from the account level. * `ANYONE_WITH_ID_CAN_READ` - The run can be viewed anonymously by anyone who has its ID. * `RESTRICTED` - Only users with explicit access to the resource can access the run.  When a run is accessible anonymously, all of the run's default storages and logs also become accessible anonymously. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |
**update_run_request** | [**UpdateRunRequest**](UpdateRunRequest.md) |  | [required] |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_run_reboot_post

> models::RunResponse actor_run_reboot_post(run_id)
Reboot run

Reboots an Actor run and returns an object that contains all the details about the rebooted run.  Only runs that are running, i.e. runs with status `RUNNING` can be rebooted.  The run's container will be restarted, so any data not persisted in the key-value store, dataset, or request queue will be lost. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_runs_get

> models::ListOfRunsResponse actor_runs_get(offset, limit, desc, status, started_after, started_before)
Get user runs list

Gets a list of all runs for a user. The response is a list of objects, where each object contains basic information about a single Actor run.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 array elements.  By default, the records are sorted by the `startedAt` field in ascending order. Therefore, you can use pagination to incrementally fetch all records while new ones are still being created. To sort the records in descending order, use `desc=1` parameter. You can also filter runs by `startedAt`` and `status`` fields ([available statuses](https://docs.apify.com/platform/actors/running/runs-and-builds#lifecycle)). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `startedAt` field in descending order. By default, they are sorted in ascending order.  |  |
**status** | Option<[**Vec<String>**](String.md)> | Single status or comma-separated list of statuses, see ([available statuses](https://docs.apify.com/platform/actors/running/runs-and-builds#lifecycle)). Used to filter runs by the specified statuses only.  |  |
**started_after** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Filter runs that started after the specified date and time (inclusive). The value must be a valid ISO 8601 datetime string (UTC).  |  |
**started_before** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Filter runs that started before the specified date and time (inclusive). The value must be a valid ISO 8601 datetime string (UTC).  |  |

### Return type

[**models::ListOfRunsResponse**](ListOfRunsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_charge_run

> serde_json::Value post_charge_run(run_id, charge_run_request, idempotency_key)
Charge events in run

Charge for events in the run of your [pay per event Actor](https://docs.apify.com/platform/actors/running/actors-in-store#pay-per-event). The event you are charging for must be one of the configured events in your Actor. If the Actor is not set up as pay per event, or if the event is not configured, the endpoint will return an error. The endpoint must be called from the Actor run itself, with the same API token that the run was started with.  :::info Learn more about pay-per-event pricing  For more details about pay-per-event (PPE) pricing, refer to our [PPE documentation](/platform/actors/publishing/monetize/pay-per-event).  ::: 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |
**charge_run_request** | [**ChargeRunRequest**](ChargeRunRequest.md) | Define which event, and how many times, you want to charge for. | [required] |
**idempotency_key** | Option<**String**> | Always pass a unique idempotency key (any unique string) for each charge to avoid double charging in case of retries or network errors. |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_resurrect_run

> models::RunResponse post_resurrect_run(run_id, build, timeout, memory, max_items, max_total_charge_usd, restart_on_error)
Resurrect run

Resurrects a finished Actor run and returns an object that contains all the details about the resurrected run. Only finished runs, i.e. runs with status `FINISHED`, `FAILED`, `ABORTED` and `TIMED-OUT` can be resurrected. Run status will be updated to RUNNING and its container will be restarted with the same storages (the same behaviour as when the run gets migrated to the new server).  For more information, see the [Actor docs](https://docs.apify.com/platform/actors/running/runs-and-builds#resurrection-of-finished-run). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**run_id** | **String** | Actor run ID. | [required] |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run is resurrected with the same build it originally used. Specifically, if a run was first started with the `latest` tag, which resolves to version `0.0.3` at the time, a run resurrected without this parameter will continue running with `0.0.3`, even if `latest` already points to a newer build.  |  |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout specified in the run that is being resurrected.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit specified in the run that is being resurrected.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**max_total_charge_usd** | Option<**f64**> | Specifies the maximum cost of the run. This parameter is useful for pay-per-event Actors, as it allows you to limit the amount charged to your subscription. You can access the maximum cost in your Actor by using the `ACTOR_MAX_TOTAL_CHARGE_USD` environment variable.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the resurrected run will be restarted if it fails. By default, the resurrected run uses the same setting as before.  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

