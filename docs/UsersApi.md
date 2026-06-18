# \UsersApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**user_get**](UsersApi.md#user_get) | **GET** /v2/users/{userId} | Get public user data
[**users_me_get**](UsersApi.md#users_me_get) | **GET** /v2/users/me | Get private user data
[**users_me_limits_get**](UsersApi.md#users_me_limits_get) | **GET** /v2/users/me/limits | Get limits
[**users_me_limits_put**](UsersApi.md#users_me_limits_put) | **PUT** /v2/users/me/limits | Update limits
[**users_me_usage_monthly_get**](UsersApi.md#users_me_usage_monthly_get) | **GET** /v2/users/me/usage/monthly | Get monthly usage



## user_get

> models::PublicUserDataResponse user_get(user_id)
Get public user data

Returns public information about a specific user account, similar to what can be seen on public profile pages (e.g. https://apify.com/apify).  This operation requires no authentication token. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User ID or username. | [required] |

### Return type

[**models::PublicUserDataResponse**](PublicUserDataResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## users_me_get

> models::PrivateUserDataResponse users_me_get()
Get private user data

Returns information about the current user account, including both public and private information.  The user account is identified by the provided authentication token.  The fields `plan`, `email` and `profile` are omitted when this endpoint is accessed from Actor run. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::PrivateUserDataResponse**](PrivateUserDataResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## users_me_limits_get

> models::LimitsResponse users_me_limits_get()
Get limits

Returns a complete summary of your account's limits. It is the same information you will see on your account's [Limits page](https://console.apify.com/billing#/limits). The returned data includes the current usage cycle, a summary of your limits, and your current usage. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::LimitsResponse**](LimitsResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## users_me_limits_put

> serde_json::Value users_me_limits_put(update_limits_request)
Update limits

Updates the account's limits manageable on your account's [Limits page](https://console.apify.com/billing#/limits). Specifically the: `maxMonthlyUsageUsd` and `dataRetentionDays` limits (see request body schema for more details). 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**update_limits_request** | Option<[**UpdateLimitsRequest**](UpdateLimitsRequest.md)> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## users_me_usage_monthly_get

> models::MonthlyUsageResponse users_me_usage_monthly_get(date)
Get monthly usage

Returns a complete summary of your usage for the current monthly usage cycle, an overall sum, as well as a daily breakdown of usage. It is the same information you will see on your account's [Billing > Historical usage page](https://console.apify.com/billing/historical-usage). The information includes your use of Actors, compute, data transfer, and storage.  Using the `date` parameter will show your usage in the monthly usage cycle that includes that date. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**date** | Option<**String**> | Date in the YYYY-MM-DD format. |  |

### Return type

[**models::MonthlyUsageResponse**](MonthlyUsageResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

