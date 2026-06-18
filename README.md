# Rust API client for apify-client


The Apify API (version 2) provides programmatic access to the [Apify
platform](https://docs.apify.com). The API is organized
around [RESTful](https://en.wikipedia.org/wiki/Representational_state_transfer)
HTTP endpoints.

You can download the complete OpenAPI schema of Apify API in the [YAML](http://docs.apify.com/api/openapi.yaml) or [JSON](http://docs.apify.com/api/openapi.json) formats. The source code is also available on [GitHub](https://github.com/apify/apify-docs/tree/master/apify-api/openapi).

All requests and responses (including errors) are encoded in
[JSON](http://www.json.org/) format with UTF-8 encoding,
with a few exceptions that are explicitly described in the reference.

- To access the API using [Node.js](https://nodejs.org/en/), we recommend the [`apify-client`](https://docs.apify.com/api/client/js) [NPM
package](https://www.npmjs.com/package/apify-client).
- To access the API using [Python](https://www.python.org/), we recommend the [`apify-client`](https://docs.apify.com/api/client/python) [PyPI
package](https://pypi.org/project/apify-client/).

The clients' functions correspond to the API endpoints and have the same
parameters. This simplifies development of apps that depend on the Apify
platform.

:::note Important Request Details

- `Content-Type` header: For requests with a JSON body, you must include the `Content-Type: application/json` header.

- Method override: You can override the HTTP method using the `method` query parameter. This is useful for clients that can only send `GET` requests. For example, to call a `POST` endpoint, append `?method=POST` to the URL of your `GET` request.

:::

## Authentication
<span id=\"/introduction/authentication\"></span>

**You can find your API token on the
[Integrations](https://console.apify.com/settings/integrations) page in the
Apify Console.**

To use your token in a request, either:

- Add the token to your request's `Authorization` header as `Bearer <token>`.
E.g., `Authorization: Bearer xxxxxxx`.
[More info](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization).
(Recommended).
- Add it as the `token` parameter to your request URL. (Less secure).

Using your token in the request header is more secure than using it as a URL
parameter because URLs are often stored
in browser history and server logs. This creates a chance for someone
unauthorized to access your API token.

**Never share your API token or password with untrusted parties!**

For more information, see our
[integrations](https://docs.apify.com/platform/integrations) documentation.

### Agentic payments

AI agents can authenticate and pay for Actor runs without an Apify account
using agentic payments. Instead of an API token, the request carries a
payment credential that both authorizes and pays for the call. Apify supports
the [x402 protocol](https://docs.apify.com/platform/integrations/x402)
(`PAYMENT-SIGNATURE` header) and
[Skyfire](https://docs.apify.com/platform/integrations/skyfire)
(`skyfire-pay-id` header).

## Basic usage
<span id=\"/introduction/basic-usage\"></span>

To run an Actor, send a POST request to the [Run
Actor](#/reference/actors/run-collection/run-actor) endpoint using either the
Actor ID code (e.g. `vKg4IjxZbEYTYeW8T`) or its name (e.g.
`janedoe~my-actor`):

`https://api.apify.com/v2/actors/[actor_id]/runs`

If the Actor is not runnable anonymously, you will receive a 401 or 403
[response code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status).
This means you need to add your [secret API
token](https://console.apify.com/account#/integrations) to the request's
`Authorization` header ([recommended](#/introduction/authentication)) or as a
URL query parameter `?token=[your_token]` (less secure).

Optionally, you can include the query parameters described in the [Run
Actor](#/reference/actors/run-collection/run-actor) section to customize your
run.

If you're using Node.js, the best way to run an Actor is using the
`Apify.call()` method from the [Apify
SDK](https://sdk.apify.com/docs/api/apify#apifycallactid-input-options). It
runs the Actor using the account you are currently logged into (determined
by the [secret API token](https://console.apify.com/account#/integrations)).
The result is an [Actor run
object](https://sdk.apify.com/docs/typedefs/actor-run) and its output (if
any).

A typical workflow is as follows:

1. Run an Actor or task using the [Run
Actor](#/reference/actors/run-collection/run-actor) or [Run
task](#/reference/actor-tasks/run-collection/run-task) API endpoints.
2. Monitor the Actor run by periodically polling its progress using the [Get
run](#/reference/actor-runs/run-object-and-its-storages/get-run) API
endpoint.
3. Fetch the results from the [Get
items](#/reference/datasets/item-collection/get-items) API endpoint using the
`defaultDatasetId`, which you receive in the Run request response.
Additional data may be stored in a key-value store. You can fetch them from
the [Get record](#/reference/key-value-stores/record/get-record) API endpoint
using the `defaultKeyValueStoreId` and the store's `key`.

**Note**: Instead of periodic polling, you can also run your
[Actor](#/reference/actors/run-actor-synchronously) or
[task](#/reference/actor-tasks/runs-collection/run-task-synchronously)
synchronously. This will ensure that the request waits for 300 seconds (5
minutes) for the run to finish and returns its output. If the run takes
longer, the request will time out and throw an error.

## Legacy `/v2/acts/` URL prefix
<span id=\"/introduction/legacy-acts-prefix\"></span>

The `/v2/acts/` prefix is deprecated but still fully functional, and 
such endpoint routes to the same handler as its `/v2/actors/...` counterpart. 
New integrations should use the canonical /v2/actors/ prefix, 
but existing clients keep working without changes.

## Response structure
<span id=\"/introduction/response-structure\"></span>

Most API endpoints return a JSON object with the `data` property:

```
{
    \"data\": {
        ...
    }
}
```

However, there are a few explicitly described exceptions, such as
[Get dataset items](#/reference/datasets/item-collection/get-items) or
Key-value store [Get record](#/reference/key-value-stores/record/get-record)
API endpoints, which return data in other formats.
In case of an error, the response has the HTTP status code in the range of
4xx or 5xx and the `data` property is replaced with `error`. For example:

```
{
    \"error\": {
        \"type\": \"record-not-found\",
        \"message\": \"Store was not found.\"
    }
}
```

See [Errors](#/introduction/errors) for more details.

## Pagination
<span id=\"/introduction/pagination\"></span>

All API endpoints that return a list of records
(e.g. [Get list of
Actors](#/reference/actors/actor-collection/get-list-of-actors))
enforce pagination in order to limit the size of their responses.

Most of these API endpoints are paginated using the `offset` and `limit`
query parameters.
The only exception is [Get list of
keys](#/reference/key-value-stores/key-collection/get-list-of-keys),
which is paginated using the `exclusiveStartKey` query parameter.

**IMPORTANT**: Each API endpoint that supports pagination enforces a certain
maximum value for the `limit` parameter,
in order to reduce the load on Apify servers.
The maximum limit could change in future so you should never
rely on a specific value and check the responses of these API endpoints.

### Using offset
<span id=\"/introduction/pagination/using-offset\"></span>

Most API endpoints that return a list of records enable pagination using the
following query parameters:

<table>
  <tr>
    <td><code>limit</code></td>
    <td>Limits the response to contain a specific maximum number of items, e.g. <code>limit=20</code>.</td>
  </tr>
  <tr>
    <td><code>offset</code></td>
    <td>Skips a number of items from the beginning of the list, e.g. <code>offset=100</code>.</td>
  </tr>
  <tr>
    <td><code>desc</code></td>
    <td>
    By default, items are sorted in the order in which they were created or added to the list.
    This feature is useful when fetching all the items, because it ensures that items
    created after the client started the pagination will not be skipped.
    If you specify the <code>desc=1</code> parameter, the items will be returned in the reverse order,
    i.e. from the newest to the oldest items.
    </td>
  </tr>
</table>

The response of these API endpoints is always a JSON object with the
following structure:

```
{
    \"data\": {
        \"total\": 2560,
        \"offset\": 250,
        \"limit\": 1000,
        \"count\": 1000,
        \"desc\": false,
        \"items\": [
            { 1st object },
            { 2nd object },
            ...
            { 1000th object }
        ]
    }
}
```

The following table describes the meaning of the response properties:

<table>
  <tr>
    <th>Property</th>
    <th>Description</th>
  </tr>
  <tr>
    <td><code>total</code></td>
    <td>The total number of items available in the list.</td>
  </tr>
  <tr>
    <td><code>offset</code></td>
    <td>The number of items that were skipped at the start.
    This is equal to the <code>offset</code> query parameter if it was provided, otherwise it is <code>0</code>.</td>
  </tr>
  <tr>
    <td><code>limit</code></td>
    <td>The maximum number of items that can be returned in the HTTP response.
    It equals to the <code>limit</code> query parameter if it was provided or
    the maximum limit enforced for the particular API endpoint, whichever is smaller.</td>
  </tr>
  <tr>
    <td><code>count</code></td>
    <td>The actual number of items returned in the HTTP response.</td>
  </tr>
  <tr>
    <td><code>desc</code></td>
    <td><code>true</code> if data were requested in descending order and <code>false</code> otherwise.</td>
  </tr>
  <tr>
    <td><code>items</code></td>
    <td>An array of requested items.</td>
  </tr>
</table>

### Using key
<span id=\"/introduction/pagination/using-key\"></span>

The records in the [key-value
store](https://docs.apify.com/platform/storage/key-value-store)
are not ordered based on numerical indexes,
but rather by their keys in the UTF-8 binary order.
Therefore the [Get list of
keys](#/reference/key-value-stores/key-collection/get-list-of-keys)
API endpoint only supports pagination using the following query parameters:

<table>
  <tr>
    <td><code>limit</code></td>
    <td>Limits the response to contain a specific maximum number items, e.g. <code>limit=20</code>.</td>
  </tr>
  <tr>
    <td><code>exclusiveStartKey</code></td>
    <td>Skips all records with keys up to the given key including the given key,
    in the UTF-8 binary order.</td>
  </tr>
</table>

The response of the API endpoint is always a JSON object with following
structure:

```
{
    \"data\": {
        \"limit\": 1000,
        \"isTruncated\": true,
        \"exclusiveStartKey\": \"my-key\",
        \"nextExclusiveStartKey\": \"some-other-key\",
        \"items\": [
            { 1st object },
            { 2nd object },
            ...
            { 1000th object }
        ]
    }
}
```

The following table describes the meaning of the response properties:

<table>
  <tr>
    <th>Property</th>
    <th>Description</th>
  </tr>
  <tr>
    <td><code>limit</code></td>
    <td>The maximum number of items that can be returned in the HTTP response.
    It equals to the <code>limit</code> query parameter if it was provided or
    the maximum limit enforced for the particular endpoint, whichever is smaller.</td>
  </tr>
  <tr>
    <td><code>isTruncated</code></td>
    <td><code>true</code> if there are more items left to be queried. Otherwise <code>false</code>.</td>
  </tr>
  <tr>
    <td><code>exclusiveStartKey</code></td>
    <td>The last key that was skipped at the start. Is `null` for the first page.</td>
  </tr>
  <tr>
    <td><code>nextExclusiveStartKey</code></td>
    <td>The value for the <code>exclusiveStartKey</code> parameter to query the next page of items.</td>
  </tr>
</table>

## Errors
<span id=\"/introduction/errors\"></span>

The Apify API uses common HTTP status codes: `2xx` range for success, `4xx`
range for errors caused by the caller
(invalid requests) and `5xx` range for server errors (these are rare).
Each error response contains a JSON object defining the `error` property,
which is an object with
the `type` and `message` properties that contain the error code and a
human-readable error description, respectively.

For example:

```
{
    \"error\": {
        \"type\": \"record-not-found\",
        \"message\": \"Store was not found.\"
    }
}
```

Here is the table of the most common errors that can occur for many API
endpoints:

<table>
  <tr>
    <th>status</th>
    <th>type</th>
    <th>message</th>
  </tr>
  <tr>
    <td><code>400</code></td>
    <td><code>invalid-request</code></td>
    <td>POST data must be a JSON object</td>
  </tr>
  <tr>
    <td><code>400</code></td>
    <td><code>invalid-value</code></td>
    <td>Invalid value provided: Comments required</td>
  </tr>
  <tr>
    <td><code>400</code></td>
    <td><code>invalid-record-key</code></td>
    <td>Record key contains invalid character</td>
  </tr>
  <tr>
    <td><code>401</code></td>
    <td><code>token-not-provided</code></td>
    <td>Authentication token was not provided</td>
  </tr>
  <tr>
    <td><code>404</code></td>
    <td><code>record-not-found</code></td>
    <td>Store was not found</td>
  </tr>
  <tr>
    <td><code>429</code></td>
    <td><code>rate-limit-exceeded</code></td>
    <td>You have exceeded the rate limit of ... requests per second</td>
  </tr>
  <tr>
    <td><code>405</code></td>
    <td><code>method-not-allowed</code></td>
    <td>This API endpoint can only be accessed using the following HTTP methods: OPTIONS, POST</td>
  </tr>
</table>

## Rate limiting
<span id=\"/introduction/rate-limiting\"></span>

All API endpoints limit the rate of requests in order to prevent overloading of Apify servers by misbehaving clients.

There are two kinds of rate limits - a global rate limit and a per-resource rate limit.

### Global rate limit
<span id=\"/introduction/rate-limiting/global-rate-limit\"></span>

The global rate limit is set to _250 000 requests per minute_.
For [authenticated](#/introduction/authentication) requests, it is counted per user,
and for unauthenticated requests, it is counted per IP address.

### Per-resource rate limit
<span id=\"/introduction/rate-limiting/per-resource-rate-limit\"></span>

The default per-resource rate limit is _60 requests per second per resource_, which in this context means a single Actor, a single Actor run, a single dataset, single key-value store etc.
The default rate limit is applied to every API endpoint except a few select ones, which have higher rate limits.
Each API endpoint returns its rate limit in `X-RateLimit-Limit` header.

These endpoints have a rate limit of _200 requests per second per resource_:

* CRUD ([get](#/reference/key-value-stores/record/get-record),
  [put](#/reference/key-value-stores/record/put-record),
  [delete](#/reference/key-value-stores/record/delete-record))
  operations on key-value store records

These endpoints have a rate limit of _400 requests per second per resource_:
* [Run Actor](#/reference/actors/run-collection/run-actor)
* [Run Actor task asynchronously](#/reference/actor-tasks/runs-collection/run-task-asynchronously)
* [Run Actor task synchronously](#/reference/actor-tasks/runs-collection/run-task-synchronously)
* [Metamorph Actor run](#/reference/actors/metamorph-run/metamorph-run)
* [Push items](#/reference/datasets/item-collection/put-items) to dataset
* CRUD
  ([add](#/reference/request-queues/request-collection/add-request),
  [get](#/reference/request-queues/request-collection/get-request),
  [update](#/reference/request-queues/request-collection/update-request),
  [delete](#/reference/request-queues/request-collection/delete-request))
  operations on requests in request queues

### Rate limit exceeded errors
<span id=\"/introduction/rate-limiting/rate-limit-exceeded-errors\"></span>

If the client is sending too many requests, the API endpoints respond with the HTTP status code `429 Too Many Requests`
and the following body:

```
{
    \"error\": {
        \"type\": \"rate-limit-exceeded\",
        \"message\": \"You have exceeded the rate limit of ... requests per second\"
    }
}
```

### Retrying rate-limited requests with exponential backoff
<span id=\"/introduction/rate-limiting/retrying-rate-limited-requests-with-exponential-backoff\"></span>

If the client receives the rate limit error, it should wait a certain period of time and then retry the request.
If the error happens again, the client should double the wait period and retry the request,
and so on. This algorithm is known as _exponential backoff_
and it can be described using the following pseudo-code:

1. Define a variable `DELAY=500`
2. Send the HTTP request to the API endpoint
3. If the response has status code not equal to `429` then you are done. Otherwise:
   * Wait for a period of time chosen randomly from the interval `DELAY` to `2*DELAY` milliseconds
   * Double the future wait period by setting `DELAY = 2*DELAY`
   * Continue with step 2

If all requests sent by the client implement the above steps,
the client will automatically use the maximum available bandwidth for its requests.

Note that the Apify API clients [for JavaScript](https://docs.apify.com/api/client/js)
and [for Python](https://docs.apify.com/api/client/python)
use the exponential backoff algorithm transparently, so that you do not need to worry about it.

## Referring to resources
<span id=\"/introduction/referring-to-resources\"></span>

There are three main ways to refer to a resource you're accessing via API.

- the resource ID (e.g. `iKkPcIgVvwmztduf8`)
- `username~resourcename` - when using this access method, you will need to
use your API token, and access will only work if you have the correct
permissions.
- `~resourcename` - for this, you need to use an API token, and the
`resourcename` refers to a resource in the API token owner's account.



## Overview

This API client was generated by the [OpenAPI Generator](https://openapi-generator.tech) project.  By using the [openapi-spec](https://openapis.org) from a remote server, you can easily generate an API client.

- API version: v2-2026-06-16T064758Z
- Package version: v2-2026-06-16T064758Z
- Generator version: 7.23.0
- Build package: `org.openapitools.codegen.languages.RustClientCodegen`

## Installation

Put the package under your project folder in a directory named `apify-client` and add the following to `Cargo.toml` under `[dependencies]`:

```
apify-client = { path = "./apify-client" }
```

## Documentation for API Endpoints

All URIs are relative to *https://api.apify.com*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*ActorBuildsApi* | [**actor_build_abort_post**](docs/ActorBuildsApi.md#actor_build_abort_post) | **POST** /v2/actor-builds/{buildId}/abort | Abort build
*ActorBuildsApi* | [**actor_build_delete**](docs/ActorBuildsApi.md#actor_build_delete) | **DELETE** /v2/actor-builds/{buildId} | Delete build
*ActorBuildsApi* | [**actor_build_get**](docs/ActorBuildsApi.md#actor_build_get) | **GET** /v2/actor-builds/{buildId} | Get build
*ActorBuildsApi* | [**actor_build_log_get**](docs/ActorBuildsApi.md#actor_build_log_get) | **GET** /v2/actor-builds/{buildId}/log | Get build's Log
*ActorBuildsApi* | [**actor_build_openapi_json_get**](docs/ActorBuildsApi.md#actor_build_openapi_json_get) | **GET** /v2/actor-builds/{buildId}/openapi.json | Get OpenAPI definition
*ActorBuildsApi* | [**actor_builds_get**](docs/ActorBuildsApi.md#actor_builds_get) | **GET** /v2/actor-builds | Get user builds list
*ActorRunsApi* | [**actor_run_abort_post**](docs/ActorRunsApi.md#actor_run_abort_post) | **POST** /v2/actor-runs/{runId}/abort | Abort run
*ActorRunsApi* | [**actor_run_delete**](docs/ActorRunsApi.md#actor_run_delete) | **DELETE** /v2/actor-runs/{runId} | Delete run
*ActorRunsApi* | [**actor_run_get**](docs/ActorRunsApi.md#actor_run_get) | **GET** /v2/actor-runs/{runId} | Get run
*ActorRunsApi* | [**actor_run_log_get**](docs/ActorRunsApi.md#actor_run_log_get) | **GET** /v2/actor-runs/{runId}/log | Get run's log
*ActorRunsApi* | [**actor_run_metamorph_post**](docs/ActorRunsApi.md#actor_run_metamorph_post) | **POST** /v2/actor-runs/{runId}/metamorph | Metamorph run
*ActorRunsApi* | [**actor_run_put**](docs/ActorRunsApi.md#actor_run_put) | **PUT** /v2/actor-runs/{runId} | Update run
*ActorRunsApi* | [**actor_run_reboot_post**](docs/ActorRunsApi.md#actor_run_reboot_post) | **POST** /v2/actor-runs/{runId}/reboot | Reboot run
*ActorRunsApi* | [**actor_runs_get**](docs/ActorRunsApi.md#actor_runs_get) | **GET** /v2/actor-runs | Get user runs list
*ActorRunsApi* | [**post_charge_run**](docs/ActorRunsApi.md#post_charge_run) | **POST** /v2/actor-runs/{runId}/charge | Charge events in run
*ActorRunsApi* | [**post_resurrect_run**](docs/ActorRunsApi.md#post_resurrect_run) | **POST** /v2/actor-runs/{runId}/resurrect | Resurrect run
*ActorTasksApi* | [**actor_task_delete**](docs/ActorTasksApi.md#actor_task_delete) | **DELETE** /v2/actor-tasks/{actorTaskId} | Delete task
*ActorTasksApi* | [**actor_task_get**](docs/ActorTasksApi.md#actor_task_get) | **GET** /v2/actor-tasks/{actorTaskId} | Get task
*ActorTasksApi* | [**actor_task_input_get**](docs/ActorTasksApi.md#actor_task_input_get) | **GET** /v2/actor-tasks/{actorTaskId}/input | Get task input
*ActorTasksApi* | [**actor_task_input_put**](docs/ActorTasksApi.md#actor_task_input_put) | **PUT** /v2/actor-tasks/{actorTaskId}/input | Update task input
*ActorTasksApi* | [**actor_task_put**](docs/ActorTasksApi.md#actor_task_put) | **PUT** /v2/actor-tasks/{actorTaskId} | Update task
*ActorTasksApi* | [**actor_task_run_sync_get**](docs/ActorTasksApi.md#actor_task_run_sync_get) | **GET** /v2/actor-tasks/{actorTaskId}/run-sync | Run task synchronously
*ActorTasksApi* | [**actor_task_run_sync_get_dataset_items_get**](docs/ActorTasksApi.md#actor_task_run_sync_get_dataset_items_get) | **GET** /v2/actor-tasks/{actorTaskId}/run-sync-get-dataset-items | Run task synchronously and get dataset items
*ActorTasksApi* | [**actor_task_run_sync_get_dataset_items_post**](docs/ActorTasksApi.md#actor_task_run_sync_get_dataset_items_post) | **POST** /v2/actor-tasks/{actorTaskId}/run-sync-get-dataset-items | Run task synchronously and get dataset items
*ActorTasksApi* | [**actor_task_run_sync_post**](docs/ActorTasksApi.md#actor_task_run_sync_post) | **POST** /v2/actor-tasks/{actorTaskId}/run-sync | Run task synchronously
*ActorTasksApi* | [**actor_task_runs_get**](docs/ActorTasksApi.md#actor_task_runs_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs | Get list of task runs
*ActorTasksApi* | [**actor_task_runs_last_get**](docs/ActorTasksApi.md#actor_task_runs_last_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last | Get last run
*ActorTasksApi* | [**actor_task_runs_post**](docs/ActorTasksApi.md#actor_task_runs_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs | Run task
*ActorTasksApi* | [**actor_task_webhooks_get**](docs/ActorTasksApi.md#actor_task_webhooks_get) | **GET** /v2/actor-tasks/{actorTaskId}/webhooks | Get list of webhooks
*ActorTasksApi* | [**actor_tasks_get**](docs/ActorTasksApi.md#actor_tasks_get) | **GET** /v2/actor-tasks | Get list of tasks
*ActorTasksApi* | [**actor_tasks_post**](docs/ActorTasksApi.md#actor_tasks_post) | **POST** /v2/actor-tasks | Create task
*ActorsApi* | [**act_delete**](docs/ActorsApi.md#act_delete) | **DELETE** /v2/actors/{actorId} | Delete Actor
*ActorsApi* | [**act_get**](docs/ActorsApi.md#act_get) | **GET** /v2/actors/{actorId} | Get Actor
*ActorsApi* | [**act_put**](docs/ActorsApi.md#act_put) | **PUT** /v2/actors/{actorId} | Update Actor
*ActorsApi* | [**act_validate_input_post**](docs/ActorsApi.md#act_validate_input_post) | **POST** /v2/actors/{actorId}/validate-input | Validate Actor input
*ActorsApi* | [**acts_get**](docs/ActorsApi.md#acts_get) | **GET** /v2/actors | Get list of Actors
*ActorsApi* | [**acts_post**](docs/ActorsApi.md#acts_post) | **POST** /v2/actors | Create Actor
*ActorsActorBuildsApi* | [**act_build_abort_post**](docs/ActorsActorBuildsApi.md#act_build_abort_post) | **POST** /v2/actors/{actorId}/builds/{buildId}/abort | Abort build
*ActorsActorBuildsApi* | [**act_build_default_get**](docs/ActorsActorBuildsApi.md#act_build_default_get) | **GET** /v2/actors/{actorId}/builds/default | Get default build
*ActorsActorBuildsApi* | [**act_build_get**](docs/ActorsActorBuildsApi.md#act_build_get) | **GET** /v2/actors/{actorId}/builds/{buildId} | Get build
*ActorsActorBuildsApi* | [**act_builds_get**](docs/ActorsActorBuildsApi.md#act_builds_get) | **GET** /v2/actors/{actorId}/builds | Get list of builds
*ActorsActorBuildsApi* | [**act_builds_post**](docs/ActorsActorBuildsApi.md#act_builds_post) | **POST** /v2/actors/{actorId}/builds | Build Actor
*ActorsActorBuildsApi* | [**act_openapi_json_get**](docs/ActorsActorBuildsApi.md#act_openapi_json_get) | **GET** /v2/actors/{actorId}/builds/{buildId}/openapi.json | Get OpenAPI definition
*ActorsActorRunsApi* | [**act_run_abort_post**](docs/ActorsActorRunsApi.md#act_run_abort_post) | **POST** /v2/actors/{actorId}/runs/{runId}/abort | Abort run
*ActorsActorRunsApi* | [**act_run_get**](docs/ActorsActorRunsApi.md#act_run_get) | **GET** /v2/actors/{actorId}/runs/{runId} | Get run
*ActorsActorRunsApi* | [**act_run_metamorph_post**](docs/ActorsActorRunsApi.md#act_run_metamorph_post) | **POST** /v2/actors/{actorId}/runs/{runId}/metamorph | Metamorph run
*ActorsActorRunsApi* | [**act_run_resurrect_post**](docs/ActorsActorRunsApi.md#act_run_resurrect_post) | **POST** /v2/actors/{actorId}/runs/{runId}/resurrect | Resurrect run
*ActorsActorRunsApi* | [**act_run_sync_get**](docs/ActorsActorRunsApi.md#act_run_sync_get) | **GET** /v2/actors/{actorId}/run-sync | Run Actor synchronously without input
*ActorsActorRunsApi* | [**act_run_sync_get_dataset_items_get**](docs/ActorsActorRunsApi.md#act_run_sync_get_dataset_items_get) | **GET** /v2/actors/{actorId}/run-sync-get-dataset-items | Run Actor synchronously without input and get dataset items
*ActorsActorRunsApi* | [**act_run_sync_get_dataset_items_post**](docs/ActorsActorRunsApi.md#act_run_sync_get_dataset_items_post) | **POST** /v2/actors/{actorId}/run-sync-get-dataset-items | Run Actor synchronously and get dataset items
*ActorsActorRunsApi* | [**act_run_sync_post**](docs/ActorsActorRunsApi.md#act_run_sync_post) | **POST** /v2/actors/{actorId}/run-sync | Run Actor synchronously and return output
*ActorsActorRunsApi* | [**act_runs_get**](docs/ActorsActorRunsApi.md#act_runs_get) | **GET** /v2/actors/{actorId}/runs | Get list of runs
*ActorsActorRunsApi* | [**act_runs_last_get**](docs/ActorsActorRunsApi.md#act_runs_last_get) | **GET** /v2/actors/{actorId}/runs/last | Get last run
*ActorsActorRunsApi* | [**act_runs_post**](docs/ActorsActorRunsApi.md#act_runs_post) | **POST** /v2/actors/{actorId}/runs | Run Actor
*ActorsActorVersionsApi* | [**act_version_delete**](docs/ActorsActorVersionsApi.md#act_version_delete) | **DELETE** /v2/actors/{actorId}/versions/{versionNumber} | Delete version
*ActorsActorVersionsApi* | [**act_version_env_var_delete**](docs/ActorsActorVersionsApi.md#act_version_env_var_delete) | **DELETE** /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName} | Delete environment variable
*ActorsActorVersionsApi* | [**act_version_env_var_get**](docs/ActorsActorVersionsApi.md#act_version_env_var_get) | **GET** /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName} | Get environment variable
*ActorsActorVersionsApi* | [**act_version_env_var_post**](docs/ActorsActorVersionsApi.md#act_version_env_var_post) | **POST** /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName} | Update environment variable (POST)
*ActorsActorVersionsApi* | [**act_version_env_var_put**](docs/ActorsActorVersionsApi.md#act_version_env_var_put) | **PUT** /v2/actors/{actorId}/versions/{versionNumber}/env-vars/{envVarName} | Update environment variable
*ActorsActorVersionsApi* | [**act_version_env_vars_get**](docs/ActorsActorVersionsApi.md#act_version_env_vars_get) | **GET** /v2/actors/{actorId}/versions/{versionNumber}/env-vars | Get list of environment variables
*ActorsActorVersionsApi* | [**act_version_env_vars_post**](docs/ActorsActorVersionsApi.md#act_version_env_vars_post) | **POST** /v2/actors/{actorId}/versions/{versionNumber}/env-vars | Create environment variable
*ActorsActorVersionsApi* | [**act_version_get**](docs/ActorsActorVersionsApi.md#act_version_get) | **GET** /v2/actors/{actorId}/versions/{versionNumber} | Get version
*ActorsActorVersionsApi* | [**act_version_post**](docs/ActorsActorVersionsApi.md#act_version_post) | **POST** /v2/actors/{actorId}/versions/{versionNumber} | Update version (POST)
*ActorsActorVersionsApi* | [**act_version_put**](docs/ActorsActorVersionsApi.md#act_version_put) | **PUT** /v2/actors/{actorId}/versions/{versionNumber} | Update version
*ActorsActorVersionsApi* | [**act_versions_get**](docs/ActorsActorVersionsApi.md#act_versions_get) | **GET** /v2/actors/{actorId}/versions | Get list of versions
*ActorsActorVersionsApi* | [**act_versions_post**](docs/ActorsActorVersionsApi.md#act_versions_post) | **POST** /v2/actors/{actorId}/versions | Create version
*ActorsWebhookCollectionApi* | [**act_webhooks_get**](docs/ActorsWebhookCollectionApi.md#act_webhooks_get) | **GET** /v2/actors/{actorId}/webhooks | Get list of webhooks
*DefaultDatasetApi* | [**actor_run_dataset_delete**](docs/DefaultDatasetApi.md#actor_run_dataset_delete) | **DELETE** /v2/actor-runs/{runId}/dataset | Delete default dataset
*DefaultDatasetApi* | [**actor_run_dataset_get**](docs/DefaultDatasetApi.md#actor_run_dataset_get) | **GET** /v2/actor-runs/{runId}/dataset | Get default dataset
*DefaultDatasetApi* | [**actor_run_dataset_items_get**](docs/DefaultDatasetApi.md#actor_run_dataset_items_get) | **GET** /v2/actor-runs/{runId}/dataset/items | Get default dataset items
*DefaultDatasetApi* | [**actor_run_dataset_items_post**](docs/DefaultDatasetApi.md#actor_run_dataset_items_post) | **POST** /v2/actor-runs/{runId}/dataset/items | Store items
*DefaultDatasetApi* | [**actor_run_dataset_put**](docs/DefaultDatasetApi.md#actor_run_dataset_put) | **PUT** /v2/actor-runs/{runId}/dataset | Update default dataset
*DefaultDatasetApi* | [**actor_run_dataset_statistics_get**](docs/DefaultDatasetApi.md#actor_run_dataset_statistics_get) | **GET** /v2/actor-runs/{runId}/dataset/statistics | Get default dataset statistics
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_delete**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_delete) | **DELETE** /v2/actor-runs/{runId}/key-value-store | Delete default store
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_get**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_get) | **GET** /v2/actor-runs/{runId}/key-value-store | Get default store
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_keys_get**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_keys_get) | **GET** /v2/actor-runs/{runId}/key-value-store/keys | Get default store's list of keys
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_put**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_put) | **PUT** /v2/actor-runs/{runId}/key-value-store | Update default store
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_record_delete**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_record_delete) | **DELETE** /v2/actor-runs/{runId}/key-value-store/records/{recordKey} | Delete default store's record
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_record_get**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_record_get) | **GET** /v2/actor-runs/{runId}/key-value-store/records/{recordKey} | Get default store's record
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_record_post**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_record_post) | **POST** /v2/actor-runs/{runId}/key-value-store/records/{recordKey} | Store record in default store (POST)
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_record_put**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_record_put) | **PUT** /v2/actor-runs/{runId}/key-value-store/records/{recordKey} | Store record in default store
*DefaultKeyValueStoreApi* | [**actor_run_key_value_store_records_get**](docs/DefaultKeyValueStoreApi.md#actor_run_key_value_store_records_get) | **GET** /v2/actor-runs/{runId}/key-value-store/records | Download default store's records
*DefaultRequestQueueApi* | [**actor_run_request_queue_delete**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_delete) | **DELETE** /v2/actor-runs/{runId}/request-queue | Delete default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_get**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_get) | **GET** /v2/actor-runs/{runId}/request-queue | Get default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_head_get**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_head_get) | **GET** /v2/actor-runs/{runId}/request-queue/head | Get default request queue head
*DefaultRequestQueueApi* | [**actor_run_request_queue_head_lock_post**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_head_lock_post) | **POST** /v2/actor-runs/{runId}/request-queue/head/lock | Get and lock default request queue head
*DefaultRequestQueueApi* | [**actor_run_request_queue_put**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_put) | **PUT** /v2/actor-runs/{runId}/request-queue | Update default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_request_delete**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_request_delete) | **DELETE** /v2/actor-runs/{runId}/request-queue/requests/{requestId} | Delete request from default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_request_get**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_request_get) | **GET** /v2/actor-runs/{runId}/request-queue/requests/{requestId} | Get request from default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_request_lock_delete**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_request_lock_delete) | **DELETE** /v2/actor-runs/{runId}/request-queue/requests/{requestId}/lock | Delete lock on request in default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_request_lock_put**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_request_lock_put) | **PUT** /v2/actor-runs/{runId}/request-queue/requests/{requestId}/lock | Prolong lock on request in default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_request_put**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_request_put) | **PUT** /v2/actor-runs/{runId}/request-queue/requests/{requestId} | Update request in default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_requests_batch_delete**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_requests_batch_delete) | **DELETE** /v2/actor-runs/{runId}/request-queue/requests/batch | Batch delete requests from default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_requests_batch_post**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_requests_batch_post) | **POST** /v2/actor-runs/{runId}/request-queue/requests/batch | Batch add requests to default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_requests_get**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_requests_get) | **GET** /v2/actor-runs/{runId}/request-queue/requests | List default request queue's requests
*DefaultRequestQueueApi* | [**actor_run_request_queue_requests_post**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_requests_post) | **POST** /v2/actor-runs/{runId}/request-queue/requests | Add request to default request queue
*DefaultRequestQueueApi* | [**actor_run_request_queue_requests_unlock_post**](docs/DefaultRequestQueueApi.md#actor_run_request_queue_requests_unlock_post) | **POST** /v2/actor-runs/{runId}/request-queue/requests/unlock | Unlock requests in default request queue
*LastActorRunsAbortApi* | [**act_runs_last_abort_post**](docs/LastActorRunsAbortApi.md#act_runs_last_abort_post) | **POST** /v2/actors/{actorId}/runs/last/abort | Abort Actor's last run
*LastActorRunsDefaultDatasetApi* | [**act_runs_last_dataset_delete**](docs/LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_delete) | **DELETE** /v2/actors/{actorId}/runs/last/dataset | Delete last run's default dataset
*LastActorRunsDefaultDatasetApi* | [**act_runs_last_dataset_get**](docs/LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_get) | **GET** /v2/actors/{actorId}/runs/last/dataset | Get last run's default dataset
*LastActorRunsDefaultDatasetApi* | [**act_runs_last_dataset_items_get**](docs/LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_items_get) | **GET** /v2/actors/{actorId}/runs/last/dataset/items | Get last run's dataset items
*LastActorRunsDefaultDatasetApi* | [**act_runs_last_dataset_items_post**](docs/LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_items_post) | **POST** /v2/actors/{actorId}/runs/last/dataset/items | Store items in last run's dataset
*LastActorRunsDefaultDatasetApi* | [**act_runs_last_dataset_put**](docs/LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_put) | **PUT** /v2/actors/{actorId}/runs/last/dataset | Update last run's default dataset
*LastActorRunsDefaultDatasetApi* | [**act_runs_last_dataset_statistics_get**](docs/LastActorRunsDefaultDatasetApi.md#act_runs_last_dataset_statistics_get) | **GET** /v2/actors/{actorId}/runs/last/dataset/statistics | Get last run's dataset statistics
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_delete**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_delete) | **DELETE** /v2/actors/{actorId}/runs/last/key-value-store | Delete last run's default store
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_get**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_get) | **GET** /v2/actors/{actorId}/runs/last/key-value-store | Get last run's default store
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_keys_get**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_keys_get) | **GET** /v2/actors/{actorId}/runs/last/key-value-store/keys | Get last run's default store's list of keys
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_put**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_put) | **PUT** /v2/actors/{actorId}/runs/last/key-value-store | Update last run's default store
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_record_delete**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_record_delete) | **DELETE** /v2/actors/{actorId}/runs/last/key-value-store/records/{recordKey} | Delete last run's default store's record
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_record_get**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_record_get) | **GET** /v2/actors/{actorId}/runs/last/key-value-store/records/{recordKey} | Get last run's default store's record
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_record_post**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_record_post) | **POST** /v2/actors/{actorId}/runs/last/key-value-store/records/{recordKey} | Store record in last run's default store (POST)
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_record_put**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_record_put) | **PUT** /v2/actors/{actorId}/runs/last/key-value-store/records/{recordKey} | Store record in last run's default store
*LastActorRunsDefaultKeyValueStoreApi* | [**act_runs_last_key_value_store_records_get**](docs/LastActorRunsDefaultKeyValueStoreApi.md#act_runs_last_key_value_store_records_get) | **GET** /v2/actors/{actorId}/runs/last/key-value-store/records | Download last run's default store's records
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_delete**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_delete) | **DELETE** /v2/actors/{actorId}/runs/last/request-queue | Delete last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_get**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_get) | **GET** /v2/actors/{actorId}/runs/last/request-queue | Get last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_head_get**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_head_get) | **GET** /v2/actors/{actorId}/runs/last/request-queue/head | Get last run's default request queue head
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_head_lock_post**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_head_lock_post) | **POST** /v2/actors/{actorId}/runs/last/request-queue/head/lock | Get and lock last run's default request queue head
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_put**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_put) | **PUT** /v2/actors/{actorId}/runs/last/request-queue | Update last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_request_delete**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_delete) | **DELETE** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId} | Delete request from last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_request_get**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_get) | **GET** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId} | Get request from last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_request_lock_delete**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_lock_delete) | **DELETE** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId}/lock | Delete lock on request in last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_request_lock_put**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_lock_put) | **PUT** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId}/lock | Prolong lock on request in last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_request_put**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_request_put) | **PUT** /v2/actors/{actorId}/runs/last/request-queue/requests/{requestId} | Update request in last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_requests_batch_delete**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_batch_delete) | **DELETE** /v2/actors/{actorId}/runs/last/request-queue/requests/batch | Batch delete requests from last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_requests_batch_post**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_batch_post) | **POST** /v2/actors/{actorId}/runs/last/request-queue/requests/batch | Batch add requests to last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_requests_get**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_get) | **GET** /v2/actors/{actorId}/runs/last/request-queue/requests | List last run's default request queue's requests
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_requests_post**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_post) | **POST** /v2/actors/{actorId}/runs/last/request-queue/requests | Add request to last run's default request queue
*LastActorRunsDefaultRequestQueueApi* | [**act_runs_last_request_queue_requests_unlock_post**](docs/LastActorRunsDefaultRequestQueueApi.md#act_runs_last_request_queue_requests_unlock_post) | **POST** /v2/actors/{actorId}/runs/last/request-queue/requests/unlock | Unlock requests in last run's default request queue
*LastActorRunsLogApi* | [**act_runs_last_log_get**](docs/LastActorRunsLogApi.md#act_runs_last_log_get) | **GET** /v2/actors/{actorId}/runs/last/log | Get last Actor run's log
*LastActorRunsMetamorphApi* | [**act_runs_last_metamorph_post**](docs/LastActorRunsMetamorphApi.md#act_runs_last_metamorph_post) | **POST** /v2/actors/{actorId}/runs/last/metamorph | Metamorph Actor's last run
*LastActorRunsRebootApi* | [**act_runs_last_reboot_post**](docs/LastActorRunsRebootApi.md#act_runs_last_reboot_post) | **POST** /v2/actors/{actorId}/runs/last/reboot | Reboot Actor's last run
*LastActorTaskRunsAbortApi* | [**actor_task_runs_last_abort_post**](docs/LastActorTaskRunsAbortApi.md#actor_task_runs_last_abort_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/abort | Abort Actor task's last run
*LastActorTaskRunsDefaultDatasetApi* | [**actor_task_runs_last_dataset_delete**](docs/LastActorTaskRunsDefaultDatasetApi.md#actor_task_runs_last_dataset_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/dataset | Delete last task run's default dataset
*LastActorTaskRunsDefaultDatasetApi* | [**actor_task_runs_last_dataset_get**](docs/LastActorTaskRunsDefaultDatasetApi.md#actor_task_runs_last_dataset_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/dataset | Get last task run's default dataset
*LastActorTaskRunsDefaultDatasetApi* | [**actor_task_runs_last_dataset_items_get**](docs/LastActorTaskRunsDefaultDatasetApi.md#actor_task_runs_last_dataset_items_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/dataset/items | Get last task run's dataset items
*LastActorTaskRunsDefaultDatasetApi* | [**actor_task_runs_last_dataset_items_post**](docs/LastActorTaskRunsDefaultDatasetApi.md#actor_task_runs_last_dataset_items_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/dataset/items | Store items in last task run's dataset
*LastActorTaskRunsDefaultDatasetApi* | [**actor_task_runs_last_dataset_put**](docs/LastActorTaskRunsDefaultDatasetApi.md#actor_task_runs_last_dataset_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/dataset | Update last task run's default dataset
*LastActorTaskRunsDefaultDatasetApi* | [**actor_task_runs_last_dataset_statistics_get**](docs/LastActorTaskRunsDefaultDatasetApi.md#actor_task_runs_last_dataset_statistics_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/dataset/statistics | Get last task run's dataset statistics
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_delete**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store | Delete last task run's default store
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_get**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store | Get last task run's default store
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_keys_get**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_keys_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/keys | Get last task run's default store's list of keys
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_put**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store | Update last task run's default store
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_record_delete**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_record_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records/{recordKey} | Delete last task run's default store's record
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_record_get**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_record_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records/{recordKey} | Get last task run's default store's record
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_record_post**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_record_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records/{recordKey} | Store record in last task run's default store (POST)
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_record_put**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_record_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records/{recordKey} | Store record in last task run's default store
*LastActorTaskRunsDefaultKeyValueStoreApi* | [**actor_task_runs_last_key_value_store_records_get**](docs/LastActorTaskRunsDefaultKeyValueStoreApi.md#actor_task_runs_last_key_value_store_records_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/key-value-store/records | Download last task run's default store's records
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_delete**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue | Delete last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_get**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue | Get last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_head_get**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_head_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/head | Get last task run's default request queue head
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_head_lock_post**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_head_lock_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/head/lock | Get and lock last task run's default request queue head
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_put**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue | Update last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_request_delete**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId} | Delete request from last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_request_get**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId} | Get request from last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_request_lock_delete**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_lock_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId}/lock | Delete lock on request in last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_request_lock_put**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_lock_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId}/lock | Prolong lock on request in last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_request_put**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_request_put) | **PUT** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/{requestId} | Update request in last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_requests_batch_delete**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_batch_delete) | **DELETE** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/batch | Batch delete requests from last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_requests_batch_post**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_batch_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/batch | Batch add requests to last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_requests_get**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests | List last task run's default request queue's requests
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_requests_post**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests | Add request to last task run's default request queue
*LastActorTaskRunsDefaultRequestQueueApi* | [**actor_task_runs_last_request_queue_requests_unlock_post**](docs/LastActorTaskRunsDefaultRequestQueueApi.md#actor_task_runs_last_request_queue_requests_unlock_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/request-queue/requests/unlock | Unlock requests in last task run's default request queue
*LastActorTaskRunsLogApi* | [**actor_task_last_log_get**](docs/LastActorTaskRunsLogApi.md#actor_task_last_log_get) | **GET** /v2/actor-tasks/{actorTaskId}/runs/last/log | Get last Actor task run's log
*LastActorTaskRunsMetamorphApi* | [**actor_task_runs_last_metamorph_post**](docs/LastActorTaskRunsMetamorphApi.md#actor_task_runs_last_metamorph_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/metamorph | Metamorph Actor task's last run
*LastActorTaskRunsRebootApi* | [**actor_task_runs_last_reboot_post**](docs/LastActorTaskRunsRebootApi.md#actor_task_runs_last_reboot_post) | **POST** /v2/actor-tasks/{actorTaskId}/runs/last/reboot | Reboot Actor task's last run
*LogsApi* | [**log_get**](docs/LogsApi.md#log_get) | **GET** /v2/logs/{buildOrRunId} | Get log
*SchedulesApi* | [**schedule_delete**](docs/SchedulesApi.md#schedule_delete) | **DELETE** /v2/schedules/{scheduleId} | Delete schedule
*SchedulesApi* | [**schedule_get**](docs/SchedulesApi.md#schedule_get) | **GET** /v2/schedules/{scheduleId} | Get schedule
*SchedulesApi* | [**schedule_log_get**](docs/SchedulesApi.md#schedule_log_get) | **GET** /v2/schedules/{scheduleId}/log | Get schedule log
*SchedulesApi* | [**schedule_put**](docs/SchedulesApi.md#schedule_put) | **PUT** /v2/schedules/{scheduleId} | Update schedule
*SchedulesApi* | [**schedules_get**](docs/SchedulesApi.md#schedules_get) | **GET** /v2/schedules | Get list of schedules
*SchedulesApi* | [**schedules_post**](docs/SchedulesApi.md#schedules_post) | **POST** /v2/schedules | Create schedule
*StorageDatasetsApi* | [**dataset_delete**](docs/StorageDatasetsApi.md#dataset_delete) | **DELETE** /v2/datasets/{datasetId} | Delete dataset
*StorageDatasetsApi* | [**dataset_get**](docs/StorageDatasetsApi.md#dataset_get) | **GET** /v2/datasets/{datasetId} | Get dataset
*StorageDatasetsApi* | [**dataset_items_get**](docs/StorageDatasetsApi.md#dataset_items_get) | **GET** /v2/datasets/{datasetId}/items | Get dataset items
*StorageDatasetsApi* | [**dataset_items_head**](docs/StorageDatasetsApi.md#dataset_items_head) | **HEAD** /v2/datasets/{datasetId}/items | Get dataset items headers
*StorageDatasetsApi* | [**dataset_items_post**](docs/StorageDatasetsApi.md#dataset_items_post) | **POST** /v2/datasets/{datasetId}/items | Store items
*StorageDatasetsApi* | [**dataset_put**](docs/StorageDatasetsApi.md#dataset_put) | **PUT** /v2/datasets/{datasetId} | Update dataset
*StorageDatasetsApi* | [**dataset_statistics_get**](docs/StorageDatasetsApi.md#dataset_statistics_get) | **GET** /v2/datasets/{datasetId}/statistics | Get dataset statistics
*StorageDatasetsApi* | [**datasets_get**](docs/StorageDatasetsApi.md#datasets_get) | **GET** /v2/datasets | Get list of datasets
*StorageDatasetsApi* | [**datasets_post**](docs/StorageDatasetsApi.md#datasets_post) | **POST** /v2/datasets | Create dataset
*StorageKeyValueStoresApi* | [**key_value_store_delete**](docs/StorageKeyValueStoresApi.md#key_value_store_delete) | **DELETE** /v2/key-value-stores/{storeId} | Delete store
*StorageKeyValueStoresApi* | [**key_value_store_get**](docs/StorageKeyValueStoresApi.md#key_value_store_get) | **GET** /v2/key-value-stores/{storeId} | Get store
*StorageKeyValueStoresApi* | [**key_value_store_keys_get**](docs/StorageKeyValueStoresApi.md#key_value_store_keys_get) | **GET** /v2/key-value-stores/{storeId}/keys | Get list of keys
*StorageKeyValueStoresApi* | [**key_value_store_put**](docs/StorageKeyValueStoresApi.md#key_value_store_put) | **PUT** /v2/key-value-stores/{storeId} | Update store
*StorageKeyValueStoresApi* | [**key_value_store_record_delete**](docs/StorageKeyValueStoresApi.md#key_value_store_record_delete) | **DELETE** /v2/key-value-stores/{storeId}/records/{recordKey} | Delete record
*StorageKeyValueStoresApi* | [**key_value_store_record_get**](docs/StorageKeyValueStoresApi.md#key_value_store_record_get) | **GET** /v2/key-value-stores/{storeId}/records/{recordKey} | Get record
*StorageKeyValueStoresApi* | [**key_value_store_record_head**](docs/StorageKeyValueStoresApi.md#key_value_store_record_head) | **HEAD** /v2/key-value-stores/{storeId}/records/{recordKey} | Check if a record exists
*StorageKeyValueStoresApi* | [**key_value_store_record_post**](docs/StorageKeyValueStoresApi.md#key_value_store_record_post) | **POST** /v2/key-value-stores/{storeId}/records/{recordKey} | Store record (POST)
*StorageKeyValueStoresApi* | [**key_value_store_record_put**](docs/StorageKeyValueStoresApi.md#key_value_store_record_put) | **PUT** /v2/key-value-stores/{storeId}/records/{recordKey} | Store record
*StorageKeyValueStoresApi* | [**key_value_store_records_get**](docs/StorageKeyValueStoresApi.md#key_value_store_records_get) | **GET** /v2/key-value-stores/{storeId}/records | Download records
*StorageKeyValueStoresApi* | [**key_value_stores_get**](docs/StorageKeyValueStoresApi.md#key_value_stores_get) | **GET** /v2/key-value-stores | Get list of key-value stores
*StorageKeyValueStoresApi* | [**key_value_stores_post**](docs/StorageKeyValueStoresApi.md#key_value_stores_post) | **POST** /v2/key-value-stores | Create key-value store
*StorageRequestQueuesApi* | [**request_queue_delete**](docs/StorageRequestQueuesApi.md#request_queue_delete) | **DELETE** /v2/request-queues/{queueId} | Delete request queue
*StorageRequestQueuesApi* | [**request_queue_get**](docs/StorageRequestQueuesApi.md#request_queue_get) | **GET** /v2/request-queues/{queueId} | Get request queue
*StorageRequestQueuesApi* | [**request_queue_put**](docs/StorageRequestQueuesApi.md#request_queue_put) | **PUT** /v2/request-queues/{queueId} | Update request queue
*StorageRequestQueuesApi* | [**request_queue_requests_batch_delete**](docs/StorageRequestQueuesApi.md#request_queue_requests_batch_delete) | **DELETE** /v2/request-queues/{queueId}/requests/batch | Delete requests
*StorageRequestQueuesApi* | [**request_queue_requests_batch_post**](docs/StorageRequestQueuesApi.md#request_queue_requests_batch_post) | **POST** /v2/request-queues/{queueId}/requests/batch | Add requests
*StorageRequestQueuesApi* | [**request_queues_get**](docs/StorageRequestQueuesApi.md#request_queues_get) | **GET** /v2/request-queues | Get list of request queues
*StorageRequestQueuesApi* | [**request_queues_post**](docs/StorageRequestQueuesApi.md#request_queues_post) | **POST** /v2/request-queues | Create request queue
*StorageRequestQueuesRequestsApi* | [**request_queue_request_delete**](docs/StorageRequestQueuesRequestsApi.md#request_queue_request_delete) | **DELETE** /v2/request-queues/{queueId}/requests/{requestId} | Delete request
*StorageRequestQueuesRequestsApi* | [**request_queue_request_get**](docs/StorageRequestQueuesRequestsApi.md#request_queue_request_get) | **GET** /v2/request-queues/{queueId}/requests/{requestId} | Get request
*StorageRequestQueuesRequestsApi* | [**request_queue_request_put**](docs/StorageRequestQueuesRequestsApi.md#request_queue_request_put) | **PUT** /v2/request-queues/{queueId}/requests/{requestId} | Update request
*StorageRequestQueuesRequestsApi* | [**request_queue_requests_get**](docs/StorageRequestQueuesRequestsApi.md#request_queue_requests_get) | **GET** /v2/request-queues/{queueId}/requests | List requests
*StorageRequestQueuesRequestsApi* | [**request_queue_requests_post**](docs/StorageRequestQueuesRequestsApi.md#request_queue_requests_post) | **POST** /v2/request-queues/{queueId}/requests | Add request
*StorageRequestQueuesRequestsLocksApi* | [**request_queue_head_get**](docs/StorageRequestQueuesRequestsLocksApi.md#request_queue_head_get) | **GET** /v2/request-queues/{queueId}/head | Get head
*StorageRequestQueuesRequestsLocksApi* | [**request_queue_head_lock_post**](docs/StorageRequestQueuesRequestsLocksApi.md#request_queue_head_lock_post) | **POST** /v2/request-queues/{queueId}/head/lock | Get head and lock
*StorageRequestQueuesRequestsLocksApi* | [**request_queue_request_lock_delete**](docs/StorageRequestQueuesRequestsLocksApi.md#request_queue_request_lock_delete) | **DELETE** /v2/request-queues/{queueId}/requests/{requestId}/lock | Delete request lock
*StorageRequestQueuesRequestsLocksApi* | [**request_queue_request_lock_put**](docs/StorageRequestQueuesRequestsLocksApi.md#request_queue_request_lock_put) | **PUT** /v2/request-queues/{queueId}/requests/{requestId}/lock | Prolong request lock
*StorageRequestQueuesRequestsLocksApi* | [**request_queue_requests_unlock_post**](docs/StorageRequestQueuesRequestsLocksApi.md#request_queue_requests_unlock_post) | **POST** /v2/request-queues/{queueId}/requests/unlock | Unlock requests
*StoreApi* | [**store_get**](docs/StoreApi.md#store_get) | **GET** /v2/store | Get list of Actors in Store
*ToolsApi* | [**tools_browser_info_delete**](docs/ToolsApi.md#tools_browser_info_delete) | **DELETE** /v2/browser-info | Get browser info
*ToolsApi* | [**tools_browser_info_get**](docs/ToolsApi.md#tools_browser_info_get) | **GET** /v2/browser-info | Get browser info
*ToolsApi* | [**tools_browser_info_post**](docs/ToolsApi.md#tools_browser_info_post) | **POST** /v2/browser-info | Get browser info
*ToolsApi* | [**tools_browser_info_put**](docs/ToolsApi.md#tools_browser_info_put) | **PUT** /v2/browser-info | Get browser info
*ToolsApi* | [**tools_decode_and_verify_post**](docs/ToolsApi.md#tools_decode_and_verify_post) | **POST** /v2/tools/decode-and-verify | Decode and verify object
*ToolsApi* | [**tools_encode_and_sign_post**](docs/ToolsApi.md#tools_encode_and_sign_post) | **POST** /v2/tools/encode-and-sign | Encode and sign object
*UsersApi* | [**user_get**](docs/UsersApi.md#user_get) | **GET** /v2/users/{userId} | Get public user data
*UsersApi* | [**users_me_get**](docs/UsersApi.md#users_me_get) | **GET** /v2/users/me | Get private user data
*UsersApi* | [**users_me_limits_get**](docs/UsersApi.md#users_me_limits_get) | **GET** /v2/users/me/limits | Get limits
*UsersApi* | [**users_me_limits_put**](docs/UsersApi.md#users_me_limits_put) | **PUT** /v2/users/me/limits | Update limits
*UsersApi* | [**users_me_usage_monthly_get**](docs/UsersApi.md#users_me_usage_monthly_get) | **GET** /v2/users/me/usage/monthly | Get monthly usage
*WebhooksWebhookDispatchesApi* | [**webhook_dispatch_get**](docs/WebhooksWebhookDispatchesApi.md#webhook_dispatch_get) | **GET** /v2/webhook-dispatches/{dispatchId} | Get webhook dispatch
*WebhooksWebhookDispatchesApi* | [**webhook_dispatches_get**](docs/WebhooksWebhookDispatchesApi.md#webhook_dispatches_get) | **GET** /v2/webhook-dispatches | Get list of webhook dispatches
*WebhooksWebhooksApi* | [**webhook_delete**](docs/WebhooksWebhooksApi.md#webhook_delete) | **DELETE** /v2/webhooks/{webhookId} | Delete webhook
*WebhooksWebhooksApi* | [**webhook_get**](docs/WebhooksWebhooksApi.md#webhook_get) | **GET** /v2/webhooks/{webhookId} | Get webhook
*WebhooksWebhooksApi* | [**webhook_put**](docs/WebhooksWebhooksApi.md#webhook_put) | **PUT** /v2/webhooks/{webhookId} | Update webhook
*WebhooksWebhooksApi* | [**webhook_test_post**](docs/WebhooksWebhooksApi.md#webhook_test_post) | **POST** /v2/webhooks/{webhookId}/test | Test webhook
*WebhooksWebhooksApi* | [**webhook_webhook_dispatches_get**](docs/WebhooksWebhooksApi.md#webhook_webhook_dispatches_get) | **GET** /v2/webhooks/{webhookId}/dispatches | Get collection
*WebhooksWebhooksApi* | [**webhooks_get**](docs/WebhooksWebhooksApi.md#webhooks_get) | **GET** /v2/webhooks | Get list of webhooks
*WebhooksWebhooksApi* | [**webhooks_post**](docs/WebhooksWebhooksApi.md#webhooks_post) | **POST** /v2/webhooks | Create webhook


## Documentation For Models

 - [AccountLimits](docs/AccountLimits.md)
 - [ActRunsLastDatasetItemsPost400Response](docs/ActRunsLastDatasetItemsPost400Response.md)
 - [ActRunsLastDatasetItemsPostRequest](docs/ActRunsLastDatasetItemsPostRequest.md)
 - [ActValidateInputPost200Response](docs/ActValidateInputPost200Response.md)
 - [Actor](docs/Actor.md)
 - [ActorChargeEvent](docs/ActorChargeEvent.md)
 - [ActorDefinition](docs/ActorDefinition.md)
 - [ActorDefinitionDefaultMemoryMbytes](docs/ActorDefinitionDefaultMemoryMbytes.md)
 - [ActorDefinitionStorages](docs/ActorDefinitionStorages.md)
 - [ActorJobStatus](docs/ActorJobStatus.md)
 - [ActorPermissionLevel](docs/ActorPermissionLevel.md)
 - [ActorResponse](docs/ActorResponse.md)
 - [ActorRunFailedError](docs/ActorRunFailedError.md)
 - [ActorRunPricingInfo](docs/ActorRunPricingInfo.md)
 - [ActorRunTimeoutExceededError](docs/ActorRunTimeoutExceededError.md)
 - [ActorShort](docs/ActorShort.md)
 - [ActorStandby](docs/ActorStandby.md)
 - [ActorStats](docs/ActorStats.md)
 - [ActorStatsPublicActorRunStats30Days](docs/ActorStatsPublicActorRunStats30Days.md)
 - [ActorTaskGet200Response](docs/ActorTaskGet200Response.md)
 - [ActorTaskRunsGet200Response](docs/ActorTaskRunsGet200Response.md)
 - [ActorTaskRunsGet200ResponseData](docs/ActorTaskRunsGet200ResponseData.md)
 - [ActorTaskRunsPost201Response](docs/ActorTaskRunsPost201Response.md)
 - [ActorTaskWebhooksGet200Response](docs/ActorTaskWebhooksGet200Response.md)
 - [ActorTaskWebhooksGet200ResponseData](docs/ActorTaskWebhooksGet200ResponseData.md)
 - [AddRequestResponse](docs/AddRequestResponse.md)
 - [AddedRequest](docs/AddedRequest.md)
 - [BatchAddResponse](docs/BatchAddResponse.md)
 - [BatchAddResult](docs/BatchAddResult.md)
 - [BatchDeleteResponse](docs/BatchDeleteResponse.md)
 - [BatchDeleteResult](docs/BatchDeleteResult.md)
 - [BrowserInfoResponse](docs/BrowserInfoResponse.md)
 - [BrowserInfoResponseHeadersValue](docs/BrowserInfoResponseHeadersValue.md)
 - [Build](docs/Build.md)
 - [BuildActVersion](docs/BuildActVersion.md)
 - [BuildOptions](docs/BuildOptions.md)
 - [BuildResponse](docs/BuildResponse.md)
 - [BuildShort](docs/BuildShort.md)
 - [BuildStats](docs/BuildStats.md)
 - [BuildTag](docs/BuildTag.md)
 - [BuildUsage](docs/BuildUsage.md)
 - [BuildsMeta](docs/BuildsMeta.md)
 - [CallsInner](docs/CallsInner.md)
 - [ChargeRunRequest](docs/ChargeRunRequest.md)
 - [CommonActorPricingInfo](docs/CommonActorPricingInfo.md)
 - [CreateActorRequest](docs/CreateActorRequest.md)
 - [CreateOrUpdateVersionRequest](docs/CreateOrUpdateVersionRequest.md)
 - [CreateTaskRequest](docs/CreateTaskRequest.md)
 - [Current](docs/Current.md)
 - [CurrentPricingInfo](docs/CurrentPricingInfo.md)
 - [DailyServiceUsages](docs/DailyServiceUsages.md)
 - [Dataset](docs/Dataset.md)
 - [DatasetFieldStatistics](docs/DatasetFieldStatistics.md)
 - [DatasetListItem](docs/DatasetListItem.md)
 - [DatasetResponse](docs/DatasetResponse.md)
 - [DatasetSchemaValidationError](docs/DatasetSchemaValidationError.md)
 - [DatasetStatistics](docs/DatasetStatistics.md)
 - [DatasetStatisticsResponse](docs/DatasetStatisticsResponse.md)
 - [DatasetStats](docs/DatasetStats.md)
 - [DecodeAndVerifyData](docs/DecodeAndVerifyData.md)
 - [DecodeAndVerifyRequest](docs/DecodeAndVerifyRequest.md)
 - [DecodeAndVerifyResponse](docs/DecodeAndVerifyResponse.md)
 - [DefaultRunOptions](docs/DefaultRunOptions.md)
 - [DeletedRequest](docs/DeletedRequest.md)
 - [DeletedRequestById](docs/DeletedRequestById.md)
 - [DeletedRequestByUniqueKey](docs/DeletedRequestByUniqueKey.md)
 - [EffectivePlatformFeature](docs/EffectivePlatformFeature.md)
 - [EffectivePlatformFeatures](docs/EffectivePlatformFeatures.md)
 - [EncodeAndSignData](docs/EncodeAndSignData.md)
 - [EncodeAndSignResponse](docs/EncodeAndSignResponse.md)
 - [EnvVar](docs/EnvVar.md)
 - [EnvVarRequest](docs/EnvVarRequest.md)
 - [EnvVarResponse](docs/EnvVarResponse.md)
 - [ErrorDetail](docs/ErrorDetail.md)
 - [ErrorResponse](docs/ErrorResponse.md)
 - [ErrorType](docs/ErrorType.md)
 - [EventData](docs/EventData.md)
 - [ExampleRunInput](docs/ExampleRunInput.md)
 - [ExampleWebhookDispatch](docs/ExampleWebhookDispatch.md)
 - [FlatPricePerMonthActorPricingInfo](docs/FlatPricePerMonthActorPricingInfo.md)
 - [FreeActorPricingInfo](docs/FreeActorPricingInfo.md)
 - [GeneralAccess](docs/GeneralAccess.md)
 - [HeadAndLockResponse](docs/HeadAndLockResponse.md)
 - [HeadRequest](docs/HeadRequest.md)
 - [HeadResponse](docs/HeadResponse.md)
 - [HttpMethod](docs/HttpMethod.md)
 - [InvalidItem](docs/InvalidItem.md)
 - [KeyValueStore](docs/KeyValueStore.md)
 - [KeyValueStoreKey](docs/KeyValueStoreKey.md)
 - [KeyValueStoreResponse](docs/KeyValueStoreResponse.md)
 - [KeyValueStoreStats](docs/KeyValueStoreStats.md)
 - [Limits](docs/Limits.md)
 - [LimitsResponse](docs/LimitsResponse.md)
 - [ListOfActors](docs/ListOfActors.md)
 - [ListOfActorsInStoreResponse](docs/ListOfActorsInStoreResponse.md)
 - [ListOfActorsResponse](docs/ListOfActorsResponse.md)
 - [ListOfBuilds](docs/ListOfBuilds.md)
 - [ListOfBuildsResponse](docs/ListOfBuildsResponse.md)
 - [ListOfDatasets](docs/ListOfDatasets.md)
 - [ListOfDatasetsResponse](docs/ListOfDatasetsResponse.md)
 - [ListOfEnvVars](docs/ListOfEnvVars.md)
 - [ListOfEnvVarsResponse](docs/ListOfEnvVarsResponse.md)
 - [ListOfKeyValueStores](docs/ListOfKeyValueStores.md)
 - [ListOfKeyValueStoresResponse](docs/ListOfKeyValueStoresResponse.md)
 - [ListOfKeys](docs/ListOfKeys.md)
 - [ListOfKeysResponse](docs/ListOfKeysResponse.md)
 - [ListOfRequestQueues](docs/ListOfRequestQueues.md)
 - [ListOfRequestQueuesResponse](docs/ListOfRequestQueuesResponse.md)
 - [ListOfRequests](docs/ListOfRequests.md)
 - [ListOfRequestsResponse](docs/ListOfRequestsResponse.md)
 - [ListOfRuns](docs/ListOfRuns.md)
 - [ListOfRunsResponse](docs/ListOfRunsResponse.md)
 - [ListOfSchedules](docs/ListOfSchedules.md)
 - [ListOfSchedulesResponse](docs/ListOfSchedulesResponse.md)
 - [ListOfStoreActors](docs/ListOfStoreActors.md)
 - [ListOfTasks](docs/ListOfTasks.md)
 - [ListOfTasksResponse](docs/ListOfTasksResponse.md)
 - [ListOfVersions](docs/ListOfVersions.md)
 - [ListOfVersionsResponse](docs/ListOfVersionsResponse.md)
 - [ListOfWebhookDispatches](docs/ListOfWebhookDispatches.md)
 - [ListOfWebhookDispatchesResponse](docs/ListOfWebhookDispatchesResponse.md)
 - [ListOfWebhooks](docs/ListOfWebhooks.md)
 - [ListOfWebhooksResponse](docs/ListOfWebhooksResponse.md)
 - [LockedHeadRequest](docs/LockedHeadRequest.md)
 - [LockedRequestQueueHead](docs/LockedRequestQueueHead.md)
 - [Metamorph](docs/Metamorph.md)
 - [MonthlyUsage](docs/MonthlyUsage.md)
 - [MonthlyUsageResponse](docs/MonthlyUsageResponse.md)
 - [PaginationResponse](docs/PaginationResponse.md)
 - [PayPerEventActorPricingInfo](docs/PayPerEventActorPricingInfo.md)
 - [PayPerEventActorPricingInfoAllOfPricingPerEvent](docs/PayPerEventActorPricingInfoAllOfPricingPerEvent.md)
 - [Plan](docs/Plan.md)
 - [PricePerDatasetItemActorPricingInfo](docs/PricePerDatasetItemActorPricingInfo.md)
 - [PriceTiers](docs/PriceTiers.md)
 - [PrivateUserDataResponse](docs/PrivateUserDataResponse.md)
 - [Profile](docs/Profile.md)
 - [ProlongRequestLockResponse](docs/ProlongRequestLockResponse.md)
 - [Proxy](docs/Proxy.md)
 - [ProxyGroup](docs/ProxyGroup.md)
 - [PublicUserDataResponse](docs/PublicUserDataResponse.md)
 - [PutItemResponseError](docs/PutItemResponseError.md)
 - [Request](docs/Request.md)
 - [RequestBase](docs/RequestBase.md)
 - [RequestBasePayload](docs/RequestBasePayload.md)
 - [RequestDraft](docs/RequestDraft.md)
 - [RequestDraftDelete](docs/RequestDraftDelete.md)
 - [RequestDraftDeleteById](docs/RequestDraftDeleteById.md)
 - [RequestDraftDeleteByUniqueKey](docs/RequestDraftDeleteByUniqueKey.md)
 - [RequestLockInfo](docs/RequestLockInfo.md)
 - [RequestQueue](docs/RequestQueue.md)
 - [RequestQueueHead](docs/RequestQueueHead.md)
 - [RequestQueueResponse](docs/RequestQueueResponse.md)
 - [RequestQueueShort](docs/RequestQueueShort.md)
 - [RequestQueueStats](docs/RequestQueueStats.md)
 - [RequestRegistration](docs/RequestRegistration.md)
 - [RequestResponse](docs/RequestResponse.md)
 - [Run](docs/Run.md)
 - [RunFailedErrorDetail](docs/RunFailedErrorDetail.md)
 - [RunMeta](docs/RunMeta.md)
 - [RunOptions](docs/RunOptions.md)
 - [RunOrigin](docs/RunOrigin.md)
 - [RunResponse](docs/RunResponse.md)
 - [RunShort](docs/RunShort.md)
 - [RunStats](docs/RunStats.md)
 - [RunStorageIds](docs/RunStorageIds.md)
 - [RunStorageIdsDatasets](docs/RunStorageIdsDatasets.md)
 - [RunTimeoutExceededErrorDetail](docs/RunTimeoutExceededErrorDetail.md)
 - [RunUsage](docs/RunUsage.md)
 - [RunUsageUsd](docs/RunUsageUsd.md)
 - [Schedule](docs/Schedule.md)
 - [ScheduleAction](docs/ScheduleAction.md)
 - [ScheduleActionRunActor](docs/ScheduleActionRunActor.md)
 - [ScheduleActionRunActorTask](docs/ScheduleActionRunActorTask.md)
 - [ScheduleActionRunInput](docs/ScheduleActionRunInput.md)
 - [ScheduleActionShort](docs/ScheduleActionShort.md)
 - [ScheduleActionShortRunActor](docs/ScheduleActionShortRunActor.md)
 - [ScheduleActionShortRunActorTask](docs/ScheduleActionShortRunActorTask.md)
 - [ScheduleBase](docs/ScheduleBase.md)
 - [ScheduleCreate](docs/ScheduleCreate.md)
 - [ScheduleCreateAction](docs/ScheduleCreateAction.md)
 - [ScheduleCreateActionRunActor](docs/ScheduleCreateActionRunActor.md)
 - [ScheduleCreateActionRunActorTask](docs/ScheduleCreateActionRunActorTask.md)
 - [ScheduleInvoked](docs/ScheduleInvoked.md)
 - [ScheduleLogResponse](docs/ScheduleLogResponse.md)
 - [ScheduleNotifications](docs/ScheduleNotifications.md)
 - [ScheduleResponse](docs/ScheduleResponse.md)
 - [ScheduleShort](docs/ScheduleShort.md)
 - [SchemaValidationErrorData](docs/SchemaValidationErrorData.md)
 - [SourceCodeFile](docs/SourceCodeFile.md)
 - [SourceCodeFileFormat](docs/SourceCodeFileFormat.md)
 - [SourceCodeFolder](docs/SourceCodeFolder.md)
 - [StorageOwnership](docs/StorageOwnership.md)
 - [StoreListActor](docs/StoreListActor.md)
 - [TaggedBuildInfo](docs/TaggedBuildInfo.md)
 - [TaggedBuildsValue](docs/TaggedBuildsValue.md)
 - [Task](docs/Task.md)
 - [TaskOptions](docs/TaskOptions.md)
 - [TaskResponse](docs/TaskResponse.md)
 - [TaskShort](docs/TaskShort.md)
 - [TaskStats](docs/TaskStats.md)
 - [TestWebhookResponse](docs/TestWebhookResponse.md)
 - [TieredPricingPerDatasetItemEntry](docs/TieredPricingPerDatasetItemEntry.md)
 - [TieredPricingPerEventEntry](docs/TieredPricingPerEventEntry.md)
 - [UnlockRequestsResponse](docs/UnlockRequestsResponse.md)
 - [UnlockRequestsResult](docs/UnlockRequestsResult.md)
 - [UpdateActorRequest](docs/UpdateActorRequest.md)
 - [UpdateDatasetRequest](docs/UpdateDatasetRequest.md)
 - [UpdateLimitsRequest](docs/UpdateLimitsRequest.md)
 - [UpdateRequestQueueRequest](docs/UpdateRequestQueueRequest.md)
 - [UpdateRequestResponse](docs/UpdateRequestResponse.md)
 - [UpdateRunRequest](docs/UpdateRunRequest.md)
 - [UpdateStoreRequest](docs/UpdateStoreRequest.md)
 - [UpdateTaskRequest](docs/UpdateTaskRequest.md)
 - [UsageCycle](docs/UsageCycle.md)
 - [UsageItem](docs/UsageItem.md)
 - [UserPrivateInfo](docs/UserPrivateInfo.md)
 - [UserPublicInfo](docs/UserPublicInfo.md)
 - [ValidationError](docs/ValidationError.md)
 - [Version](docs/Version.md)
 - [VersionResponse](docs/VersionResponse.md)
 - [VersionSourceFilesInner](docs/VersionSourceFilesInner.md)
 - [VersionSourceType](docs/VersionSourceType.md)
 - [Webhook](docs/Webhook.md)
 - [WebhookCondition](docs/WebhookCondition.md)
 - [WebhookCreate](docs/WebhookCreate.md)
 - [WebhookDispatch](docs/WebhookDispatch.md)
 - [WebhookDispatchResponse](docs/WebhookDispatchResponse.md)
 - [WebhookDispatchStatus](docs/WebhookDispatchStatus.md)
 - [WebhookDispatchWebhookSummary](docs/WebhookDispatchWebhookSummary.md)
 - [WebhookEventType](docs/WebhookEventType.md)
 - [WebhookRepresentation](docs/WebhookRepresentation.md)
 - [WebhookResponse](docs/WebhookResponse.md)
 - [WebhookShort](docs/WebhookShort.md)
 - [WebhookStats](docs/WebhookStats.md)
 - [WebhookUpdate](docs/WebhookUpdate.md)


To get access to the crate's generated documentation, use:

```
cargo doc --open
```

## Author



