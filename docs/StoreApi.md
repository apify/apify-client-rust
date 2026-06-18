# \StoreApi

All URIs are relative to *https://api.apify.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**store_get**](StoreApi.md#store_get) | **GET** /v2/store | Get list of Actors in Store



## store_get

> models::ListOfActorsInStoreResponse store_get(limit, offset, search, sort_by, category, username, pricing_model, allows_agentic_users, response_format, include_unrunnable_actors)
Get list of Actors in Store

Gets the list of public Actors in Apify Store. You can use `search` parameter to search Actors by string in title, name, description, username and readme. If you need detailed info about a specific Actor, use the [Get Actor](#/reference/actors/actor-object/get-actor) endpoint.  The endpoint supports pagination using the `limit` and `offset` parameters. It will not return more than 1,000 records. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**f64**> | Maximum number of items to return. The default value as well as the maximum is `1000`.  |  |
**offset** | Option<**f64**> | Number of items that should be skipped at the start. The default value is `0`.  |  |
**search** | Option<**String**> | String to search by. The search runs on the following fields: `title`, `name`, `description`, `username`, `readme`.  |  |
**sort_by** | Option<**String**> | Specifies the field by which to sort the results. The supported values are `relevance` (default), `popularity`, `newest` and `lastUpdate`.  |  |
**category** | Option<**String**> | Filters the results by the specified category. |  |
**username** | Option<**String**> | Filters the results by the specified username. |  |
**pricing_model** | Option<**String**> | Only return Actors with the specified pricing model.  |  |
**allows_agentic_users** | Option<**bool**> | If true, only return Actors that allow agentic users. If false, only return Actors that do not allow agentic users.  |  |
**response_format** | Option<**String**> | Controls the shape of the response. Use `full` (default) for the complete response including image URLs and all fields. Use `agent` for a reduced field set optimized for LLM consumers, which only includes `id`, `title`, `name`, `username`, `description`, `notice`, `badge`, `categories`, and minimal `stats`.  |  |[default to full]
**include_unrunnable_actors** | Option<**bool**> | By default, search results exclude Actors that are not safe to run automatically (e.g. Actors from developers who haven't passed KYC, or full-permission Actors without a large user base). Set to `true` to bypass this safety filtering and include all Actors in the results.  |  |[default to false]

### Return type

[**models::ListOfActorsInStoreResponse**](ListOfActorsInStoreResponse.md)

### Authorization

[apiKey](../README.md#apiKey), [httpBearer](../README.md#httpBearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

