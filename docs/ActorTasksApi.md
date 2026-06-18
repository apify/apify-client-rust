# \ActorTasksApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**actor_task_delete**](ActorTasksApi.md#actor_task_delete) | **DELETE** /v2/actor-tasks/{actorTaskId} | Delete task
[**actor_task_get**](ActorTasksApi.md#actor_task_get) | **GET** /v2/actor-tasks/{actorTaskId} | Get task
[**actor_task_input_get**](ActorTasksApi.md#actor_task_input_get) | **GET** /v2/actor-tasks/{actorTaskId}/input | Get task input
[**actor_task_input_put**](ActorTasksApi.md#actor_task_input_put) | **PUT** /v2/actor-tasks/{actorTaskId}/input | Update task input
[**actor_task_put**](ActorTasksApi.md#actor_task_put) | **PUT** /v2/actor-tasks/{actorTaskId} | Update task
[**actor_task_run_sync_get**](ActorTasksApi.md#actor_task_run_sync_get) | **GET** /v2/actor-tasks/{actorTaskId}/run-sync | Run task synchronously
[**actor_task_run_sync_get_dataset_items_get**](ActorTasksApi.md#actor_task_run_sync_get_dataset_items_get) | **GET** /v2/actor-tasks/{actorTaskId}/run-sync-get-dataset-items | Run task synchronously and get dataset items
[**actor_task_run_sync_get_dataset_items_post**](ActorTasksApi.md#actor_task_run_sync_get_dataset_items_post) | **POST** /v2/actor-tasks/{actorTaskId}/run-sync-get-dataset-items | Run task synchronously and get dataset items
[**actor_task_run_sync_post**](ActorTasksApi.md#actor_task_run_sync_post) | **POST** /v2/actor-tasks/{actorTaskId}/run-sync | Run task synchronously
[**actor_task_runs_get**](ActorTasksApi.md#actor_task_runs_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs | Get list of task runs
[**actor_task_runs_last_get**](ActorTasksApi.md#actor_task_runs_last_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last | Get last run
[**actor_task_runs_post**](ActorTasksApi.md#actor_task_runs_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs | Run task
[**actor_task_webhooks_get**](ActorTasksApi.md#actor_task_webhooks_get) | **GET** /v2/actor-tasks/{actorTaskId}/webhooks | Get list of webhooks
[**actor_tasks_get**](ActorTasksApi.md#actor_tasks_get) | **GET** /v2/actor-tasks | Get list of tasks
[**actor_tasks_post**](ActorTasksApi.md#actor_tasks_post) | **POST** /v2/actor-tasks | Create task



## actor_task_delete

> serde_json::Value actor_task_delete(actor_task_id)
Delete task

Delete the task specified through the `actorTaskId` parameter.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_get

> models::ActorTaskGet200Response actor_task_get(actor_task_id)
Get task

Get an object that contains all the details about a task.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |

### Return type

[**models::ActorTaskGet200Response**](actorTask_get_200_response.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_input_get

> serde_json::Value actor_task_input_get(actor_task_id)
Get task input

Returns the input of a given task.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_input_put

> serde_json::Value actor_task_input_put(actor_task_id, body)
Update task input

Updates the input of a task using values specified by an object passed as JSON in the PUT payload.  If the object does not define a specific property, its value is not updated.  The response is the full task input as returned by the [Get task input](#/reference/tasks/task-input-object/get-task-input) endpoint.  The request needs to specify the `Content-Type: application/json` HTTP header!  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. ([More info](#/introduction/authentication)). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**body** | **serde_json::Value** |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_put

> models::ActorTaskGet200Response actor_task_put(actor_task_id, update_task_request)
Update task

Update settings of a task using values specified by an object passed as JSON in the POST payload.  If the object does not define a specific property, its value is not updated.  The response is the full task object as returned by the [Get task](/api/v2/actor-task-get) endpoint.  The request needs to specify the `Content-Type: application/json` HTTP header!  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**update_task_request** | [**UpdateTaskRequest**](UpdateTaskRequest.md) |  | [required] |

### Return type

[**models::ActorTaskGet200Response**](actorTask_get_200_response.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_run_sync_get

> serde_json::Value actor_task_run_sync_get(actor_task_id, timeout, memory, max_items, build, output_record_key, webhooks)
Run task synchronously

Run a specific task and return its output.  The run must finish in 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds otherwise the HTTP request fails with a timeout error (this won't abort the run itself).  Beware that it might be impossible to maintain an idle HTTP connection for an extended period, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout.  If the connection breaks, you will not receive any information about the run and its status.  To run the Task asynchronously, use the [Run task asynchronously](#/reference/actor-tasks/run-collection/run-task) endpoint instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run uses the build from its configuration (typically `latest`).  |  |
**output_record_key** | Option<**String**> | Key of the record from run's default key-value store to be returned in the response. By default, it is `OUTPUT`.  |  |
**webhooks** | Option<**String**> | Specifies optional webhooks associated with the Actor run, which can be used to receive a notification e.g. when the Actor finished or failed. The value is a Base64-encoded JSON array whose items follow the WebhookRepresentation schema. For more information, see [Webhooks documentation](https://docs.apify.com/platform/integrations/webhooks).  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_run_sync_get_dataset_items_get

> serde_json::Value actor_task_run_sync_get_dataset_items_get(actor_task_id, timeout, memory, max_items, build, webhooks, format, clean, offset, limit, fields, output_fields, omit, unwind, flatten, desc, attachment, delimiter, bom, xml_root, xml_row, skip_header_row, skip_hidden, skip_empty, simplified, view, skip_failed_pages, feed_title, feed_description)
Run task synchronously and get dataset items

Run a specific task and return its dataset items.  The run must finish in 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds otherwise the HTTP request fails with a timeout error (this won't abort the run itself).  You can send all the same options in parameters as the [Get Dataset Items](#/reference/datasets/item-collection/get-items) API endpoint.  Beware that it might be impossible to maintain an idle HTTP connection for an extended period, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout.  If the connection breaks, you will not receive any information about the run and its status.  To run the Task asynchronously, use the [Run task asynchronously](#/reference/actor-tasks/run-collection/run-task) endpoint instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
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

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_run_sync_get_dataset_items_post

> serde_json::Value actor_task_run_sync_get_dataset_items_post(actor_task_id, body, timeout, memory, max_items, max_total_charge_usd, restart_on_error, build, webhooks, format, clean, offset, limit, fields, output_fields, omit, unwind, flatten, desc, attachment, delimiter, bom, xml_root, xml_row, skip_header_row, skip_hidden, skip_empty, simplified, view, skip_failed_pages, feed_title, feed_description)
Run task synchronously and get dataset items

Runs an Actor task and synchronously returns its dataset items.  The run must finish in 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds otherwise the HTTP request fails with a timeout error (this won't abort the run itself).  Optionally, you can override the Actor input configuration by passing a JSON object as the POST payload and setting the `Content-Type: application/json` HTTP header.  Note that if the object in the POST payload does not define a particular input property, the Actor run uses the default value defined by the task (or the Actor's input schema if not defined by the task).  You can send all the same options in parameters as the [Get Dataset Items](#/reference/datasets/item-collection/get-items) API endpoint.  Beware that it might be impossible to maintain an idle HTTP connection for an extended period, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout.  If the connection breaks, you will not receive any information about the run and its status.  Input fields from Actor task configuration can be overloaded with values passed as the POST payload.  Just make sure to specify the `Content-Type` header as `application/json` and that the input is an object.  To run the task asynchronously, use the [Run task](#/reference/actor-tasks/run-collection/run-task) API endpoint instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
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

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_run_sync_post

> serde_json::Value actor_task_run_sync_post(actor_task_id, body, timeout, memory, max_items, max_total_charge_usd, restart_on_error, build, output_record_key, webhooks)
Run task synchronously

Runs an Actor task and synchronously returns its output.  The run must finish in 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds otherwise the HTTP request fails with a timeout error (this won't abort the run itself).  Optionally, you can override the Actor input configuration by passing a JSON object as the POST payload and setting the `Content-Type: application/json` HTTP header.  Note that if the object in the POST payload does not define a particular input property, the Actor run uses the default value defined by the task (or Actor's input schema if not defined by the task).  Beware that it might be impossible to maintain an idle HTTP connection for an extended period, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout.  If the connection breaks, you will not receive any information about the run and its status.  Input fields from Actor task configuration can be overloaded with values passed as the POST payload.  Just make sure to specify `Content-Type` header to be `application/json` and input to be an object.  To run the task asynchronously, use the [Run task](#/reference/actor-tasks/run-collection/run-task) API endpoint instead. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**body** | **serde_json::Value** |  | [required] |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**max_total_charge_usd** | Option<**f64**> | Specifies the maximum cost of the run. This parameter is useful for pay-per-event Actors, as it allows you to limit the amount charged to your subscription. You can access the maximum cost in your Actor by using the `ACTOR_MAX_TOTAL_CHARGE_USD` environment variable.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the run will be restarted if it fails.  |  |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run uses the build from its configuration (typically `latest`).  |  |
**output_record_key** | Option<**String**> | Key of the record from run's default key-value store to be returned in the response. By default, it is `OUTPUT`.  |  |
**webhooks** | Option<**String**> | Specifies optional webhooks associated with the Actor run, which can be used to receive a notification e.g. when the Actor finished or failed. The value is a Base64-encoded JSON array whose items follow the WebhookRepresentation schema. For more information, see [Webhooks documentation](https://docs.apify.com/platform/integrations/webhooks).  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_get

> models::ActorTaskRunsGet200Response actor_task_runs_get(actor_task_id, offset, limit, desc, status)
Get list of task runs

Get a list of runs of a specific task. The response is a list of objects, where each object contains essential information about a single task run.  The endpoint supports pagination using the `limit` and `offset` parameters, and it does not return more than a 1000 array elements.  By default, the records are sorted by the `startedAt` field in ascending order; therefore you can use pagination to incrementally fetch all records while new ones are still being created. To sort the records in descending order, use the `desc=1` parameter. You can also filter runs by status ([available statuses](https://docs.apify.com/platform/actors/running/runs-and-builds#lifecycle)). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `startedAt` field in descending order. By default, they are sorted in ascending order.  |  |
**status** | Option<[**Vec<String>**](String.md)> | Single status or comma-separated list of statuses, see ([available statuses](https://docs.apify.com/platform/actors/running/runs-and-builds#lifecycle)). Used to filter runs by the specified statuses only.  |  |

### Return type

[**models::ActorTaskRunsGet200Response**](actorTask_runs_get_200_response.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_last_get

> models::ActorTaskRunsPost201Response actor_task_runs_last_get(actor_task_id, status, wait_for_finish)
Get last run

This is not a single endpoint, but an entire group of endpoints that lets you to retrieve and manage the last run of given actor task or any of its default storages. All the endpoints require an authentication token.  The base path represents the last actor task run object is:  `/v2/actor-tasks/{actorTaskId}/runs/last{?token,status}`  Using the `status` query parameter you can ensure to only get a run with a certain status (e.g. `status=SUCCEEDED`). The output of this endpoint and other query parameters are the same as in the [Run object](/api/v2/actor-run-get) endpoint.  ##### Convenience endpoints for last Actor task run  * [Dataset](/api/v2/last-actor-task-runs-default-dataset)  * [Key-value store](/api/v2/last-actor-task-runs-default-key-value-store)  * [Request queue](/api/v2/last-actor-task-runs-default-request-queue)  * [Log](/api/v2/last-actor-task-runs-log) 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
**wait_for_finish** | Option<**f64**> | The maximum number of seconds the server waits for the run to finish. By default it is `0`, the maximum value is `60`. <!-- MAX_ACTOR_JOB_ASYNC_WAIT_SECS --> If the run finishes in time then the returned run object will have a terminal status (e.g. `SUCCEEDED`), otherwise it will have a transitional status (e.g. `RUNNING`).  |  |

### Return type

[**models::ActorTaskRunsPost201Response**](actorTask_runs_post_201_response.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_runs_post

> models::ActorTaskRunsPost201Response actor_task_runs_post(actor_task_id, body, timeout, memory, max_items, max_total_charge_usd, restart_on_error, build, wait_for_finish, webhooks)
Run task

Runs an Actor task and immediately returns without waiting for the run to finish.  Optionally, you can override the Actor input configuration by passing a JSON object as the POST payload and setting the `Content-Type: application/json` HTTP header.  Note that if the object in the POST payload does not define a particular input property, the Actor run uses the default value defined by the task (or Actor's input schema if not defined by the task).  The response is the Actor Run object as returned by the [Get run](#/reference/actor-runs/run-object-and-its-storages/get-run) endpoint.  If you want to wait for the run to finish and receive the actual output of the Actor run as the response, use one of the [Run task synchronously](#/reference/actor-tasks/run-task-synchronously) API endpoints instead.  To fetch the Actor run results that are typically stored in the default dataset, you'll need to pass the ID received in the `defaultDatasetId` field received in the response JSON to the [Get dataset items](#/reference/datasets/item-collection/get-items) API endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**body** | **serde_json::Value** |  | [required] |
**timeout** | Option<**f64**> | Optional timeout for the run, in seconds. By default, the run uses the timeout from its configuration.  |  |
**memory** | Option<**f64**> | Memory limit for the run, in megabytes. The amount of memory can be set to a power of 2 with a minimum of 128. By default, the run uses the memory limit from its configuration.  |  |
**max_items** | Option<**f64**> | Specifies the maximum number of dataset items that will be charged for pay-per-result Actors. This does NOT guarantee that the Actor will return only this many items. It only ensures you won't be charged for more than this number of items. Only works for pay-per-result Actors. Value can be accessed in the actor run using `ACTOR_MAX_PAID_DATASET_ITEMS` environment variable.  |  |
**max_total_charge_usd** | Option<**f64**> | Specifies the maximum cost of the run. This parameter is useful for pay-per-event Actors, as it allows you to limit the amount charged to your subscription. You can access the maximum cost in your Actor by using the `ACTOR_MAX_TOTAL_CHARGE_USD` environment variable.  |  |
**restart_on_error** | Option<**bool**> | Determines whether the run will be restarted if it fails.  |  |
**build** | Option<**String**> | Specifies the Actor build to run. It can be either a build tag or build number. By default, the run uses the build from its configuration (typically `latest`).  |  |
**wait_for_finish** | Option<**f64**> | The maximum number of seconds the server waits for the run to finish. By default it is `0`, the maximum value is `60`. <!-- MAX_ACTOR_JOB_ASYNC_WAIT_SECS --> If the run finishes in time then the returned run object will have a terminal status (e.g. `SUCCEEDED`), otherwise it will have a transitional status (e.g. `RUNNING`).  |  |
**webhooks** | Option<**String**> | Specifies optional webhooks associated with the Actor run, which can be used to receive a notification e.g. when the Actor finished or failed. The value is a Base64-encoded JSON array whose items follow the WebhookRepresentation schema. For more information, see [Webhooks documentation](https://docs.apify.com/platform/integrations/webhooks).  |  |

### Return type

[**models::ActorTaskRunsPost201Response**](actorTask_runs_post_201_response.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_task_webhooks_get

> models::ActorTaskWebhooksGet200Response actor_task_webhooks_get(actor_task_id, offset, limit, desc)
Get list of webhooks

Gets the list of webhooks of a specific Actor task. The response is a JSON with the list of objects, where each object contains basic information about a single webhook.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 records.  By default, the records are sorted by the `createdAt` field in ascending order, to sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_task_id** | **String** | Task ID or a tilde-separated owner's username and task's name. | [required] |
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |

### Return type

[**models::ActorTaskWebhooksGet200Response**](actorTask_webhooks_get_200_response.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_tasks_get

> models::ListOfTasksResponse actor_tasks_get(offset, limit, desc)
Get list of tasks

Gets the complete list of tasks that a user has created or used.  The response is a list of objects in which each object contains essential information about a single task.  The endpoint supports pagination using the `limit` and `offset` parameters, and it does not return more than a 1000 records.  By default, the records are sorted by the `createdAt` field in ascending order; therefore you can use pagination to incrementally fetch all tasks while new ones are still being created. To sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |

### Return type

[**models::ListOfTasksResponse**](ListOfTasksResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## actor_tasks_post

> models::TaskResponse actor_tasks_post(create_task_request)
Create task

Create a new task with settings specified by the object passed as JSON in the POST payload.  The response is the full task object as returned by the [Get task](/api/v2/actor-task-get) endpoint.  The request needs to specify the `Content-Type: application/json` HTTP header!  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_task_request** | [**CreateTaskRequest**](CreateTaskRequest.md) |  | [required] |

### Return type

[**models::TaskResponse**](TaskResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

