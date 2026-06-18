# \StorageDatasetsApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**dataset_delete**](StorageDatasetsApi.md#dataset_delete) | **DELETE** /v2/datasets/{datasetId} | Delete dataset
[**dataset_get**](StorageDatasetsApi.md#dataset_get) | **GET** /v2/datasets/{datasetId} | Get dataset
[**dataset_items_get**](StorageDatasetsApi.md#dataset_items_get) | **GET** /v2/datasets/{datasetId}/items | Get dataset items
[**dataset_items_head**](StorageDatasetsApi.md#dataset_items_head) | **HEAD** /v2/datasets/{datasetId}/items | Get dataset items headers
[**dataset_items_post**](StorageDatasetsApi.md#dataset_items_post) | **POST** /v2/datasets/{datasetId}/items | Store items
[**dataset_put**](StorageDatasetsApi.md#dataset_put) | **PUT** /v2/datasets/{datasetId} | Update dataset
[**dataset_statistics_get**](StorageDatasetsApi.md#dataset_statistics_get) | **GET** /v2/datasets/{datasetId}/statistics | Get dataset statistics
[**datasets_get**](StorageDatasetsApi.md#datasets_get) | **GET** /v2/datasets | Get list of datasets
[**datasets_post**](StorageDatasetsApi.md#datasets_post) | **POST** /v2/datasets | Create dataset



## dataset_delete

> dataset_delete(dataset_id)
Delete dataset

Deletes a specific dataset.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset ID or `username~dataset-name`. | [required] |

### Return type

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dataset_get

> models::DatasetResponse dataset_get(dataset_id)
Get dataset

Returns dataset object for given dataset ID.  This does not return dataset items, only information about the storage itself. To retrieve dataset items, use the [List dataset items](/api/v2/dataset-items-get) endpoint.  :::note  Keep in mind that attributes `itemCount` and `cleanItemCount` are not propagated right away after data are pushed into a dataset.  :::  There is a short period (up to 5 seconds) during which these counters may not match with exact counts in dataset items. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset ID or `username~dataset-name`. | [required] |

### Return type

[**models::DatasetResponse**](DatasetResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dataset_items_get

> Vec<serde_json::Value> dataset_items_get(dataset_id, format, clean, offset, limit, fields, output_fields, omit, unwind, flatten, desc, attachment, delimiter, bom, xml_root, xml_row, skip_header_row, skip_hidden, skip_empty, simplified, view, skip_failed_pages, feed_title, feed_description, signature)
Get dataset items

Returns data stored in the dataset in a desired format.  ### Response format  The format of the response depends on <code>format</code> query parameter.  The <code>format</code> parameter can have one of the following values: <code>json</code>, <code>jsonl</code>, <code>xml</code>, <code>html</code>, <code>csv</code>, <code>xlsx</code> and <code>rss</code>.  The following table describes how each format is treated.  <table>   <tr>     <th>Format</th>     <th>Items</th>   </tr>   <tr>     <td><code>json</code></td>     <td rowspan=\"3\">The response is a JSON, JSONL or XML array of raw item objects.</td>   </tr>   <tr>     <td><code>jsonl</code></td>   </tr>   <tr>     <td><code>xml</code></td>   </tr>   <tr>     <td><code>html</code></td>     <td rowspan=\"3\">The response is a HTML, CSV or XLSX table, where columns correspond to the     properties of the item and rows correspond to each dataset item.</td>   </tr>   <tr>     <td><code>csv</code></td>   </tr>   <tr>     <td><code>xlsx</code></td>   </tr>   <tr>     <td><code>rss</code></td>     <td colspan=\"2\">The response is a RSS file. Each item is displayed as child elements of one     <code>&lt;item&gt;</code>.</td>   </tr> </table>  Note that CSV, XLSX and HTML tables are limited to 2000 columns and the column names cannot be longer than 200 characters. JSON, XML and RSS formats do not have such restrictions.  ### Hidden fields  The top-level fields starting with the `#` character are considered hidden. These are useful to store debugging information and can be omitted from the output by providing the `skipHidden=1` or `clean=1` query parameters. For example, if you store the following object to the dataset:  ``` {     productName: \"iPhone Xs\",     description: \"Welcome to the big screens.\"     #debug: {         url: \"https://www.apple.com/lae/iphone-xs/\",         crawledAt: \"2019-01-21T16:06:03.683Z\"     } } ```  The `#debug` field will be considered as hidden and can be omitted from the results. This is useful to provide nice cleaned data to end users, while keeping debugging info available if needed. The Dataset object returned by the API contains the number of such clean items in the`dataset.cleanItemCount` property.  ### XML format extension  When exporting results to XML or RSS formats, the names of object properties become XML tags and the corresponding values become tag's children. For example, the following JavaScript object:  ``` {     name: \"Paul Newman\",     address: [         { type: \"home\", street: \"21st\", city: \"Chicago\" },         { type: \"office\", street: null, city: null }     ] } ```  will be transformed to the following XML snippet:  ``` <name>Paul Newman</name> <address>   <type>home</type>   <street>21st</street>   <city>Chicago</city> </address> <address>   <type>office</type>   <street/>   <city/> </address> ```  If the JavaScript object contains a property named `@` then its sub-properties are exported as attributes of the parent XML element. If the parent XML element does not have any child elements then its value is taken from a JavaScript object property named `#`.  For example, the following JavaScript object:  ``` {   \"address\": [{     \"@\": {       \"type\": \"home\"     },     \"street\": \"21st\",     \"city\": \"Chicago\"   },   {     \"@\": {       \"type\": \"office\"     },     \"#\": 'unknown'   }] } ```  will be transformed to the following XML snippet:  ``` <address type=\"home\">   <street>21st</street>   <city>Chicago</city> </address> <address type=\"office\">unknown</address> ```  This feature is also useful to customize your RSS feeds generated for various websites.  By default the whole result is wrapped in a `<items>` element and each page object is wrapped in a `<item>` element. You can change this using <code>xmlRoot</code> and <code>xmlRow</code> url parameters.  ### Pagination  The generated response supports [pagination](#/introduction/pagination). The pagination is always performed with the granularity of a single item, regardless whether <code>unwind</code> parameter was provided. By default, the **Items** in the response are sorted by the time they were stored to the database, therefore you can use pagination to incrementally fetch the items as they are being added. No limit exists to how many items can be returned in one response.  If you specify `desc=1` query parameter, the results are returned in the reverse order than they were stored (i.e. from newest to oldest items). Note that only the order of **Items** is reversed, but not the order of the `unwind` array elements. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset ID or `username~dataset-name`. | [required] |
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


## dataset_items_head

> dataset_items_head(dataset_id, format, clean, offset, limit, fields, output_fields, omit, unwind, flatten, desc, attachment, delimiter, bom, xml_root, xml_row, skip_header_row, skip_hidden, skip_empty, simplified, view, skip_failed_pages, feed_title, feed_description, signature)
Get dataset items headers

Returns only the HTTP headers for the dataset items endpoint, without the response body. This is useful to check pagination metadata or verify access without downloading the full dataset. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset ID or `username~dataset-name`. | [required] |
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

 (empty response body)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dataset_items_post

> serde_json::Value dataset_items_post(dataset_id, act_runs_last_dataset_items_post_request)
Store items

Appends an item or an array of items to the end of the dataset. The POST payload is a JSON object or a JSON array of objects to save into the dataset.  If the data you attempt to store in the dataset is invalid (meaning any of the items received by the API fails the validation), the whole request is discarded and the API will return a response with status code 400. For more information about dataset schema validation, see [Dataset schema](https://docs.apify.com/platform/actors/development/actor-definition/dataset-schema/validation).  **IMPORTANT:** The limit of request payload size for the dataset is 5 MB. If the array exceeds the size, you'll need to split it into a number of smaller arrays. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset ID or `username~dataset-name`. | [required] |
**act_runs_last_dataset_items_post_request** | Option<[**ActRunsLastDatasetItemsPostRequest**](ActRunsLastDatasetItemsPostRequest.md)> |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dataset_put

> models::DatasetResponse dataset_put(dataset_id, update_dataset_request)
Update dataset

Updates a dataset's name and general resource access level using a value specified by a JSON object passed in the PUT payload. The response is the updated dataset object, as returned by the [Get dataset](/api/v2/dataset-get) API endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset ID or `username~dataset-name`. | [required] |
**update_dataset_request** | [**UpdateDatasetRequest**](UpdateDatasetRequest.md) |  | [required] |

### Return type

[**models::DatasetResponse**](DatasetResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dataset_statistics_get

> models::DatasetStatisticsResponse dataset_statistics_get(dataset_id)
Get dataset statistics

Returns statistics for given dataset.  Provides only [field statistics](https://docs.apify.com/platform/actors/development/actor-definition/dataset-schema/validation#dataset-field-statistics). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dataset_id** | **String** | Dataset ID or `username~dataset-name`. | [required] |

### Return type

[**models::DatasetStatisticsResponse**](DatasetStatisticsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## datasets_get

> models::ListOfDatasetsResponse datasets_get(offset, limit, desc, unnamed, ownership)
Get list of datasets

Lists all of a user's datasets.  The response is a JSON array of objects, where each object contains basic information about one dataset.  By default, the objects are sorted by the `createdAt` field in ascending order, therefore you can use pagination to incrementally fetch all datasets while new ones are still being created. To sort them in descending order, use `desc=1` parameter. The endpoint supports pagination using `limit` and `offset` parameters and it will not return more than 1000 array elements. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |
**unnamed** | Option<**bool**> | If `true` or `1` then all the storages are returned. By default, only named storages are returned.  |  |
**ownership** | Option<[**StorageOwnership**](StorageOwnership.md)> | Filter by ownership. If this parameter is omitted, all accessible datasets are returned.  - `ownedByMe`: Return only datasets owned by the user. - `sharedWithMe`: Return only datasets shared with the user by other users.  |  |

### Return type

[**models::ListOfDatasetsResponse**](ListOfDatasetsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## datasets_post

> models::DatasetResponse datasets_post(name)
Create dataset

Creates a dataset and returns its object. Keep in mind that data stored under unnamed dataset follows [data retention period](https://docs.apify.com/platform/storage#data-retention). It creates a dataset with the given name if the parameter name is used. If a dataset with the given name already exists then returns its object. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | Option<**String**> | Custom unique name to easily identify the dataset in the future. |  |

### Return type

[**models::DatasetResponse**](DatasetResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

