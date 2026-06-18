/*
 * Apify API
 *
 *  The Apify API (version 2) provides programmatic access to the [Apify platform](https://docs.apify.com). The API is organized around [RESTful](https://en.wikipedia.org/wiki/Representational_state_transfer) HTTP endpoints.  You can download the complete OpenAPI schema of Apify API in the [YAML](http://docs.apify.com/api/openapi.yaml) or [JSON](http://docs.apify.com/api/openapi.json) formats. The source code is also available on [GitHub](https://github.com/apify/apify-docs/tree/master/apify-api/openapi).  All requests and responses (including errors) are encoded in [JSON](http://www.json.org/) format with UTF-8 encoding, with a few exceptions that are explicitly described in the reference.  - To access the API using [Node.js](https://nodejs.org/en/), we recommend the [`apify-client`](https://docs.apify.com/api/client/js) [NPM package](https://www.npmjs.com/package/apify-client). - To access the API using [Python](https://www.python.org/), we recommend the [`apify-client`](https://docs.apify.com/api/client/python) [PyPI package](https://pypi.org/project/apify-client/).  The clients' functions correspond to the API endpoints and have the same parameters. This simplifies development of apps that depend on the Apify platform.  :::note Important Request Details  - `Content-Type` header: For requests with a JSON body, you must include the `Content-Type: application/json` header.  - Method override: You can override the HTTP method using the `method` query parameter. This is useful for clients that can only send `GET` requests. For example, to call a `POST` endpoint, append `?method=POST` to the URL of your `GET` request.  :::  ## Authentication <span id=\"/introduction/authentication\"></span>  **You can find your API token on the [Integrations](https://console.apify.com/settings/integrations) page in the Apify Console.**  To use your token in a request, either:  - Add the token to your request's `Authorization` header as `Bearer <token>`. E.g., `Authorization: Bearer xxxxxxx`. [More info](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization). (Recommended). - Add it as the `token` parameter to your request URL. (Less secure).  Using your token in the request header is more secure than using it as a URL parameter because URLs are often stored in browser history and server logs. This creates a chance for someone unauthorized to access your API token.  **Never share your API token or password with untrusted parties!**  For more information, see our [integrations](https://docs.apify.com/platform/integrations) documentation.  ### Agentic payments  AI agents can authenticate and pay for Actor runs without an Apify account using agentic payments. Instead of an API token, the request carries a payment credential that both authorizes and pays for the call. Apify supports the [x402 protocol](https://docs.apify.com/platform/integrations/x402) (`PAYMENT-SIGNATURE` header) and [Skyfire](https://docs.apify.com/platform/integrations/skyfire) (`skyfire-pay-id` header).  ## Basic usage <span id=\"/introduction/basic-usage\"></span>  To run an Actor, send a POST request to the [Run Actor](#/reference/actors/run-collection/run-actor) endpoint using either the Actor ID code (e.g. `vKg4IjxZbEYTYeW8T`) or its name (e.g. `janedoe~my-actor`):  `https://api.apify.com/v2/actors/[actor_id]/runs`  If the Actor is not runnable anonymously, you will receive a 401 or 403 [response code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status). This means you need to add your [secret API token](https://console.apify.com/account#/integrations) to the request's `Authorization` header ([recommended](#/introduction/authentication)) or as a URL query parameter `?token=[your_token]` (less secure).  Optionally, you can include the query parameters described in the [Run Actor](#/reference/actors/run-collection/run-actor) section to customize your run.  If you're using Node.js, the best way to run an Actor is using the `Apify.call()` method from the [Apify SDK](https://sdk.apify.com/docs/api/apify#apifycallactid-input-options). It runs the Actor using the account you are currently logged into (determined by the [secret API token](https://console.apify.com/account#/integrations)). The result is an [Actor run object](https://sdk.apify.com/docs/typedefs/actor-run) and its output (if any).  A typical workflow is as follows:  1. Run an Actor or task using the [Run Actor](#/reference/actors/run-collection/run-actor) or [Run task](#/reference/actor-tasks/run-collection/run-task) API endpoints. 2. Monitor the Actor run by periodically polling its progress using the [Get run](#/reference/actor-runs/run-object-and-its-storages/get-run) API endpoint. 3. Fetch the results from the [Get items](#/reference/datasets/item-collection/get-items) API endpoint using the `defaultDatasetId`, which you receive in the Run request response. Additional data may be stored in a key-value store. You can fetch them from the [Get record](#/reference/key-value-stores/record/get-record) API endpoint using the `defaultKeyValueStoreId` and the store's `key`.  **Note**: Instead of periodic polling, you can also run your [Actor](#/reference/actors/run-actor-synchronously) or [task](#/reference/actor-tasks/runs-collection/run-task-synchronously) synchronously. This will ensure that the request waits for 300 seconds (5 minutes) for the run to finish and returns its output. If the run takes longer, the request will time out and throw an error.  ## Legacy `/v2/acts/` URL prefix <span id=\"/introduction/legacy-acts-prefix\"></span>  The `/v2/acts/` prefix is deprecated but still fully functional, and  such endpoint routes to the same handler as its `/v2/actors/...` counterpart.  New integrations should use the canonical /v2/actors/ prefix,  but existing clients keep working without changes.  ## Response structure <span id=\"/introduction/response-structure\"></span>  Most API endpoints return a JSON object with the `data` property:  ``` {     \"data\": {         ...     } } ```  However, there are a few explicitly described exceptions, such as [Get dataset items](#/reference/datasets/item-collection/get-items) or Key-value store [Get record](#/reference/key-value-stores/record/get-record) API endpoints, which return data in other formats. In case of an error, the response has the HTTP status code in the range of 4xx or 5xx and the `data` property is replaced with `error`. For example:  ``` {     \"error\": {         \"type\": \"record-not-found\",         \"message\": \"Store was not found.\"     } } ```  See [Errors](#/introduction/errors) for more details.  ## Pagination <span id=\"/introduction/pagination\"></span>  All API endpoints that return a list of records (e.g. [Get list of Actors](#/reference/actors/actor-collection/get-list-of-actors)) enforce pagination in order to limit the size of their responses.  Most of these API endpoints are paginated using the `offset` and `limit` query parameters. The only exception is [Get list of keys](#/reference/key-value-stores/key-collection/get-list-of-keys), which is paginated using the `exclusiveStartKey` query parameter.  **IMPORTANT**: Each API endpoint that supports pagination enforces a certain maximum value for the `limit` parameter, in order to reduce the load on Apify servers. The maximum limit could change in future so you should never rely on a specific value and check the responses of these API endpoints.  ### Using offset <span id=\"/introduction/pagination/using-offset\"></span>  Most API endpoints that return a list of records enable pagination using the following query parameters:  <table>   <tr>     <td><code>limit</code></td>     <td>Limits the response to contain a specific maximum number of items, e.g. <code>limit=20</code>.</td>   </tr>   <tr>     <td><code>offset</code></td>     <td>Skips a number of items from the beginning of the list, e.g. <code>offset=100</code>.</td>   </tr>   <tr>     <td><code>desc</code></td>     <td>     By default, items are sorted in the order in which they were created or added to the list.     This feature is useful when fetching all the items, because it ensures that items     created after the client started the pagination will not be skipped.     If you specify the <code>desc=1</code> parameter, the items will be returned in the reverse order,     i.e. from the newest to the oldest items.     </td>   </tr> </table>  The response of these API endpoints is always a JSON object with the following structure:  ``` {     \"data\": {         \"total\": 2560,         \"offset\": 250,         \"limit\": 1000,         \"count\": 1000,         \"desc\": false,         \"items\": [             { 1st object },             { 2nd object },             ...             { 1000th object }         ]     } } ```  The following table describes the meaning of the response properties:  <table>   <tr>     <th>Property</th>     <th>Description</th>   </tr>   <tr>     <td><code>total</code></td>     <td>The total number of items available in the list.</td>   </tr>   <tr>     <td><code>offset</code></td>     <td>The number of items that were skipped at the start.     This is equal to the <code>offset</code> query parameter if it was provided, otherwise it is <code>0</code>.</td>   </tr>   <tr>     <td><code>limit</code></td>     <td>The maximum number of items that can be returned in the HTTP response.     It equals to the <code>limit</code> query parameter if it was provided or     the maximum limit enforced for the particular API endpoint, whichever is smaller.</td>   </tr>   <tr>     <td><code>count</code></td>     <td>The actual number of items returned in the HTTP response.</td>   </tr>   <tr>     <td><code>desc</code></td>     <td><code>true</code> if data were requested in descending order and <code>false</code> otherwise.</td>   </tr>   <tr>     <td><code>items</code></td>     <td>An array of requested items.</td>   </tr> </table>  ### Using key <span id=\"/introduction/pagination/using-key\"></span>  The records in the [key-value store](https://docs.apify.com/platform/storage/key-value-store) are not ordered based on numerical indexes, but rather by their keys in the UTF-8 binary order. Therefore the [Get list of keys](#/reference/key-value-stores/key-collection/get-list-of-keys) API endpoint only supports pagination using the following query parameters:  <table>   <tr>     <td><code>limit</code></td>     <td>Limits the response to contain a specific maximum number items, e.g. <code>limit=20</code>.</td>   </tr>   <tr>     <td><code>exclusiveStartKey</code></td>     <td>Skips all records with keys up to the given key including the given key,     in the UTF-8 binary order.</td>   </tr> </table>  The response of the API endpoint is always a JSON object with following structure:  ``` {     \"data\": {         \"limit\": 1000,         \"isTruncated\": true,         \"exclusiveStartKey\": \"my-key\",         \"nextExclusiveStartKey\": \"some-other-key\",         \"items\": [             { 1st object },             { 2nd object },             ...             { 1000th object }         ]     } } ```  The following table describes the meaning of the response properties:  <table>   <tr>     <th>Property</th>     <th>Description</th>   </tr>   <tr>     <td><code>limit</code></td>     <td>The maximum number of items that can be returned in the HTTP response.     It equals to the <code>limit</code> query parameter if it was provided or     the maximum limit enforced for the particular endpoint, whichever is smaller.</td>   </tr>   <tr>     <td><code>isTruncated</code></td>     <td><code>true</code> if there are more items left to be queried. Otherwise <code>false</code>.</td>   </tr>   <tr>     <td><code>exclusiveStartKey</code></td>     <td>The last key that was skipped at the start. Is `null` for the first page.</td>   </tr>   <tr>     <td><code>nextExclusiveStartKey</code></td>     <td>The value for the <code>exclusiveStartKey</code> parameter to query the next page of items.</td>   </tr> </table>  ## Errors <span id=\"/introduction/errors\"></span>  The Apify API uses common HTTP status codes: `2xx` range for success, `4xx` range for errors caused by the caller (invalid requests) and `5xx` range for server errors (these are rare). Each error response contains a JSON object defining the `error` property, which is an object with the `type` and `message` properties that contain the error code and a human-readable error description, respectively.  For example:  ``` {     \"error\": {         \"type\": \"record-not-found\",         \"message\": \"Store was not found.\"     } } ```  Here is the table of the most common errors that can occur for many API endpoints:  <table>   <tr>     <th>status</th>     <th>type</th>     <th>message</th>   </tr>   <tr>     <td><code>400</code></td>     <td><code>invalid-request</code></td>     <td>POST data must be a JSON object</td>   </tr>   <tr>     <td><code>400</code></td>     <td><code>invalid-value</code></td>     <td>Invalid value provided: Comments required</td>   </tr>   <tr>     <td><code>400</code></td>     <td><code>invalid-record-key</code></td>     <td>Record key contains invalid character</td>   </tr>   <tr>     <td><code>401</code></td>     <td><code>token-not-provided</code></td>     <td>Authentication token was not provided</td>   </tr>   <tr>     <td><code>404</code></td>     <td><code>record-not-found</code></td>     <td>Store was not found</td>   </tr>   <tr>     <td><code>429</code></td>     <td><code>rate-limit-exceeded</code></td>     <td>You have exceeded the rate limit of ... requests per second</td>   </tr>   <tr>     <td><code>405</code></td>     <td><code>method-not-allowed</code></td>     <td>This API endpoint can only be accessed using the following HTTP methods: OPTIONS, POST</td>   </tr> </table>  ## Rate limiting <span id=\"/introduction/rate-limiting\"></span>  All API endpoints limit the rate of requests in order to prevent overloading of Apify servers by misbehaving clients.  There are two kinds of rate limits - a global rate limit and a per-resource rate limit.  ### Global rate limit <span id=\"/introduction/rate-limiting/global-rate-limit\"></span>  The global rate limit is set to _250 000 requests per minute_. For [authenticated](#/introduction/authentication) requests, it is counted per user, and for unauthenticated requests, it is counted per IP address.  ### Per-resource rate limit <span id=\"/introduction/rate-limiting/per-resource-rate-limit\"></span>  The default per-resource rate limit is _60 requests per second per resource_, which in this context means a single Actor, a single Actor run, a single dataset, single key-value store etc. The default rate limit is applied to every API endpoint except a few select ones, which have higher rate limits. Each API endpoint returns its rate limit in `X-RateLimit-Limit` header.  These endpoints have a rate limit of _200 requests per second per resource_:  * CRUD ([get](#/reference/key-value-stores/record/get-record),   [put](#/reference/key-value-stores/record/put-record),   [delete](#/reference/key-value-stores/record/delete-record))   operations on key-value store records  These endpoints have a rate limit of _400 requests per second per resource_: * [Run Actor](#/reference/actors/run-collection/run-actor) * [Run Actor task asynchronously](#/reference/actor-tasks/runs-collection/run-task-asynchronously) * [Run Actor task synchronously](#/reference/actor-tasks/runs-collection/run-task-synchronously) * [Metamorph Actor run](#/reference/actors/metamorph-run/metamorph-run) * [Push items](#/reference/datasets/item-collection/put-items) to dataset * CRUD   ([add](#/reference/request-queues/request-collection/add-request),   [get](#/reference/request-queues/request-collection/get-request),   [update](#/reference/request-queues/request-collection/update-request),   [delete](#/reference/request-queues/request-collection/delete-request))   operations on requests in request queues  ### Rate limit exceeded errors <span id=\"/introduction/rate-limiting/rate-limit-exceeded-errors\"></span>  If the client is sending too many requests, the API endpoints respond with the HTTP status code `429 Too Many Requests` and the following body:  ``` {     \"error\": {         \"type\": \"rate-limit-exceeded\",         \"message\": \"You have exceeded the rate limit of ... requests per second\"     } } ```  ### Retrying rate-limited requests with exponential backoff <span id=\"/introduction/rate-limiting/retrying-rate-limited-requests-with-exponential-backoff\"></span>  If the client receives the rate limit error, it should wait a certain period of time and then retry the request. If the error happens again, the client should double the wait period and retry the request, and so on. This algorithm is known as _exponential backoff_ and it can be described using the following pseudo-code:  1. Define a variable `DELAY=500` 2. Send the HTTP request to the API endpoint 3. If the response has status code not equal to `429` then you are done. Otherwise:    * Wait for a period of time chosen randomly from the interval `DELAY` to `2*DELAY` milliseconds    * Double the future wait period by setting `DELAY = 2*DELAY`    * Continue with step 2  If all requests sent by the client implement the above steps, the client will automatically use the maximum available bandwidth for its requests.  Note that the Apify API clients [for JavaScript](https://docs.apify.com/api/client/js) and [for Python](https://docs.apify.com/api/client/python) use the exponential backoff algorithm transparently, so that you do not need to worry about it.  ## Referring to resources <span id=\"/introduction/referring-to-resources\"></span>  There are three main ways to refer to a resource you're accessing via API.  - the resource ID (e.g. `iKkPcIgVvwmztduf8`) - `username~resourcename` - when using this access method, you will need to use your API token, and access will only work if you have the correct permissions. - `~resourcename` - for this, you need to use an API token, and the `resourcename` refers to a resource in the API token owner's account. 
 *
 * The version of the OpenAPI document: v2-2026-06-16T064758Z
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;
use serde::{Deserialize, Serialize, de::Error as _};
use crate::{apis::ResponseContent, models};
use super::{Error, configuration, ContentType};


/// struct for typed errors of method [`act_run_abort_post`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunAbortPostError {
    Status400(models::ErrorResponse),
    Status401(models::ErrorResponse),
    Status403(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status405(models::ErrorResponse),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_run_get`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunGetError {
    Status400(models::ErrorResponse),
    Status401(models::ErrorResponse),
    Status403(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status405(models::ErrorResponse),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_run_metamorph_post`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunMetamorphPostError {
    Status400(models::ErrorResponse),
    Status401(models::ErrorResponse),
    Status403(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status405(models::ErrorResponse),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_run_resurrect_post`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunResurrectPostError {
    Status400(models::ErrorResponse),
    Status401(models::ErrorResponse),
    Status403(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status405(models::ErrorResponse),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_run_sync_get`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunSyncGetError {
    Status400(models::ActorRunFailedError),
    Status404(models::ErrorResponse),
    Status408(models::ActorRunTimeoutExceededError),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_run_sync_get_dataset_items_get`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunSyncGetDatasetItemsGetError {
    Status400(models::ActorRunFailedError),
    Status404(models::ErrorResponse),
    Status408(models::ActorRunTimeoutExceededError),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_run_sync_get_dataset_items_post`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunSyncGetDatasetItemsPostError {
    Status400(models::ActorRunFailedError),
    Status402(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status408(models::ActorRunTimeoutExceededError),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_run_sync_post`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunSyncPostError {
    Status400(models::ActorRunFailedError),
    Status402(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status408(models::ActorRunTimeoutExceededError),
    Status415(models::ErrorResponse),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_runs_get`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunsGetError {
    Status400(models::ErrorResponse),
    Status401(models::ErrorResponse),
    Status403(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status405(models::ErrorResponse),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_runs_last_get`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunsLastGetError {
    Status400(models::ErrorResponse),
    Status401(models::ErrorResponse),
    Status403(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status405(models::ErrorResponse),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`act_runs_post`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActRunsPostError {
    Status400(models::ErrorResponse),
    Status401(models::ErrorResponse),
    Status402(models::ErrorResponse),
    Status403(models::ErrorResponse),
    Status404(models::ErrorResponse),
    Status405(models::ErrorResponse),
    Status413(models::ErrorResponse),
    Status415(models::ErrorResponse),
    Status429(models::ErrorResponse),
    UnknownValue(serde_json::Value),
}


/// **[DEPRECATED]** API endpoints related to run of the Actor were moved under new namespace [`actor-runs`](#/reference/actor-runs). Aborts an Actor run and returns an object that contains all the details about the run.  Only runs that are starting or running are aborted. For runs with status `FINISHED`, `FAILED`, `ABORTING` and `TIMED-OUT` this call does nothing. 
#[deprecated]
pub async fn act_run_abort_post(configuration: &configuration::Configuration, actor_id: &str, run_id: &str, gracefully: Option<bool>) -> Result<models::RunResponse, Error<ActRunAbortPostError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_path_run_id = run_id;
    let p_query_gracefully = gracefully;

    let uri_str = format!("{}/v2/actors/{actorId}/runs/{runId}/abort", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id), runId=crate::apis::urlencode(p_path_run_id));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref param_value) = p_query_gracefully {
        req_builder = req_builder.query(&[("gracefully", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::RunResponse`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::RunResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunAbortPostError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// **[DEPRECATED]** API endpoints related to run of the Actor were moved under new namespace [`actor-runs`](#/reference/actor-runs).  Gets an object that contains all the details about a specific run of an Actor.  By passing the optional `waitForFinish` parameter the API endpoint will synchronously wait for the run to finish. This is useful to avoid periodic polling when waiting for Actor run to complete.  This endpoint does not require the authentication token. Instead, calls are authenticated using a hard-to-guess ID of the run. However, if you access the endpoint without the token, certain attributes, such as `usageUsd` and `usageTotalUsd`, will be hidden. 
#[deprecated]
pub async fn act_run_get(configuration: &configuration::Configuration, actor_id: &str, run_id: &str, wait_for_finish: Option<f64>) -> Result<models::RunResponse, Error<ActRunGetError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_path_run_id = run_id;
    let p_query_wait_for_finish = wait_for_finish;

    let uri_str = format!("{}/v2/actors/{actorId}/runs/{runId}", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id), runId=crate::apis::urlencode(p_path_run_id));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref param_value) = p_query_wait_for_finish {
        req_builder = req_builder.query(&[("waitForFinish", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::RunResponse`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::RunResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunGetError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// **[DEPRECATED]** API endpoints related to run of the Actor were moved under new namespace [`actor-runs`](#/reference/actor-runs). Transforms an Actor run into a run of another Actor with a new input.  This is useful if you want to use another Actor to finish the work of your current Actor run, without the need to create a completely new run and waiting for its finish. For the users of your Actors, the metamorph operation is transparent, they will just see your Actor got the work done.  There is a limit on how many times you can metamorph a single run. You can check the limit in [the Actor runtime limits](https://docs.apify.com/platform/limits#actor-limits).  Internally, the system stops the Docker container corresponding to the Actor run and starts a new container using a different Docker image. All the default storages are preserved and the new input is stored under the `INPUT-METAMORPH-1` key in the same default key-value store.  For more information, see the [Actor docs](https://docs.apify.com/platform/actors/development/programming-interface/metamorph). 
#[deprecated]
pub async fn act_run_metamorph_post(configuration: &configuration::Configuration, actor_id: &str, run_id: &str, target_actor_id: &str, build: Option<&str>) -> Result<models::RunResponse, Error<ActRunMetamorphPostError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_path_run_id = run_id;
    let p_query_target_actor_id = target_actor_id;
    let p_query_build = build;

    let uri_str = format!("{}/v2/actors/{actorId}/runs/{runId}/metamorph", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id), runId=crate::apis::urlencode(p_path_run_id));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    req_builder = req_builder.query(&[("targetActorId", &p_query_target_actor_id.to_string())]);
    if let Some(ref param_value) = p_query_build {
        req_builder = req_builder.query(&[("build", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::RunResponse`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::RunResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunMetamorphPostError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// **[DEPRECATED]** API endpoints related to run of the Actor were moved under new namespace [`actor-runs`](#/reference/actor-runs).Resurrects a finished Actor run and returns an object that contains all the details about the resurrected run.  Only finished runs, i.e. runs with status `FINISHED`, `FAILED`, `ABORTED` and `TIMED-OUT` can be resurrected. Run status will be updated to RUNNING and its container will be restarted with the same storages (the same behaviour as when the run gets migrated to the new server).  For more information, see the [Actor docs](https://docs.apify.com/platform/actors/running/runs-and-builds#resurrection-of-finished-run). 
pub async fn act_run_resurrect_post(configuration: &configuration::Configuration, actor_id: &str, run_id: &str, build: Option<&str>, timeout: Option<f64>, memory: Option<f64>, restart_on_error: Option<bool>) -> Result<models::RunResponse, Error<ActRunResurrectPostError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_path_run_id = run_id;
    let p_query_build = build;
    let p_query_timeout = timeout;
    let p_query_memory = memory;
    let p_query_restart_on_error = restart_on_error;

    let uri_str = format!("{}/v2/actors/{actorId}/runs/{runId}/resurrect", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id), runId=crate::apis::urlencode(p_path_run_id));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref param_value) = p_query_build {
        req_builder = req_builder.query(&[("build", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_timeout {
        req_builder = req_builder.query(&[("timeout", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_memory {
        req_builder = req_builder.query(&[("memory", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_restart_on_error {
        req_builder = req_builder.query(&[("restartOnError", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::RunResponse`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::RunResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunResurrectPostError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// Runs a specific Actor and returns its output. The run must finish in 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds otherwise the API endpoint returns a timeout error. The Actor is not passed any input.  Beware that it might be impossible to maintain an idle HTTP connection for a long period of time, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout. If the connection breaks, you will not receive any information about the run and its status.  To run the Actor asynchronously, use the [Run Actor](#/reference/actors/run-collection/run-actor) API endpoint instead. 
pub async fn act_run_sync_get(configuration: &configuration::Configuration, actor_id: &str, output_record_key: Option<&str>, timeout: Option<f64>, memory: Option<f64>, max_items: Option<f64>, max_total_charge_usd: Option<f64>, restart_on_error: Option<bool>, build: Option<&str>, webhooks: Option<&str>) -> Result<serde_json::Value, Error<ActRunSyncGetError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_query_output_record_key = output_record_key;
    let p_query_timeout = timeout;
    let p_query_memory = memory;
    let p_query_max_items = max_items;
    let p_query_max_total_charge_usd = max_total_charge_usd;
    let p_query_restart_on_error = restart_on_error;
    let p_query_build = build;
    let p_query_webhooks = webhooks;

    let uri_str = format!("{}/v2/actors/{actorId}/run-sync", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref param_value) = p_query_output_record_key {
        req_builder = req_builder.query(&[("outputRecordKey", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_timeout {
        req_builder = req_builder.query(&[("timeout", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_memory {
        req_builder = req_builder.query(&[("memory", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_items {
        req_builder = req_builder.query(&[("maxItems", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_total_charge_usd {
        req_builder = req_builder.query(&[("maxTotalChargeUsd", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_restart_on_error {
        req_builder = req_builder.query(&[("restartOnError", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_build {
        req_builder = req_builder.query(&[("build", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_webhooks {
        req_builder = req_builder.query(&[("webhooks", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `serde_json::Value`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `serde_json::Value`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunSyncGetError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// Runs a specific Actor and returns its dataset items. The run must finish in 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds otherwise the API endpoint returns a timeout error. The Actor is not passed any input.  It allows to send all possible options in parameters from [Get Dataset Items](#/reference/datasets/item-collection/get-items) API endpoint.  Beware that it might be impossible to maintain an idle HTTP connection for a long period of time, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout. If the connection breaks, you will not receive any information about the run and its status.  To run the Actor asynchronously, use the [Run Actor](#/reference/actors/run-collection/run-actor) API endpoint instead. 
pub async fn act_run_sync_get_dataset_items_get(configuration: &configuration::Configuration, actor_id: &str, timeout: Option<f64>, memory: Option<f64>, max_items: Option<f64>, max_total_charge_usd: Option<f64>, restart_on_error: Option<bool>, build: Option<&str>, webhooks: Option<&str>, format: Option<&str>, clean: Option<bool>, offset: Option<f64>, limit: Option<f64>, fields: Option<&str>, output_fields: Option<&str>, omit: Option<&str>, unwind: Option<&str>, flatten: Option<&str>, desc: Option<bool>, attachment: Option<bool>, delimiter: Option<&str>, bom: Option<bool>, xml_root: Option<&str>, xml_row: Option<&str>, skip_header_row: Option<bool>, skip_hidden: Option<bool>, skip_empty: Option<bool>, simplified: Option<bool>, view: Option<&str>, skip_failed_pages: Option<bool>, feed_title: Option<&str>, feed_description: Option<&str>) -> Result<Vec<serde_json::Value>, Error<ActRunSyncGetDatasetItemsGetError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_query_timeout = timeout;
    let p_query_memory = memory;
    let p_query_max_items = max_items;
    let p_query_max_total_charge_usd = max_total_charge_usd;
    let p_query_restart_on_error = restart_on_error;
    let p_query_build = build;
    let p_query_webhooks = webhooks;
    let p_query_format = format;
    let p_query_clean = clean;
    let p_query_offset = offset;
    let p_query_limit = limit;
    let p_query_fields = fields;
    let p_query_output_fields = output_fields;
    let p_query_omit = omit;
    let p_query_unwind = unwind;
    let p_query_flatten = flatten;
    let p_query_desc = desc;
    let p_query_attachment = attachment;
    let p_query_delimiter = delimiter;
    let p_query_bom = bom;
    let p_query_xml_root = xml_root;
    let p_query_xml_row = xml_row;
    let p_query_skip_header_row = skip_header_row;
    let p_query_skip_hidden = skip_hidden;
    let p_query_skip_empty = skip_empty;
    let p_query_simplified = simplified;
    let p_query_view = view;
    let p_query_skip_failed_pages = skip_failed_pages;
    let p_query_feed_title = feed_title;
    let p_query_feed_description = feed_description;

    let uri_str = format!("{}/v2/actors/{actorId}/run-sync-get-dataset-items", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref param_value) = p_query_timeout {
        req_builder = req_builder.query(&[("timeout", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_memory {
        req_builder = req_builder.query(&[("memory", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_items {
        req_builder = req_builder.query(&[("maxItems", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_total_charge_usd {
        req_builder = req_builder.query(&[("maxTotalChargeUsd", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_restart_on_error {
        req_builder = req_builder.query(&[("restartOnError", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_build {
        req_builder = req_builder.query(&[("build", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_webhooks {
        req_builder = req_builder.query(&[("webhooks", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_format {
        req_builder = req_builder.query(&[("format", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_clean {
        req_builder = req_builder.query(&[("clean", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_offset {
        req_builder = req_builder.query(&[("offset", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_limit {
        req_builder = req_builder.query(&[("limit", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_fields {
        req_builder = req_builder.query(&[("fields", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_output_fields {
        req_builder = req_builder.query(&[("outputFields", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_omit {
        req_builder = req_builder.query(&[("omit", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_unwind {
        req_builder = req_builder.query(&[("unwind", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_flatten {
        req_builder = req_builder.query(&[("flatten", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_desc {
        req_builder = req_builder.query(&[("desc", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_attachment {
        req_builder = req_builder.query(&[("attachment", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_delimiter {
        req_builder = req_builder.query(&[("delimiter", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_bom {
        req_builder = req_builder.query(&[("bom", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_xml_root {
        req_builder = req_builder.query(&[("xmlRoot", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_xml_row {
        req_builder = req_builder.query(&[("xmlRow", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_skip_header_row {
        req_builder = req_builder.query(&[("skipHeaderRow", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_skip_hidden {
        req_builder = req_builder.query(&[("skipHidden", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_skip_empty {
        req_builder = req_builder.query(&[("skipEmpty", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_simplified {
        req_builder = req_builder.query(&[("simplified", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_view {
        req_builder = req_builder.query(&[("view", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_skip_failed_pages {
        req_builder = req_builder.query(&[("skipFailedPages", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_feed_title {
        req_builder = req_builder.query(&[("feedTitle", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_feed_description {
        req_builder = req_builder.query(&[("feedDescription", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `Vec&lt;serde_json::Value&gt;`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `Vec&lt;serde_json::Value&gt;`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunSyncGetDatasetItemsGetError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// Runs a specific Actor and returns its dataset items.  The POST payload including its `Content-Type` header is passed as `INPUT` to the Actor (usually `application/json`). The HTTP response contains the Actors dataset items, while the format of items depends on specifying dataset items' `format` parameter.  You can send all the same options in parameters as the [Get Dataset Items](#/reference/datasets/item-collection/get-items) API endpoint.  The Actor is started with the default options; you can override them using URL query parameters. If the Actor run exceeds 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds, the HTTP response will return the 408 status code (Request Timeout).  Beware that it might be impossible to maintain an idle HTTP connection for a long period of time, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout. If the connection breaks, you will not receive any information about the run and its status.  To run the Actor asynchronously, use the [Run Actor](#/reference/actors/run-collection/run-actor) API endpoint instead. 
pub async fn act_run_sync_get_dataset_items_post(configuration: &configuration::Configuration, actor_id: &str, body: serde_json::Value, timeout: Option<f64>, memory: Option<f64>, max_items: Option<f64>, max_total_charge_usd: Option<f64>, restart_on_error: Option<bool>, build: Option<&str>, webhooks: Option<&str>, format: Option<&str>, clean: Option<bool>, offset: Option<f64>, limit: Option<f64>, fields: Option<&str>, output_fields: Option<&str>, omit: Option<&str>, unwind: Option<&str>, flatten: Option<&str>, desc: Option<bool>, attachment: Option<bool>, delimiter: Option<&str>, bom: Option<bool>, xml_root: Option<&str>, xml_row: Option<&str>, skip_header_row: Option<bool>, skip_hidden: Option<bool>, skip_empty: Option<bool>, simplified: Option<bool>, view: Option<&str>, skip_failed_pages: Option<bool>, feed_title: Option<&str>, feed_description: Option<&str>) -> Result<Vec<serde_json::Value>, Error<ActRunSyncGetDatasetItemsPostError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_body_body = body;
    let p_query_timeout = timeout;
    let p_query_memory = memory;
    let p_query_max_items = max_items;
    let p_query_max_total_charge_usd = max_total_charge_usd;
    let p_query_restart_on_error = restart_on_error;
    let p_query_build = build;
    let p_query_webhooks = webhooks;
    let p_query_format = format;
    let p_query_clean = clean;
    let p_query_offset = offset;
    let p_query_limit = limit;
    let p_query_fields = fields;
    let p_query_output_fields = output_fields;
    let p_query_omit = omit;
    let p_query_unwind = unwind;
    let p_query_flatten = flatten;
    let p_query_desc = desc;
    let p_query_attachment = attachment;
    let p_query_delimiter = delimiter;
    let p_query_bom = bom;
    let p_query_xml_root = xml_root;
    let p_query_xml_row = xml_row;
    let p_query_skip_header_row = skip_header_row;
    let p_query_skip_hidden = skip_hidden;
    let p_query_skip_empty = skip_empty;
    let p_query_simplified = simplified;
    let p_query_view = view;
    let p_query_skip_failed_pages = skip_failed_pages;
    let p_query_feed_title = feed_title;
    let p_query_feed_description = feed_description;

    let uri_str = format!("{}/v2/actors/{actorId}/run-sync-get-dataset-items", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref param_value) = p_query_timeout {
        req_builder = req_builder.query(&[("timeout", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_memory {
        req_builder = req_builder.query(&[("memory", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_items {
        req_builder = req_builder.query(&[("maxItems", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_total_charge_usd {
        req_builder = req_builder.query(&[("maxTotalChargeUsd", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_restart_on_error {
        req_builder = req_builder.query(&[("restartOnError", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_build {
        req_builder = req_builder.query(&[("build", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_webhooks {
        req_builder = req_builder.query(&[("webhooks", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_format {
        req_builder = req_builder.query(&[("format", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_clean {
        req_builder = req_builder.query(&[("clean", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_offset {
        req_builder = req_builder.query(&[("offset", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_limit {
        req_builder = req_builder.query(&[("limit", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_fields {
        req_builder = req_builder.query(&[("fields", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_output_fields {
        req_builder = req_builder.query(&[("outputFields", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_omit {
        req_builder = req_builder.query(&[("omit", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_unwind {
        req_builder = req_builder.query(&[("unwind", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_flatten {
        req_builder = req_builder.query(&[("flatten", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_desc {
        req_builder = req_builder.query(&[("desc", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_attachment {
        req_builder = req_builder.query(&[("attachment", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_delimiter {
        req_builder = req_builder.query(&[("delimiter", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_bom {
        req_builder = req_builder.query(&[("bom", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_xml_root {
        req_builder = req_builder.query(&[("xmlRoot", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_xml_row {
        req_builder = req_builder.query(&[("xmlRow", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_skip_header_row {
        req_builder = req_builder.query(&[("skipHeaderRow", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_skip_hidden {
        req_builder = req_builder.query(&[("skipHidden", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_skip_empty {
        req_builder = req_builder.query(&[("skipEmpty", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_simplified {
        req_builder = req_builder.query(&[("simplified", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_view {
        req_builder = req_builder.query(&[("view", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_skip_failed_pages {
        req_builder = req_builder.query(&[("skipFailedPages", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_feed_title {
        req_builder = req_builder.query(&[("feedTitle", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_feed_description {
        req_builder = req_builder.query(&[("feedDescription", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    req_builder = req_builder.json(&p_body_body);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `Vec&lt;serde_json::Value&gt;`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `Vec&lt;serde_json::Value&gt;`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunSyncGetDatasetItemsPostError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// Runs a specific Actor and returns its output.  The POST payload including its `Content-Type` header is passed as `INPUT` to the Actor (usually <code>application/json</code>). The HTTP response contains Actors `OUTPUT` record from its default key-value store.  The Actor is started with the default options; you can override them using various URL query parameters. If the Actor run exceeds 300<!-- MAX_ACTOR_JOB_SYNC_WAIT_SECS --> seconds, the HTTP response will have status 408 (Request Timeout).  Beware that it might be impossible to maintain an idle HTTP connection for a long period of time, due to client timeout or network conditions. Make sure your HTTP client is configured to have a long enough connection timeout. If the connection breaks, you will not receive any information about the run and its status.  To run the Actor asynchronously, use the [Run Actor](#/reference/actors/run-collection/run-actor) API endpoint instead. 
pub async fn act_run_sync_post(configuration: &configuration::Configuration, actor_id: &str, body: serde_json::Value, output_record_key: Option<&str>, timeout: Option<f64>, memory: Option<f64>, max_items: Option<f64>, max_total_charge_usd: Option<f64>, restart_on_error: Option<bool>, build: Option<&str>, webhooks: Option<&str>) -> Result<serde_json::Value, Error<ActRunSyncPostError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_body_body = body;
    let p_query_output_record_key = output_record_key;
    let p_query_timeout = timeout;
    let p_query_memory = memory;
    let p_query_max_items = max_items;
    let p_query_max_total_charge_usd = max_total_charge_usd;
    let p_query_restart_on_error = restart_on_error;
    let p_query_build = build;
    let p_query_webhooks = webhooks;

    let uri_str = format!("{}/v2/actors/{actorId}/run-sync", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref param_value) = p_query_output_record_key {
        req_builder = req_builder.query(&[("outputRecordKey", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_timeout {
        req_builder = req_builder.query(&[("timeout", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_memory {
        req_builder = req_builder.query(&[("memory", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_items {
        req_builder = req_builder.query(&[("maxItems", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_total_charge_usd {
        req_builder = req_builder.query(&[("maxTotalChargeUsd", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_restart_on_error {
        req_builder = req_builder.query(&[("restartOnError", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_build {
        req_builder = req_builder.query(&[("build", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_webhooks {
        req_builder = req_builder.query(&[("webhooks", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    req_builder = req_builder.json(&p_body_body);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `serde_json::Value`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `serde_json::Value`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunSyncPostError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// Gets the list of runs of a specific Actor. The response is a list of objects, where each object contains basic information about a single Actor run.  The endpoint supports pagination using the `limit` and `offset` parameters and it will not return more than 1000 array elements.  By default, the records are sorted by the `startedAt` field in ascending order, therefore you can use pagination to incrementally fetch all records while new ones are still being created. To sort the records in descending order, use `desc=1` parameter. You can also filter runs by status ([available statuses](https://docs.apify.com/platform/actors/running/runs-and-builds#lifecycle)). 
pub async fn act_runs_get(configuration: &configuration::Configuration, actor_id: &str, offset: Option<f64>, limit: Option<f64>, desc: Option<bool>, status: Option<Vec<String>>, started_after: Option<chrono::DateTime<chrono::FixedOffset>>, started_before: Option<chrono::DateTime<chrono::FixedOffset>>) -> Result<models::ListOfRunsResponse, Error<ActRunsGetError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_query_offset = offset;
    let p_query_limit = limit;
    let p_query_desc = desc;
    let p_query_status = status;
    let p_query_started_after = started_after;
    let p_query_started_before = started_before;

    let uri_str = format!("{}/v2/actors/{actorId}/runs", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref param_value) = p_query_offset {
        req_builder = req_builder.query(&[("offset", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_limit {
        req_builder = req_builder.query(&[("limit", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_desc {
        req_builder = req_builder.query(&[("desc", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_status {
        req_builder = match "csv" {
            "multi" => req_builder.query(&param_value.into_iter().map(|p| ("status".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => req_builder.query(&[("status", &param_value.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref param_value) = p_query_started_after {
        req_builder = req_builder.query(&[("startedAfter", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_started_before {
        req_builder = req_builder.query(&[("startedBefore", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::ListOfRunsResponse`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::ListOfRunsResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunsGetError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// This is not a single endpoint, but an entire group of endpoints that lets you to retrieve and manage the last run of given Actor or any of its default storages. All the endpoints require an authentication token.  The base path represents the last Actor run object is:  `/v2/actors/{actorId}/runs/last{?token,status}`  Using the `status` query parameter you can ensure to only get a run with a certain status (e.g. `status=SUCCEEDED`). The output of this endpoint and other query parameters are the same as in the [Run object](#/reference/actors/run-object) endpoint.  ##### Convenience endpoints for last Actor run  * [Dataset](/api/v2/last-actor-runs-default-dataset)  * [Key-value store](/api/v2/last-actor-runs-default-key-value-store)  * [Request queue](/api/v2/last-actor-runs-default-request-queue)  * [Log](/api/v2/last-actor-runs-log) 
pub async fn act_runs_last_get(configuration: &configuration::Configuration, actor_id: &str, status: Option<&str>, wait_for_finish: Option<f64>) -> Result<models::RunResponse, Error<ActRunsLastGetError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_query_status = status;
    let p_query_wait_for_finish = wait_for_finish;

    let uri_str = format!("{}/v2/actors/{actorId}/runs/last", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id));
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref param_value) = p_query_status {
        req_builder = req_builder.query(&[("status", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_wait_for_finish {
        req_builder = req_builder.query(&[("waitForFinish", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::RunResponse`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::RunResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunsLastGetError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

/// Runs an Actor and immediately returns without waiting for the run to finish.  The POST payload including its `Content-Type` header is passed as `INPUT` to the Actor (usually `application/json`).  The Actor is started with the default options; you can override them using various URL query parameters.  The response is the Run object as returned by the [Get run](#/reference/actor-runs/run-object-and-its-storages/get-run) API endpoint.  If you want to wait for the run to finish and receive the actual output of the Actor as the response, please use one of the [Run Actor synchronously](#/reference/actors/run-actor-synchronously) API endpoints instead.  To fetch the Actor run results that are typically stored in the default dataset, you'll need to pass the ID received in the `defaultDatasetId` field received in the response JSON to the [Get dataset items](#/reference/datasets/item-collection/get-items) API endpoint. 
pub async fn act_runs_post(configuration: &configuration::Configuration, actor_id: &str, body: serde_json::Value, timeout: Option<f64>, memory: Option<f64>, max_items: Option<f64>, max_total_charge_usd: Option<f64>, restart_on_error: Option<bool>, build: Option<&str>, wait_for_finish: Option<f64>, webhooks: Option<&str>, force_permission_level: Option<&str>) -> Result<models::RunResponse, Error<ActRunsPostError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_actor_id = actor_id;
    let p_body_body = body;
    let p_query_timeout = timeout;
    let p_query_memory = memory;
    let p_query_max_items = max_items;
    let p_query_max_total_charge_usd = max_total_charge_usd;
    let p_query_restart_on_error = restart_on_error;
    let p_query_build = build;
    let p_query_wait_for_finish = wait_for_finish;
    let p_query_webhooks = webhooks;
    let p_query_force_permission_level = force_permission_level;

    let uri_str = format!("{}/v2/actors/{actorId}/runs", configuration.base_path, actorId=crate::apis::urlencode(p_path_actor_id));
    let mut req_builder = configuration.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref param_value) = p_query_timeout {
        req_builder = req_builder.query(&[("timeout", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_memory {
        req_builder = req_builder.query(&[("memory", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_items {
        req_builder = req_builder.query(&[("maxItems", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_max_total_charge_usd {
        req_builder = req_builder.query(&[("maxTotalChargeUsd", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_restart_on_error {
        req_builder = req_builder.query(&[("restartOnError", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_build {
        req_builder = req_builder.query(&[("build", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_wait_for_finish {
        req_builder = req_builder.query(&[("waitForFinish", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_webhooks {
        req_builder = req_builder.query(&[("webhooks", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_query_force_permission_level {
        req_builder = req_builder.query(&[("forcePermissionLevel", &param_value.to_string())]);
    }
    if let Some(ref apikey) = configuration.api_key {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{} {}", prefix, key),
            None => key,
        };
        req_builder = req_builder.query(&[("token", value)]);
    }
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    req_builder = req_builder.json(&p_body_body);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => return Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::RunResponse`"))),
            ContentType::Unsupported(unknown_type) => return Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::RunResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ActRunsPostError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent { status, content, entity }))
    }
}

