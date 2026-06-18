# \ActorsActorRunsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_run_abort_post**](ActorsActorRunsApi.md#act_run_abort_post) | **POST** /v2/actors/{actorId}/runs/{runId}/abort | Abort run
[**act_run_get**](ActorsActorRunsApi.md#act_run_get) | **GET** /v2/actors/{actorId}/runs/{runId} | Get run
[**act_run_metamorph_post**](ActorsActorRunsApi.md#act_run_metamorph_post) | **POST** /v2/actors/{actorId}/runs/{runId}/metamorph | Metamorph run
[**act_run_resurrect_post**](ActorsActorRunsApi.md#act_run_resurrect_post) | **POST** /v2/actors/{actorId}/runs/{runId}/resurrect | Resurrect run
[**act_run_sync_get**](ActorsActorRunsApi.md#act_run_sync_get) | **GET** /v2/actors/{actorId}/run-sync | Run Actor synchronously without input
[**act_run_sync_get_dataset_items_get**](ActorsActorRunsApi.md#act_run_sync_get_dataset_items_get) | **GET** /v2/actors/{actorId}/run-sync-get-dataset-items | Run Actor synchronously without input and get dataset items
[**act_run_sync_get_dataset_items_post**](ActorsActorRunsApi.md#act_run_sync_get_dataset_items_post) | **POST** /v2/actors/{actorId}/run-sync-get-dataset-items | Run Actor synchronously and get dataset items
[**act_run_sync_post**](ActorsActorRunsApi.md#act_run_sync_post) | **POST** /v2/actors/{actorId}/run-sync | Run Actor synchronously and return output
[**act_runs_get**](ActorsActorRunsApi.md#act_runs_get) | **GET** /v2/actors/{actorId}/runs | Get list of runs
[**act_runs_last_get**](ActorsActorRunsApi.md#act_runs_last_get) | **GET** /v2/actors/{actorId}/runs/last | Get last run
[**act_runs_post**](ActorsActorRunsApi.md#act_runs_post) | **POST** /v2/actors/{actorId}/runs | Run Actor



## act_run_abort_post

> models::RunResponse act_run_abort_post(actor_id, run_id, gracefully)
Abort run

**[DEPRECATED]** API endpoints related to run of the Actor were moved under new namespace [`actor-runs`](#/reference/actor-runs). Aborts an Actor run and returns an object that contains all the details about the run.  Only runs that are starting or running are aborted. For runs with status `FINISHED`, `FAILED`, `ABORTING` and `TIMED-OUT` this call does nothing. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
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


## act_run_get

> models::RunResponse act_run_get(actor_id, run_id, wait_for_finish)
Get run

**[DEPRECATED]** API endpoints related to run of the Actor were moved under new namespace [`actor-runs`](#/reference/actor-runs).  Gets an object that contains all the details about a specific run of an Actor.  By passing the optional `waitForFinish` parameter the API endpoint will synchronously wait for the run to finish. This is useful to avoid periodic polling when waiting for Actor run to complete.  This endpoint does not require the authentication token. Instead, calls are authenticated using a hard-to-guess ID of the run. However, if you access the endpoint without the token, certain attributes, such as `usageUsd` and `usageTotalUsd`, will be hidden. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
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


## act_run_metamorph_post

> models::RunResponse act_run_metamorph_post(actor_id, run_id, target_actor_id, build)
Metamorph run

**[DEPRECATED]** API endpoints related to run of the Actor were moved under new namespace [`actor-runs`](#/reference/actor-runs). Transforms an Actor run into a run of another Actor with a new input.  This is useful if you want to use another Actor to finish the work of your current Actor run, without the need to create a completely new run and waiting for its finish. For the users of your Actors, the metamorph operation is transparent, they will just see your Actor got the work done.  There is a limit on how many times you can metamorph a single run. You can check the limit in [the Actor runtime limits](https://docs.apify.com/platform/limits#actor-limits).  Internally, the system stops the Docker container corresponding to the Actor run and starts a new container using a different Docker image. All the default storages are preserved and the new input is stored under the `INPUT-METAMORPH-1` key in the same default key-value store.  For more information, see the [Actor docs](https://docs.apify.com/platform/actors/development/programming-interface/metamorph). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
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


## act_run_resurrect_post

> models::RunResponse act_run_resurrect_post(actor_id, run_id, build, timeout, memory, restart_on_error)
Resurrect run

**[DEPRECATED]** API endpoints related to run of the Actor were moved under new namespace [`actor-runs`](#/reference/actor-runs).Resurrects a finished Actor run and returns an object that contains all the details about the resurrected run.  Only finished runs, i.e. runs with status `FINISHED`, `FAILED`, `ABORTED` and `TIMED-OUT` can be resurrected. Run status will be updated to RUNNING and its container will be restarted with the same storages (the same behaviour as when the run gets migrated to the new server).  For more information, see the [Actor docs](https://docs.apify.com/platform/actors/running/runs-and-builds#resurrection-of-finished-run). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**run_id** | **String** | Actor run ID. | [required] |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run is resurrected with the same build it originally used. Specifically, if a run was first started with the `latest` tag, which resolves to version `0.0.3` at the time, a run resurrected without this parameter will continue running with `0.0.3`, even if `latest` already points to a newer build.  |  |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout specified in the run that is being resurrected.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit specified in the run that is being resurrected.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the resurrected run will be restarted if it fails. By default, the resurrected run uses the same setting as before.  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_run_sync_get

> serde_json::Value act_run_sync_get(actor_id, output_record_key, timeout, memory, max_items, max_total_charge_usd, restart_on_error, build, webhooks)
Run Actor synchronously without input

Runs a specific Actor and returns its output. The run must finish in 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds otherwise the API endpoint returns a timeout error. The Actor is not passed any input.  Beware that it might be impossible to maintain an idle HTTP connection for a long period of time, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout. If the connection breaks, you will not receive any information about the run and its status.  To run the Actor asynchronously, use the [Run Actor](#/reference/actors/run-collection/run-actor) API endpoint instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**output_record_key** | Option<**String**> | Key of the record from run's default key-value store to be returned in the response. By default, it is `OUTPUT`.  |  |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**max_total_charge_usd** | Option<**f64**> | Specifies the maximum cost of the run. This parameter is useful for pay-per-event Actors, as it allows you to limit the amount charged to your subscription. You can access the maximum cost in your Actor by using the `ACTOR_MAX_TOTAL_CHARGE_USD` environment variable.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the run will be restarted if it fails.  |  |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run uses the build from its configuration (typically `latest`).  |  |
**webhooks** | Option<**String**> | Specifies optional webhooks associated with the Actor run, which can be used to receive a notification e.g. when the Actor finished or failed. The value is a Base64-encoded JSON array whose items follow the WebhookRepresentation schema. For more information, see [Webhooks documentation](https://docs.apify.com/platform/integrations/webhooks).  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_run_sync_get_dataset_items_get

> Vec<serde_json::Value> act_run_sync_get_dataset_items_get(actor_id, timeout, memory, max_items, max_total_charge_usd, restart_on_error, build, webhooks, format, clean, offset, limit, fields, output_fields, omit, unwind, flatten, desc, attachment, delimiter, bom, xml_root, xml_row, skip_header_row, skip_hidden, skip_empty, simplified, view, skip_failed_pages, feed_title, feed_description)
Run Actor synchronously without input and get dataset items

Runs a specific Actor and returns its dataset items. The run must finish in 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds otherwise the API endpoint returns a timeout error. The Actor is not passed any input.  It allows to send all possible options in parameters from [Get Dataset Items](#/reference/datasets/item-collection/get-items) API endpoint.  Beware that it might be impossible to maintain an idle HTTP connection for a long period of time, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout. If the connection breaks, you will not receive any information about the run and its status.  To run the Actor asynchronously, use the [Run Actor](#/reference/actors/run-collection/run-actor) API endpoint instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**max_total_charge_usd** | Option<**f64**> | Specifies the maximum cost of the run. This parameter is useful for pay-per-event Actors, as it allows you to limit the amount charged to your subscription. You can access the maximum cost in your Actor by using the `ACTOR_MAX_TOTAL_CHARGE_USD` environment variable.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the run will be restarted if it fails.  |  |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run uses the build from its configuration (typically `latest`).  |  |
**webhooks** | Option<**String**> | Specifies optional webhooks associated with the Actor run, which can be used to receive a notification e.g. when the Actor finished or failed. The value is a Base64-encoded JSON array whose items follow the WebhookRepresentation schema. For more information, see [Webhooks documentation](https://docs.apify.com/platform/integrations/webhooks).  |  |
**format** | Option<**String**> | Format of the results, possible values are: `json`, `jsonl`, `csv`, `html`, `xlsx`, `xml` and `rss`. The default value is `json`.  |  |
**clean** | Option<**bool**> | If `true` or `1` then the API endpoint returns only non-empty items and skips hidden fields (i.e. fields starting with the # character). The `clean` parameter is just a shortcut for `skipHidden=true` and `skipEmpty=true` parameters. Note that since some objects might be skipped from the output, that the result might contain less items than the `limit` value.  |  |
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. By default there is no limit. |  |
**fields** | Option<**String**> | A comma-separated list of fields which should be picked from the items, only these fields will remain in the resulting record objects. Note that the fields in the outputted items are sorted the same way as they are specified in the `fields` query parameter. You can use this feature to effectively fix the output format.  |  |
**output_fields** | Option<**String**> | A comma-separated list of output field names that positionally rename the fields specified in the `fields` parameter. For example, `?fields=headline,url&outputFields=title,link` renames `headline` to `title` and `url` to `link` in the output. The number of names in `outputFields` must match the number of names in `fields`. Requires the `fields` parameter to be specified as well.  |  |
**omit** | Option<**String**> | A comma-separated list of fields which should be omitted from the items. |  |
**unwind** | Option<**String**> | A comma-separated list of fields which should be unwound, in order which they should be processed. Each field should be either an array or an object. If the field is an array then every element of the array will become a separate record and merged with parent object. If the unwound field is an object then it is merged with the parent object. If the unwound field is missing or its value is neither an array nor an object and therefore cannot be merged with a parent object then the item gets preserved as it is. Note that the unwound items ignore the `desc` parameter.  |  |
**flatten** | Option<**String**> | A comma-separated list of fields which should transform nested objects into flat structures.  For example, with `flatten=\"foo\"` the object `{\"foo\":{\"bar\": \"hello\"}}` is turned into `{\"foo.bar\": \"hello\"}`.  The original object with properties is replaced with the flattened object.  |  |
**desc** | Option<**bool**> | By default, results are returned in the same order as they were stored. To reverse the order, set this parameter to `true` or `1`.  |  |
**attachment** | Option<**bool**> | If `true` or `1` then the response will define the `Content-Disposition: attachment` header, forcing a web browser to download the file rather than to display it. By default this header is not present.  |  |
**delimiter** | Option<**String**> | A delimiter character for CSV files, only used if `format=csv`. You might need to URL-encode the character (e.g. use `%09` for tab or `%3B` for semicolon). The default delimiter is a simple comma (`,`).  |  |
**bom** | Option<**bool**> | All text responses are encoded in UTF-8 encoding. By default, the `format=csv` files are prefixed with the UTF-8 Byte Order Mark (BOM), while `json`, `jsonl`, `xml`, `html` and `rss` files are not.  If you want to override this default behavior, specify `bom=1` query parameter to include the BOM or `bom=0` to skip it.  |  |
**xml_root** | Option<**String**> | Overrides default root element name of `xml` output. By default the root element is `items`.  |  |
**xml_row** | Option<**String**> | Overrides default element name that wraps each page or page function result object in `xml` output. By default the element name is `item`.  |  |
**skip_header_row** | Option<**bool**> | If `true` or `1` then header row in the `csv` format is skipped. |  |
**skip_hidden** | Option<**bool**> | If `true` or `1` then hidden fields are skipped from the output, i.e. fields starting with the `#` character.  |  |
**skip_empty** | Option<**bool**> | If `true` or `1` then empty items are skipped from the output.  Note that if used, the results might contain less items than the limit value.  |  |
**simplified** | Option<**bool**> | If `true` or `1` then, the endpoint applies the `fields=url,pageFunctionResult,errorInfo` and `unwind=pageFunctionResult` query parameters. This feature is used to emulate simplified results provided by the legacy Apify Crawler product and it's not recommended to use it in new integrations.  |  |
**view** | Option<**String**> | Defines the view configuration for dataset items based on the schema definition. This parameter determines how the data will be filtered and presented. For complete specification details, see the [dataset schema documentation](/platform/actors/development/actor-definition/dataset-schema).  |  |
**skip_failed_pages** | Option<**bool**> | If `true` or `1` then, the all the items with errorInfo property will be skipped from the output.  This feature is here to emulate functionality of API version 1 used for the legacy Apify Crawler product and it's not recommended to use it in new integrations.  |  |
**feed_title** | Option<**String**> | Overrides the auto-generated RSS channel `<title>` element. Only used when `format=rss`. If not provided, the title defaults to `Dataset <label>`.  |  |
**feed_description** | Option<**String**> | Overrides the auto-generated RSS channel `<description>` element. Only used when `format=rss`. If not provided, the description defaults to `Items in dataset with id \"<datasetId>\".`  |  |

### Return type

[**Vec<serde_json::Value>**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_run_sync_get_dataset_items_post

> Vec<serde_json::Value> act_run_sync_get_dataset_items_post(actor_id, body, timeout, memory, max_items, max_total_charge_usd, restart_on_error, build, webhooks, format, clean, offset, limit, fields, output_fields, omit, unwind, flatten, desc, attachment, delimiter, bom, xml_root, xml_row, skip_header_row, skip_hidden, skip_empty, simplified, view, skip_failed_pages, feed_title, feed_description)
Run Actor synchronously and get dataset items

Runs a specific Actor and returns its dataset items.  The POST payload including its `Content-Type` header is passed as `INPUT` to the Actor (usually `application/json`). The HTTP response contains the Actors dataset items, while the format of items depends on specifying dataset items' `format` parameter.  You can send all the same options in parameters as the [Get Dataset Items](#/reference/datasets/item-collection/get-items) API endpoint.  The Actor is started with the default options; you can override them using URL query parameters. If the Actor run exceeds 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds, the HTTP response will return the 408 status code (Request Timeout).  Beware that it might be impossible to maintain an idle HTTP connection for a long period of time, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout. If the connection breaks, you will not receive any information about the run and its status.  To run the Actor asynchronously, use the [Run Actor](#/reference/actors/run-collection/run-actor) API endpoint instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**body** | **serde_json::Value** |  | [required] |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**max_total_charge_usd** | Option<**f64**> | Specifies the maximum cost of the run. This parameter is useful for pay-per-event Actors, as it allows you to limit the amount charged to your subscription. You can access the maximum cost in your Actor by using the `ACTOR_MAX_TOTAL_CHARGE_USD` environment variable.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the run will be restarted if it fails.  |  |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run uses the build from its configuration (typically `latest`).  |  |
**webhooks** | Option<**String**> | Specifies optional webhooks associated with the Actor run, which can be used to receive a notification e.g. when the Actor finished or failed. The value is a Base64-encoded JSON array whose items follow the WebhookRepresentation schema. For more information, see [Webhooks documentation](https://docs.apify.com/platform/integrations/webhooks).  |  |
**format** | Option<**String**> | Format of the results, possible values are: `json`, `jsonl`, `csv`, `html`, `xlsx`, `xml` and `rss`. The default value is `json`.  |  |
**clean** | Option<**bool**> | If `true` or `1` then the API endpoint returns only non-empty items and skips hidden fields (i.e. fields starting with the # character). The `clean` parameter is just a shortcut for `skipHidden=true` and `skipEmpty=true` parameters. Note that since some objects might be skipped from the output, that the result might contain less items than the `limit` value.  |  |
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. By default there is no limit. |  |
**fields** | Option<**String**> | A comma-separated list of fields which should be picked from the items, only these fields will remain in the resulting record objects. Note that the fields in the outputted items are sorted the same way as they are specified in the `fields` query parameter. You can use this feature to effectively fix the output format.  |  |
**output_fields** | Option<**String**> | A comma-separated list of output field names that positionally rename the fields specified in the `fields` parameter. For example, `?fields=headline,url&outputFields=title,link` renames `headline` to `title` and `url` to `link` in the output. The number of names in `outputFields` must match the number of names in `fields`. Requires the `fields` parameter to be specified as well.  |  |
**omit** | Option<**String**> | A comma-separated list of fields which should be omitted from the items. |  |
**unwind** | Option<**String**> | A comma-separated list of fields which should be unwound, in order which they should be processed. Each field should be either an array or an object. If the field is an array then every element of the array will become a separate record and merged with parent object. If the unwound field is an object then it is merged with the parent object. If the unwound field is missing or its value is neither an array nor an object and therefore cannot be merged with a parent object then the item gets preserved as it is. Note that the unwound items ignore the `desc` parameter.  |  |
**flatten** | Option<**String**> | A comma-separated list of fields which should transform nested objects into flat structures.  For example, with `flatten=\"foo\"` the object `{\"foo\":{\"bar\": \"hello\"}}` is turned into `{\"foo.bar\": \"hello\"}`.  The original object with properties is replaced with the flattened object.  |  |
**desc** | Option<**bool**> | By default, results are returned in the same order as they were stored. To reverse the order, set this parameter to `true` or `1`.  |  |
**attachment** | Option<**bool**> | If `true` or `1` then the response will define the `Content-Disposition: attachment` header, forcing a web browser to download the file rather than to display it. By default this header is not present.  |  |
**delimiter** | Option<**String**> | A delimiter character for CSV files, only used if `format=csv`. You might need to URL-encode the character (e.g. use `%09` for tab or `%3B` for semicolon). The default delimiter is a simple comma (`,`).  |  |
**bom** | Option<**bool**> | All text responses are encoded in UTF-8 encoding. By default, the `format=csv` files are prefixed with the UTF-8 Byte Order Mark (BOM), while `json`, `jsonl`, `xml`, `html` and `rss` files are not.  If you want to override this default behavior, specify `bom=1` query parameter to include the BOM or `bom=0` to skip it.  |  |
**xml_root** | Option<**String**> | Overrides default root element name of `xml` output. By default the root element is `items`.  |  |
**xml_row** | Option<**String**> | Overrides default element name that wraps each page or page function result object in `xml` output. By default the element name is `item`.  |  |
**skip_header_row** | Option<**bool**> | If `true` or `1` then header row in the `csv` format is skipped. |  |
**skip_hidden** | Option<**bool**> | If `true` or `1` then hidden fields are skipped from the output, i.e. fields starting with the `#` character.  |  |
**skip_empty** | Option<**bool**> | If `true` or `1` then empty items are skipped from the output.  Note that if used, the results might contain less items than the limit value.  |  |
**simplified** | Option<**bool**> | If `true` or `1` then, the endpoint applies the `fields=url,pageFunctionResult,errorInfo` and `unwind=pageFunctionResult` query parameters. This feature is used to emulate simplified results provided by the legacy Apify Crawler product and it's not recommended to use it in new integrations.  |  |
**view** | Option<**String**> | Defines the view configuration for dataset items based on the schema definition. This parameter determines how the data will be filtered and presented. For complete specification details, see the [dataset schema documentation](/platform/actors/development/actor-definition/dataset-schema).  |  |
**skip_failed_pages** | Option<**bool**> | If `true` or `1` then, the all the items with errorInfo property will be skipped from the output.  This feature is here to emulate functionality of API version 1 used for the legacy Apify Crawler product and it's not recommended to use it in new integrations.  |  |
**feed_title** | Option<**String**> | Overrides the auto-generated RSS channel `<title>` element. Only used when `format=rss`. If not provided, the title defaults to `Dataset <label>`.  |  |
**feed_description** | Option<**String**> | Overrides the auto-generated RSS channel `<description>` element. Only used when `format=rss`. If not provided, the description defaults to `Items in dataset with id \"<datasetId>\".`  |  |

### Return type

[**Vec<serde_json::Value>**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_run_sync_post

> serde_json::Value act_run_sync_post(actor_id, body, output_record_key, timeout, memory, max_items, max_total_charge_usd, restart_on_error, build, webhooks)
Run Actor synchronously and return output

Runs a specific Actor and returns its output.  The POST payload including its `Content-Type` header is passed as `INPUT` to the Actor (usually <code>application/json</code>). The HTTP response contains Actors `OUTPUT` record from its default key-value store.  The Actor is started with the default options; you can override them using various URL query parameters. If the Actor run exceeds 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds, the HTTP response will have status 408 (Request Timeout).  Beware that it might be impossible to maintain an idle HTTP connection for a long period of time, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout. If the connection breaks, you will not receive any information about the run and its status.  To run the Actor asynchronously, use the [Run Actor](#/reference/actors/run-collection/run-actor) API endpoint instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**body** | **serde_json::Value** |  | [required] |
**output_record_key** | Option<**String**> | Key of the record from run's default key-value store to be returned in the response. By default, it is `OUTPUT`.  |  |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**max_total_charge_usd** | Option<**f64**> | Specifies the maximum cost of the run. This parameter is useful for pay-per-event Actors, as it allows you to limit the amount charged to your subscription. You can access the maximum cost in your Actor by using the `ACTOR_MAX_TOTAL_CHARGE_USD` environment variable.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the run will be restarted if it fails.  |  |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run uses the build from its configuration (typically `latest`).  |  |
**webhooks** | Option<**String**> | Specifies optional webhooks associated with the Actor run, which can be used to receive a notification e.g. when the Actor finished or failed. The value is a Base64-encoded JSON array whose items follow the WebhookRepresentation schema. For more information, see [Webhooks documentation](https://docs.apify.com/platform/integrations/webhooks).  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_get

> models::ListOfRunsResponse act_runs_get(actor_id, offset, limit, desc, status, started_after, started_before)
Get list of runs

Gets the list of runs of a specific Actor. The response is a list of objects, where each object contains basic information about a single Actor run.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 array elements.  By default, the records are sorted by the `startedAt` field in ascending order, therefore you can use pagination to incrementally fetch all records while new ones are still being created. To sort the records in descending order, use `desc=1` parameter. You can also filter runs by status ([available statuses](https://docs.apify.com/platform/actors/running/runs-and-builds#lifecycle)). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
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


## act_runs_last_get

> models::RunResponse act_runs_last_get(actor_id, status, wait_for_finish)
Get last run

This is not a single endpoint, but an entire group of endpoints that lets you to retrieve and manage the last run of given Actor or any of its default storages. All the endpoints require an authentication token.  The base path represents the last Actor run object is:  `/v2/actors/{actorId}/runs/last{?token,status}`  Using the `status` query parameter you can ensure to only get a run with a certain status (e.g. `status=SUCCEEDED`). The output of this endpoint and other query parameters are the same as in the [Run object](#/reference/actors/run-object) endpoint.  ##### Convenience endpoints for last Actor run  * [Dataset](/api/v2/last-actor-runs-default-dataset)  * [Key-value store](/api/v2/last-actor-runs-default-key-value-store)  * [Request queue](/api/v2/last-actor-runs-default-request-queue)  * [Log](/api/v2/last-actor-runs-log) 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**wait_for_finish** | Option<**f64**> | The maximum number of seconds the server waits for the run to finish. By default it is `0`, the maximum value is `60`. <!-- MAX_ACTOR_JOB_ASYNC_WAIT_SECS --> If the run finishes in time then the returned run object will have a terminal status (e.g. `SUCCEEDED`), otherwise it will have a transitional status (e.g. `RUNNING`).  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_post

> models::RunResponse act_runs_post(actor_id, body, timeout, memory, max_items, max_total_charge_usd, restart_on_error, build, wait_for_finish, webhooks, force_permission_level)
Run Actor

Runs an Actor and immediately returns without waiting for the run to finish.  The POST payload including its `Content-Type` header is passed as `INPUT` to the Actor (usually `application/json`).  The Actor is started with the default options; you can override them using various URL query parameters.  The response is the Run object as returned by the [Get run](#/reference/actor-runs/run-object-and-its-storages/get-run) API endpoint.  If you want to wait for the run to finish and receive the actual output of the Actor as the response, please use one of the [Run Actor synchronously](#/reference/actors/run-actor-synchronously) API endpoints instead.  To fetch the Actor run results that are typically stored in the default dataset, you'll need to pass the ID received in the `defaultDatasetId` field received in the response JSON to the [Get dataset items](#/reference/datasets/item-collection/get-items) API endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**body** | **serde_json::Value** |  | [required] |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**max_total_charge_usd** | Option<**f64**> | Specifies the maximum cost of the run. This parameter is useful for pay-per-event Actors, as it allows you to limit the amount charged to your subscription. You can access the maximum cost in your Actor by using the `ACTOR_MAX_TOTAL_CHARGE_USD` environment variable.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the run will be restarted if it fails.  |  |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run uses the build from its configuration (typically `latest`).  |  |
**wait_for_finish** | Option<**f64**> | The maximum number of seconds the server waits for the run to finish. By default it is `0`, the maximum value is `60`. <!-- MAX_ACTOR_JOB_ASYNC_WAIT_SECS --> If the run finishes in time then the returned run object will have a terminal status (e.g. `SUCCEEDED`), otherwise it will have a transitional status (e.g. `RUNNING`).  |  |
**webhooks** | Option<**String**> | Specifies optional webhooks associated with the Actor run, which can be used to receive a notification e.g. when the Actor finished or failed. The value is a Base64-encoded JSON array whose items follow the WebhookRepresentation schema. For more information, see [Webhooks documentation](https://docs.apify.com/platform/integrations/webhooks).  |  |
**force_permission_level** | Option<**String**> | Overrides the Actor's permission level for this specific run. Use to test restricted permissions before deploying changes to your Actor or to temporarily elevate or restrict access. If you don't specify this parameter, the Actor uses its configured default permission level. For more information on permissions, see the [documentation](https://docs.apify.com/platform/actors/development/permissions).  |  |

### Return type

[**models::RunResponse**](RunResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

