# \SchedulesApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**schedule_delete**](SchedulesApi.md#schedule_delete) | **DELETE** /v2/schedules/{scheduleId} | Delete schedule
[**schedule_get**](SchedulesApi.md#schedule_get) | **GET** /v2/schedules/{scheduleId} | Get schedule
[**schedule_log_get**](SchedulesApi.md#schedule_log_get) | **GET** /v2/schedules/{scheduleId}/log | Get schedule log
[**schedule_put**](SchedulesApi.md#schedule_put) | **PUT** /v2/schedules/{scheduleId} | Update schedule
[**schedules_get**](SchedulesApi.md#schedules_get) | **GET** /v2/schedules | Get list of schedules
[**schedules_post**](SchedulesApi.md#schedules_post) | **POST** /v2/schedules | Create schedule



## schedule_delete

> serde_json::Value schedule_delete(schedule_id)
Delete schedule

Deletes a schedule.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**schedule_id** | **String** | Schedule ID. | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## schedule_get

> models::ScheduleResponse schedule_get(schedule_id)
Get schedule

Gets the schedule object with all details.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**schedule_id** | **String** | Schedule ID. | [required] |

### Return type

[**models::ScheduleResponse**](ScheduleResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## schedule_log_get

> models::ScheduleLogResponse schedule_log_get(schedule_id)
Get schedule log

Gets the schedule log as a JSON array containing information about up to a 1000 invocations of the schedule. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**schedule_id** | **String** | Schedule ID. | [required] |

### Return type

[**models::ScheduleLogResponse**](ScheduleLogResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## schedule_put

> models::ScheduleResponse schedule_put(schedule_id, schedule_create)
Update schedule

Updates a schedule using values specified by a schedule object passed as JSON in the POST payload. If the object does not define a specific property, its value will not be updated.  The response is the full schedule object as returned by the [Get schedule](#/reference/schedules/schedule-object/get-schedule) endpoint.  **The request needs to specify the `Content-Type: application/json` HTTP header!**  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. ([More info](#/introduction/authentication)). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**schedule_id** | **String** | Schedule ID. | [required] |
**schedule_create** | [**ScheduleCreate**](ScheduleCreate.md) |  | [required] |

### Return type

[**models::ScheduleResponse**](ScheduleResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## schedules_get

> models::ListOfSchedulesResponse schedules_get(offset, limit, desc)
Get list of schedules

Gets the list of schedules that the user created.  The endpoint supports pagination using the `limit` and `offset` parameters. It will not return more than 1000 records.  By default, the records are sorted by the `createdAt` field in ascending order. To sort the records in descending order, use the `desc=1` parameter. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**desc** | Option<**bool**> | If `true` or `1` then the objects are sorted by the `createdAt` field in descending order. By default, they are sorted in ascending order.  |  |

### Return type

[**models::ListOfSchedulesResponse**](ListOfSchedulesResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## schedules_post

> models::ScheduleResponse schedules_post(schedule_create)
Create schedule

Creates a new schedule with settings provided by the schedule object passed as JSON in the payload. The response is the created schedule object.  The request needs to specify the `Content-Type: application/json` HTTP header!  When providing your API authentication token, we recommend using the request's `Authorization` header, rather than the URL. ([More info](#/introduction/authentication)). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**schedule_create** | [**ScheduleCreate**](ScheduleCreate.md) |  | [required] |

### Return type

[**models::ScheduleResponse**](ScheduleResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

