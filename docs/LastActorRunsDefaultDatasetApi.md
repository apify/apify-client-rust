# \LastActorRunsDefaultDatasetApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**act_runs_last_dataset_delete**](LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_delete) | **DELETE** /v2/actors/{actorId}/runs/last/dataset | Delete last run's default dataset
[**act_runs_last_dataset_get**](LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_get) | **GET** /v2/actors/{actorId}/runs/last/dataset | Get last run's default dataset
[**act_runs_last_dataset_items_get**](LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_items_get) | **GET** /v2/actors/{actorId}/runs/last/dataset/items | Get last run's dataset items
[**act_runs_last_dataset_items_post**](LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_items_post) | **POST** /v2/actors/{actorId}/runs/last/dataset/items | Store items in last run's dataset
[**act_runs_last_dataset_put**](LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_put) | **PUT** /v2/actors/{actorId}/runs/last/dataset | Update last run's default dataset
[**act_runs_last_dataset_statistics_get**](LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_statistics_get) | **GET** /v2/actors/{actorId}/runs/last/dataset/statistics | Get last run's dataset statistics



## act_runs_last_dataset_delete

> act_runs_last_dataset_delete(actor_id, status)
Delete last run's default dataset

Deletes the default dataset associated with the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultDatasetId` and then using the [Delete dataset](/api/v2/dataset-delete) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_dataset_get

> models::DatasetResponse act_runs_last_dataset_get(actor_id, status)
Get last run's default dataset

Returns the default dataset associated with the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultDatasetId` and then using the [Get dataset](/api/v2/dataset-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::DatasetResponse**](DatasetResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_dataset_items_get

> Vec<serde_json::Value> act_runs_last_dataset_items_get(actor_id, status, format, clean, offset, limit, fields, output_fields, omit, unwind, flatten, desc, attachment, delimiter, bom, xml_root, xml_row, skip_header_row, skip_hidden, skip_empty, simplified, view, skip_failed_pages, feed_title, feed_description, signature)
Get last run's dataset items

Returns data stored in the default dataset of the last Actor run in the desired format.  This endpoint is a shortcut that resolves the last run's `defaultDatasetId` and proxies to the [Get dataset items](/api/v2/dataset-items-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |
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
**signature** | Option<**String**> | Signature used for the access. |  |

### Return type

[**Vec<serde_json::Value>**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json, application/jsonl, text/csv, text/html, application/vnd.openxmlformats-officedocument.spreadsheetml.sheet, application/rss+xml, application/xml

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_dataset_items_post

> serde_json::Value act_runs_last_dataset_items_post(actor_id, act_runs_last_dataset_items_post_request, status)
Store items in last run's dataset

Appends an item or an array of items to the end of the last Actor run's default dataset.  This endpoint is a shortcut that resolves the last run's `defaultDatasetId` and proxies to the [Store items](/api/v2/dataset-items-post) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**act_runs_last_dataset_items_post_request** | Option<[**ActRunsLastDatasetItemsPostRequest**](ActRunsLastDatasetItemsPostRequest.md)> |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_dataset_put

> models::DatasetResponse act_runs_last_dataset_put(actor_id, update_dataset_request, status)
Update last run's default dataset

Updates the default dataset associated with the last Actor run.  This endpoint is a shortcut for getting the last run's `defaultDatasetId` and then using the [Update dataset](/api/v2/dataset-put) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**update_dataset_request** | [**UpdateDatasetRequest**](UpdateDatasetRequest.md) |  | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::DatasetResponse**](DatasetResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## act_runs_last_dataset_statistics_get

> models::DatasetStatisticsResponse act_runs_last_dataset_statistics_get(actor_id, status)
Get last run's dataset statistics

Returns statistics for the last Actor run's default dataset.  This endpoint is a shortcut that resolves the last run's `defaultDatasetId` and proxies to the [Get dataset statistics](/api/v2/dataset-statistics-get) endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**actor_id** | **String** | Actor ID or a tilde-separated owner's username and Actor name. | [required] |
**status** | Option<**String**> | Filter for the run status. |  |

### Return type

[**models::DatasetStatisticsResponse**](DatasetStatisticsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

