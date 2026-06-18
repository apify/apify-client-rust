/*
 * Apify API
 *
 *  The Apify API (version 2) provides programmatic access to the [Apify platform](https://docs.apify.com). The API is organized around [RESTful](https://en.wikipedia.org/wiki/Representational_state_transfer) HTTP endpoints.  You can download the complete OpenAPI schema of Apify API in the [YAML](http://docs.apify.com/api/openapi.yaml) or [JSON](http://docs.apify.com/api/openapi.json) formats. The source code is also available on [GitHub](https://github.com/apify/apify-docs/tree/master/apify-api/openapi).  All requests and responses (including errors) are encoded in [JSON](http://www.json.org/) format with UTF-8 encoding, with a few exceptions that are explicitly described in the reference.  - To access the API using [Node.js](https://nodejs.org/en/), we recommend the [`apify-client`](https://docs.apify.com/api/client/js) [NPM package](https://www.npmjs.com/package/apify-client). - To access the API using [Python](https://www.python.org/), we recommend the [`apify-client`](https://docs.apify.com/api/client/python) [PyPI package](https://pypi.org/project/apify-client/).  The clients' functions correspond to the API endpoints and have the same parameters. This simplifies development of apps that depend on the Apify platform.  :::note Important Request Details  - `Content-Type` header: For requests with a JSON body, you must include the `Content-Type: application/json` header.  - Method override: You can override the HTTP method using the `method` query parameter. This is useful for clients that can only send `GET` requests. For example, to call a `POST` endpoint, append `?method=POST` to the URL of your `GET` request.  :::  ## Authentication <span id=\"/introduction/authentication\"></span>  **You can find your API token on the [Integrations](https://console.apify.com/settings/integrations) page in the Apify Console.**  To use your token in a request, either:  - Add the token to your request's `Authorization` header as `Bearer <token>`. E.g., `Authorization: Bearer xxxxxxx`. [More info](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization). (Recommended). - Add it as the `token` parameter to your request URL. (Less secure).  Using your token in the request header is more secure than using it as a URL parameter because URLs are often stored in browser history and server logs. This creates a chance for someone unauthorized to access your API token.  **Never share your API token or password with untrusted parties!**  For more information, see our [integrations](https://docs.apify.com/platform/integrations) documentation.  ### Agentic payments  AI agents can authenticate and pay for Actor runs without an Apify account using agentic payments. Instead of an API token, the request carries a payment credential that both authorizes and pays for the call. Apify supports the [x402 protocol](https://docs.apify.com/platform/integrations/x402) (`PAYMENT-SIGNATURE` header) and [Skyfire](https://docs.apify.com/platform/integrations/skyfire) (`skyfire-pay-id` header).  ## Basic usage <span id=\"/introduction/basic-usage\"></span>  To run an Actor, send a POST request to the [Run Actor](#/reference/actors/run-collection/run-actor) endpoint using either the Actor ID code (e.g. `vKg4IjxZbEYTYeW8T`) or its name (e.g. `janedoe~my-actor`):  `https://api.apify.com/v2/actors/[actor_id]/runs`  If the Actor is not runnable anonymously, you will receive a 401 or 403 [response code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status). This means you need to add your [secret API token](https://console.apify.com/account#/integrations) to the request's `Authorization` header ([recommended](#/introduction/authentication)) or as a URL query parameter `?token=[your_token]` (less secure).  Optionally, you can include the query parameters described in the [Run Actor](#/reference/actors/run-collection/run-actor) section to customize your run.  If you're using Node.js, the best way to run an Actor is using the `Apify.call()` method from the [Apify SDK](https://sdk.apify.com/docs/api/apify#apifycallactid-input-options). It runs the Actor using the account you are currently logged into (determined by the [secret API token](https://console.apify.com/account#/integrations)). The result is an [Actor run object](https://sdk.apify.com/docs/typedefs/actor-run) and its output (if any).  A typical workflow is as follows:  1. Run an Actor or task using the [Run Actor](#/reference/actors/run-collection/run-actor) or [Run task](#/reference/actor-tasks/run-collection/run-task) API endpoints. 2. Monitor the Actor run by periodically polling its progress using the [Get run](#/reference/actor-runs/run-object-and-its-storages/get-run) API endpoint. 3. Fetch the results from the [Get items](#/reference/datasets/item-collection/get-items) API endpoint using the `defaultDatasetId`, which you receive in the Run request response. Additional data may be stored in a key-value store. You can fetch them from the [Get record](#/reference/key-value-stores/record/get-record) API endpoint using the `defaultKeyValueStoreId` and the store's `key`.  **Note**: Instead of periodic polling, you can also run your [Actor](#/reference/actors/run-actor-synchronously) or [task](#/reference/actor-tasks/runs-collection/run-task-synchronously) synchronously. This will ensure that the request waits for 300 seconds (5 minutes) for the run to finish and returns its output. If the run takes longer, the request will time out and throw an error.  ## Legacy `/v2/acts/` URL prefix <span id=\"/introduction/legacy-acts-prefix\"></span>  The `/v2/acts/` prefix is deprecated but still fully functional, and  such endpoint routes to the same handler as its `/v2/actors/...` counterpart.  New integrations should use the canonical /v2/actors/ prefix,  but existing clients keep working without changes.  ## Response structure <span id=\"/introduction/response-structure\"></span>  Most API endpoints return a JSON object with the `data` property:  ``` {     \"data\": {         ...     } } ```  However, there are a few explicitly described exceptions, such as [Get dataset items](#/reference/datasets/item-collection/get-items) or Key-value store [Get record](#/reference/key-value-stores/record/get-record) API endpoints, which return data in other formats. In case of an error, the response has the HTTP status code in the range of 4xx or 5xx and the `data` property is replaced with `error`. For example:  ``` {     \"error\": {         \"type\": \"record-not-found\",         \"message\": \"Store was not found.\"     } } ```  See [Errors](#/introduction/errors) for more details.  ## Pagination <span id=\"/introduction/pagination\"></span>  All API endpoints that return a list of records (e.g. [Get list of Actors](#/reference/actors/actor-collection/get-list-of-actors)) enforce pagination in order to limit the size of their responses.  Most of these API endpoints are paginated using the `offset` and `limit` query parameters. The only exception is [Get list of keys](#/reference/key-value-stores/key-collection/get-list-of-keys), which is paginated using the `exclusiveStartKey` query parameter.  **IMPORTANT**: Each API endpoint that supports pagination enforces a certain maximum value for the `limit` parameter, in order to reduce the load on Apify servers. The maximum limit could change in future so you should never rely on a specific value and check the responses of these API endpoints.  ### Using offset <span id=\"/introduction/pagination/using-offset\"></span>  Most API endpoints that return a list of records enable pagination using the following query parameters:  <table>   <tr>     <td><code>limit</code></td>     <td>Limits the response to contain a specific maximum number of items, e.g. <code>limit=20</code>.</td>   </tr>   <tr>     <td><code>offset</code></td>     <td>Skips a number of items from the beginning of the list, e.g. <code>offset=100</code>.</td>   </tr>   <tr>     <td><code>desc</code></td>     <td>     By default, items are sorted in the order in which they were created or added to the list.     This feature is useful when fetching all the items, because it ensures that items     created after the client started the pagination will not be skipped.     If you specify the <code>desc=1</code> parameter, the items will be returned in the reverse order,     i.e. from the newest to the oldest items.     </td>   </tr> </table>  The response of these API endpoints is always a JSON object with the following structure:  ``` {     \"data\": {         \"total\": 2560,         \"offset\": 250,         \"limit\": 1000,         \"count\": 1000,         \"desc\": false,         \"items\": [             { 1st object },             { 2nd object },             ...             { 1000th object }         ]     } } ```  The following table describes the meaning of the response properties:  <table>   <tr>     <th>Property</th>     <th>Description</th>   </tr>   <tr>     <td><code>total</code></td>     <td>The total number of items available in the list.</td>   </tr>   <tr>     <td><code>offset</code></td>     <td>The number of items that were skipped at the start.     This is equal to the <code>offset</code> query parameter if it was provided, otherwise it is <code>0</code>.</td>   </tr>   <tr>     <td><code>limit</code></td>     <td>The maximum number of items that can be returned in the HTTP response.     It equals to the <code>limit</code> query parameter if it was provided or     the maximum limit enforced for the particular API endpoint, whichever is smaller.</td>   </tr>   <tr>     <td><code>count</code></td>     <td>The actual number of items returned in the HTTP response.</td>   </tr>   <tr>     <td><code>desc</code></td>     <td><code>true</code> if data were requested in descending order and <code>false</code> otherwise.</td>   </tr>   <tr>     <td><code>items</code></td>     <td>An array of requested items.</td>   </tr> </table>  ### Using key <span id=\"/introduction/pagination/using-key\"></span>  The records in the [key-value store](https://docs.apify.com/platform/storage/key-value-store) are not ordered based on numerical indexes, but rather by their keys in the UTF-8 binary order. Therefore the [Get list of keys](#/reference/key-value-stores/key-collection/get-list-of-keys) API endpoint only supports pagination using the following query parameters:  <table>   <tr>     <td><code>limit</code></td>     <td>Limits the response to contain a specific maximum number items, e.g. <code>limit=20</code>.</td>   </tr>   <tr>     <td><code>exclusiveStartKey</code></td>     <td>Skips all records with keys up to the given key including the given key,     in the UTF-8 binary order.</td>   </tr> </table>  The response of the API endpoint is always a JSON object with following structure:  ``` {     \"data\": {         \"limit\": 1000,         \"isTruncated\": true,         \"exclusiveStartKey\": \"my-key\",         \"nextExclusiveStartKey\": \"some-other-key\",         \"items\": [             { 1st object },             { 2nd object },             ...             { 1000th object }         ]     } } ```  The following table describes the meaning of the response properties:  <table>   <tr>     <th>Property</th>     <th>Description</th>   </tr>   <tr>     <td><code>limit</code></td>     <td>The maximum number of items that can be returned in the HTTP response.     It equals to the <code>limit</code> query parameter if it was provided or     the maximum limit enforced for the particular endpoint, whichever is smaller.</td>   </tr>   <tr>     <td><code>isTruncated</code></td>     <td><code>true</code> if there are more items left to be queried. Otherwise <code>false</code>.</td>   </tr>   <tr>     <td><code>exclusiveStartKey</code></td>     <td>The last key that was skipped at the start. Is `null` for the first page.</td>   </tr>   <tr>     <td><code>nextExclusiveStartKey</code></td>     <td>The value for the <code>exclusiveStartKey</code> parameter to query the next page of items.</td>   </tr> </table>  ## Errors <span id=\"/introduction/errors\"></span>  The Apify API uses common HTTP status codes: `2xx` range for success, `4xx` range for errors caused by the caller (invalid requests) and `5xx` range for server errors (these are rare). Each error response contains a JSON object defining the `error` property, which is an object with the `type` and `message` properties that contain the error code and a human-readable error description, respectively.  For example:  ``` {     \"error\": {         \"type\": \"record-not-found\",         \"message\": \"Store was not found.\"     } } ```  Here is the table of the most common errors that can occur for many API endpoints:  <table>   <tr>     <th>status</th>     <th>type</th>     <th>message</th>   </tr>   <tr>     <td><code>400</code></td>     <td><code>invalid-request</code></td>     <td>POST data must be a JSON object</td>   </tr>   <tr>     <td><code>400</code></td>     <td><code>invalid-value</code></td>     <td>Invalid value provided: Comments required</td>   </tr>   <tr>     <td><code>400</code></td>     <td><code>invalid-record-key</code></td>     <td>Record key contains invalid character</td>   </tr>   <tr>     <td><code>401</code></td>     <td><code>token-not-provided</code></td>     <td>Authentication token was not provided</td>   </tr>   <tr>     <td><code>404</code></td>     <td><code>record-not-found</code></td>     <td>Store was not found</td>   </tr>   <tr>     <td><code>429</code></td>     <td><code>rate-limit-exceeded</code></td>     <td>You have exceeded the rate limit of ... requests per second</td>   </tr>   <tr>     <td><code>405</code></td>     <td><code>method-not-allowed</code></td>     <td>This API endpoint can only be accessed using the following HTTP methods: OPTIONS, POST</td>   </tr> </table>  ## Rate limiting <span id=\"/introduction/rate-limiting\"></span>  All API endpoints limit the rate of requests in order to prevent overloading of Apify servers by misbehaving clients.  There are two kinds of rate limits - a global rate limit and a per-resource rate limit.  ### Global rate limit <span id=\"/introduction/rate-limiting/global-rate-limit\"></span>  The global rate limit is set to _250 000 requests per minute_. For [authenticated](#/introduction/authentication) requests, it is counted per user, and for unauthenticated requests, it is counted per IP address.  ### Per-resource rate limit <span id=\"/introduction/rate-limiting/per-resource-rate-limit\"></span>  The default per-resource rate limit is _60 requests per second per resource_, which in this context means a single Actor, a single Actor run, a single dataset, single key-value store etc. The default rate limit is applied to every API endpoint except a few select ones, which have higher rate limits. Each API endpoint returns its rate limit in `X-RateLimit-Limit` header.  These endpoints have a rate limit of _200 requests per second per resource_:  * CRUD ([get](#/reference/key-value-stores/record/get-record),   [put](#/reference/key-value-stores/record/put-record),   [delete](#/reference/key-value-stores/record/delete-record))   operations on key-value store records  These endpoints have a rate limit of _400 requests per second per resource_: * [Run Actor](#/reference/actors/run-collection/run-actor) * [Run Actor task asynchronously](#/reference/actor-tasks/runs-collection/run-task-asynchronously) * [Run Actor task synchronously](#/reference/actor-tasks/runs-collection/run-task-synchronously) * [Metamorph Actor run](#/reference/actors/metamorph-run/metamorph-run) * [Push items](#/reference/datasets/item-collection/put-items) to dataset * CRUD   ([add](#/reference/request-queues/request-collection/add-request),   [get](#/reference/request-queues/request-collection/get-request),   [update](#/reference/request-queues/request-collection/update-request),   [delete](#/reference/request-queues/request-collection/delete-request))   operations on requests in request queues  ### Rate limit exceeded errors <span id=\"/introduction/rate-limiting/rate-limit-exceeded-errors\"></span>  If the client is sending too many requests, the API endpoints respond with the HTTP status code `429 Too Many Requests` and the following body:  ``` {     \"error\": {         \"type\": \"rate-limit-exceeded\",         \"message\": \"You have exceeded the rate limit of ... requests per second\"     } } ```  ### Retrying rate-limited requests with exponential backoff <span id=\"/introduction/rate-limiting/retrying-rate-limited-requests-with-exponential-backoff\"></span>  If the client receives the rate limit error, it should wait a certain period of time and then retry the request. If the error happens again, the client should double the wait period and retry the request, and so on. This algorithm is known as _exponential backoff_ and it can be described using the following pseudo-code:  1. Define a variable `DELAY=500` 2. Send the HTTP request to the API endpoint 3. If the response has status code not equal to `429` then you are done. Otherwise:    * Wait for a period of time chosen randomly from the interval `DELAY` to `2*DELAY` milliseconds    * Double the future wait period by setting `DELAY = 2*DELAY`    * Continue with step 2  If all requests sent by the client implement the above steps, the client will automatically use the maximum available bandwidth for its requests.  Note that the Apify API clients [for JavaScript](https://docs.apify.com/api/client/js) and [for Python](https://docs.apify.com/api/client/python) use the exponential backoff algorithm transparently, so that you do not need to worry about it.  ## Referring to resources <span id=\"/introduction/referring-to-resources\"></span>  There are three main ways to refer to a resource you're accessing via API.  - the resource ID (e.g. `iKkPcIgVvwmztduf8`) - `username~resourcename` - when using this access method, you will need to use your API token, and access will only work if you have the correct permissions. - `~resourcename` - for this, you need to use an API token, and the `resourcename` refers to a resource in the API token owner's account. 
 *
 * The version of the OpenAPI document: v2-2026-06-16T064758Z
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ErrorType : Machine-processable error type identifier.
/// Machine-processable error type identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    #[serde(rename = "3d-secure-auth-failed")]
    Variant3dSecureAuthFailed,
    #[serde(rename = "access-right-already-exists")]
    AccessRightAlreadyExists,
    #[serde(rename = "action-not-found")]
    ActionNotFound,
    #[serde(rename = "actor-already-rented")]
    ActorAlreadyRented,
    #[serde(rename = "actor-can-not-be-rented")]
    ActorCanNotBeRented,
    #[serde(rename = "actor-disabled")]
    ActorDisabled,
    #[serde(rename = "actor-is-not-rented")]
    ActorIsNotRented,
    #[serde(rename = "actor-memory-limit-exceeded")]
    ActorMemoryLimitExceeded,
    #[serde(rename = "actor-name-exists-new-owner")]
    ActorNameExistsNewOwner,
    #[serde(rename = "actor-name-not-unique")]
    ActorNameNotUnique,
    #[serde(rename = "actor-not-found")]
    ActorNotFound,
    #[serde(rename = "actor-not-github-actor")]
    ActorNotGithubActor,
    #[serde(rename = "actor-not-public")]
    ActorNotPublic,
    #[serde(rename = "actor-permission-level-not-supported-for-agentic-payments")]
    ActorPermissionLevelNotSupportedForAgenticPayments,
    #[serde(rename = "actor-review-already-exists")]
    ActorReviewAlreadyExists,
    #[serde(rename = "actor-run-failed")]
    ActorRunFailed,
    #[serde(rename = "actor-standby-not-supported-for-agentic-payments")]
    ActorStandbyNotSupportedForAgenticPayments,
    #[serde(rename = "actor-task-name-not-unique")]
    ActorTaskNameNotUnique,
    #[serde(rename = "agentic-payment-info-retrieval-error")]
    AgenticPaymentInfoRetrievalError,
    #[serde(rename = "agentic-payment-information-missing")]
    AgenticPaymentInformationMissing,
    #[serde(rename = "agentic-payment-insufficient-amount")]
    AgenticPaymentInsufficientAmount,
    #[serde(rename = "agentic-payment-provider-internal-error")]
    AgenticPaymentProviderInternalError,
    #[serde(rename = "agentic-payment-provider-unauthorized")]
    AgenticPaymentProviderUnauthorized,
    #[serde(rename = "airtable-webhook-deprecated")]
    AirtableWebhookDeprecated,
    #[serde(rename = "already-subscribed-to-paid-actor")]
    AlreadySubscribedToPaidActor,
    #[serde(rename = "apify-plan-required-to-use-paid-actor")]
    ApifyPlanRequiredToUsePaidActor,
    #[serde(rename = "apify-signup-not-allowed")]
    ApifySignupNotAllowed,
    #[serde(rename = "auth-method-not-supported")]
    AuthMethodNotSupported,
    #[serde(rename = "authorization-server-not-found")]
    AuthorizationServerNotFound,
    #[serde(rename = "auto-issue-date-invalid")]
    AutoIssueDateInvalid,
    #[serde(rename = "background-check-required")]
    BackgroundCheckRequired,
    #[serde(rename = "billing-system-error")]
    BillingSystemError,
    #[serde(rename = "black-friday-plan-expired")]
    BlackFridayPlanExpired,
    #[serde(rename = "braintree-error")]
    BraintreeError,
    #[serde(rename = "braintree-not-linked")]
    BraintreeNotLinked,
    #[serde(rename = "braintree-operation-timed-out")]
    BraintreeOperationTimedOut,
    #[serde(rename = "braintree-unsupported-currency")]
    BraintreeUnsupportedCurrency,
    #[serde(rename = "build-not-found")]
    BuildNotFound,
    #[serde(rename = "build-outdated")]
    BuildOutdated,
    #[serde(rename = "cannot-add-apify-events-to-ppe-actor")]
    CannotAddApifyEventsToPpeActor,
    #[serde(rename = "cannot-add-multiple-pricing-infos")]
    CannotAddMultiplePricingInfos,
    #[serde(rename = "cannot-add-pricing-info-that-alters-past")]
    CannotAddPricingInfoThatAltersPast,
    #[serde(rename = "cannot-add-second-future-pricing-info")]
    CannotAddSecondFuturePricingInfo,
    #[serde(rename = "cannot-build-actor-from-webhook")]
    CannotBuildActorFromWebhook,
    #[serde(rename = "cannot-change-billing-interval")]
    CannotChangeBillingInterval,
    #[serde(rename = "cannot-change-owner")]
    CannotChangeOwner,
    #[serde(rename = "cannot-charge-apify-event")]
    CannotChargeApifyEvent,
    #[serde(rename = "cannot-charge-non-pay-per-event-actor")]
    CannotChargeNonPayPerEventActor,
    #[serde(rename = "cannot-comment-as-other-user")]
    CannotCommentAsOtherUser,
    #[serde(rename = "cannot-copy-actor-task")]
    CannotCopyActorTask,
    #[serde(rename = "cannot-create-payout")]
    CannotCreatePayout,
    #[serde(rename = "cannot-create-public-actor")]
    CannotCreatePublicActor,
    #[serde(rename = "cannot-create-tax-transaction")]
    CannotCreateTaxTransaction,
    #[serde(rename = "cannot-delete-critical-actor")]
    CannotDeleteCriticalActor,
    #[serde(rename = "cannot-delete-invoice")]
    CannotDeleteInvoice,
    #[serde(rename = "cannot-delete-paid-actor")]
    CannotDeletePaidActor,
    #[serde(rename = "cannot-disable-one-time-event-for-apify-start-event")]
    CannotDisableOneTimeEventForApifyStartEvent,
    #[serde(rename = "cannot-disable-organization-with-enabled-members")]
    CannotDisableOrganizationWithEnabledMembers,
    #[serde(rename = "cannot-disable-user-with-subscription")]
    CannotDisableUserWithSubscription,
    #[serde(rename = "cannot-link-oauth-to-unverified-email")]
    CannotLinkOauthToUnverifiedEmail,
    #[serde(rename = "cannot-metamorph-to-pay-per-result-actor")]
    CannotMetamorphToPayPerResultActor,
    #[serde(rename = "cannot-modify-actor-pricing-too-frequently")]
    CannotModifyActorPricingTooFrequently,
    #[serde(rename = "cannot-modify-actor-pricing-with-immediate-effect")]
    CannotModifyActorPricingWithImmediateEffect,
    #[serde(rename = "cannot-override-paid-actor-trial")]
    CannotOverridePaidActorTrial,
    #[serde(rename = "cannot-permanently-delete-subscription")]
    CannotPermanentlyDeleteSubscription,
    #[serde(rename = "cannot-publish-actor")]
    CannotPublishActor,
    #[serde(rename = "cannot-reduce-last-full-token")]
    CannotReduceLastFullToken,
    #[serde(rename = "cannot-reimburse-more-than-original-charge")]
    CannotReimburseMoreThanOriginalCharge,
    #[serde(rename = "cannot-reimburse-non-rental-charge")]
    CannotReimburseNonRentalCharge,
    #[serde(rename = "cannot-remove-own-actor-from-recently-used")]
    CannotRemoveOwnActorFromRecentlyUsed,
    #[serde(rename = "cannot-remove-payment-method")]
    CannotRemovePaymentMethod,
    #[serde(rename = "cannot-remove-pricing-info")]
    CannotRemovePricingInfo,
    #[serde(rename = "cannot-remove-running-run")]
    CannotRemoveRunningRun,
    #[serde(rename = "cannot-remove-user-with-public-actors")]
    CannotRemoveUserWithPublicActors,
    #[serde(rename = "cannot-remove-user-with-subscription")]
    CannotRemoveUserWithSubscription,
    #[serde(rename = "cannot-remove-user-with-unpaid-invoice")]
    CannotRemoveUserWithUnpaidInvoice,
    #[serde(rename = "cannot-rename-env-var")]
    CannotRenameEnvVar,
    #[serde(rename = "cannot-rent-paid-actor")]
    CannotRentPaidActor,
    #[serde(rename = "cannot-review-own-actor")]
    CannotReviewOwnActor,
    #[serde(rename = "cannot-set-access-rights-for-owner")]
    CannotSetAccessRightsForOwner,
    #[serde(rename = "cannot-set-is-status-message-terminal")]
    CannotSetIsStatusMessageTerminal,
    #[serde(rename = "cannot-unpublish-critical-actor")]
    CannotUnpublishCriticalActor,
    #[serde(rename = "cannot-unpublish-paid-actor")]
    CannotUnpublishPaidActor,
    #[serde(rename = "cannot-unpublish-profile")]
    CannotUnpublishProfile,
    #[serde(rename = "cannot-update-invoice-field")]
    CannotUpdateInvoiceField,
    #[serde(rename = "concurrent-runs-limit-exceeded")]
    ConcurrentRunsLimitExceeded,
    #[serde(rename = "concurrent-update-detected")]
    ConcurrentUpdateDetected,
    #[serde(rename = "conference-token-not-found")]
    ConferenceTokenNotFound,
    #[serde(rename = "content-encoding-forbidden-for-html")]
    ContentEncodingForbiddenForHtml,
    #[serde(rename = "coupon-already-redeemed")]
    CouponAlreadyRedeemed,
    #[serde(rename = "coupon-expired")]
    CouponExpired,
    #[serde(rename = "coupon-for-new-customers")]
    CouponForNewCustomers,
    #[serde(rename = "coupon-for-subscribed-users")]
    CouponForSubscribedUsers,
    #[serde(rename = "coupon-limits-are-in-conflict-with-current-limits")]
    CouponLimitsAreInConflictWithCurrentLimits,
    #[serde(rename = "coupon-max-number-of-redemptions-reached")]
    CouponMaxNumberOfRedemptionsReached,
    #[serde(rename = "coupon-not-found")]
    CouponNotFound,
    #[serde(rename = "coupon-not-unique")]
    CouponNotUnique,
    #[serde(rename = "coupons-disabled")]
    CouponsDisabled,
    #[serde(rename = "create-github-issue-not-allowed")]
    CreateGithubIssueNotAllowed,
    #[serde(rename = "creator-plan-not-available")]
    CreatorPlanNotAvailable,
    #[serde(rename = "cron-expression-invalid")]
    CronExpressionInvalid,
    #[serde(rename = "daily-ai-token-limit-exceeded")]
    DailyAiTokenLimitExceeded,
    #[serde(rename = "daily-publication-limit-exceeded")]
    DailyPublicationLimitExceeded,
    #[serde(rename = "dataset-does-not-have-fields-schema")]
    DatasetDoesNotHaveFieldsSchema,
    #[serde(rename = "dataset-does-not-have-schema")]
    DatasetDoesNotHaveSchema,
    #[serde(rename = "dataset-locked")]
    DatasetLocked,
    #[serde(rename = "dataset-schema-invalid")]
    DatasetSchemaInvalid,
    #[serde(rename = "dcr-not-supported")]
    DcrNotSupported,
    #[serde(rename = "default-dataset-not-found")]
    DefaultDatasetNotFound,
    #[serde(rename = "deleting-default-build")]
    DeletingDefaultBuild,
    #[serde(rename = "deleting-unfinished-build")]
    DeletingUnfinishedBuild,
    #[serde(rename = "email-already-taken")]
    EmailAlreadyTaken,
    #[serde(rename = "email-already-taken-removed-user")]
    EmailAlreadyTakenRemovedUser,
    #[serde(rename = "email-domain-not-allowed-for-coupon")]
    EmailDomainNotAllowedForCoupon,
    #[serde(rename = "email-invalid")]
    EmailInvalid,
    #[serde(rename = "email-not-allowed")]
    EmailNotAllowed,
    #[serde(rename = "email-not-valid")]
    EmailNotValid,
    #[serde(rename = "email-update-too-soon")]
    EmailUpdateTooSoon,
    #[serde(rename = "elevated-permissions-needed")]
    ElevatedPermissionsNeeded,
    #[serde(rename = "env-var-already-exists")]
    EnvVarAlreadyExists,
    #[serde(rename = "exchange-rate-fetch-failed")]
    ExchangeRateFetchFailed,
    #[serde(rename = "expired-conference-token")]
    ExpiredConferenceToken,
    #[serde(rename = "failed-to-charge-user")]
    FailedToChargeUser,
    #[serde(rename = "final-invoice-negative")]
    FinalInvoiceNegative,
    #[serde(rename = "full-permission-actor-not-approved")]
    FullPermissionActorNotApproved,
    #[serde(rename = "github-branch-empty")]
    GithubBranchEmpty,
    #[serde(rename = "github-issue-already-exists")]
    GithubIssueAlreadyExists,
    #[serde(rename = "github-public-key-not-found")]
    GithubPublicKeyNotFound,
    #[serde(rename = "github-repository-not-found")]
    GithubRepositoryNotFound,
    #[serde(rename = "github-signature-does-not-match-payload")]
    GithubSignatureDoesNotMatchPayload,
    #[serde(rename = "github-user-not-authorized-for-issues")]
    GithubUserNotAuthorizedForIssues,
    #[serde(rename = "gmail-not-allowed")]
    GmailNotAllowed,
    #[serde(rename = "id-does-not-match")]
    IdDoesNotMatch,
    #[serde(rename = "incompatible-billing-interval")]
    IncompatibleBillingInterval,
    #[serde(rename = "incomplete-payout-billing-info")]
    IncompletePayoutBillingInfo,
    #[serde(rename = "inconsistent-currencies")]
    InconsistentCurrencies,
    #[serde(rename = "incorrect-pricing-modifier-prefix")]
    IncorrectPricingModifierPrefix,
    #[serde(rename = "input-json-invalid-characters")]
    InputJsonInvalidCharacters,
    #[serde(rename = "input-json-not-object")]
    InputJsonNotObject,
    #[serde(rename = "input-json-too-long")]
    InputJsonTooLong,
    #[serde(rename = "input-update-collision")]
    InputUpdateCollision,
    #[serde(rename = "insufficient-permissions")]
    InsufficientPermissions,
    #[serde(rename = "insufficient-permissions-to-change-field")]
    InsufficientPermissionsToChangeField,
    #[serde(rename = "insufficient-security-measures")]
    InsufficientSecurityMeasures,
    #[serde(rename = "insufficient-tax-country-evidence")]
    InsufficientTaxCountryEvidence,
    #[serde(rename = "integration-auth-error")]
    IntegrationAuthError,
    #[serde(rename = "internal-server-error")]
    InternalServerError,
    #[serde(rename = "invalid-billing-info")]
    InvalidBillingInfo,
    #[serde(rename = "invalid-billing-period-for-payout")]
    InvalidBillingPeriodForPayout,
    #[serde(rename = "invalid-build")]
    InvalidBuild,
    #[serde(rename = "invalid-client-key")]
    InvalidClientKey,
    #[serde(rename = "invalid-collection")]
    InvalidCollection,
    #[serde(rename = "invalid-conference-login-password")]
    InvalidConferenceLoginPassword,
    #[serde(rename = "invalid-content-type-header")]
    InvalidContentTypeHeader,
    #[serde(rename = "invalid-credentials")]
    InvalidCredentials,
    #[serde(rename = "invalid-git-auth-token")]
    InvalidGitAuthToken,
    #[serde(rename = "invalid-github-issue-url")]
    InvalidGithubIssueUrl,
    #[serde(rename = "invalid-header")]
    InvalidHeader,
    #[serde(rename = "invalid-id")]
    InvalidId,
    #[serde(rename = "invalid-idempotency-key")]
    InvalidIdempotencyKey,
    #[serde(rename = "invalid-input")]
    InvalidInput,
    #[serde(rename = "invalid-input-schema")]
    InvalidInputSchema,
    #[serde(rename = "invalid-invoice")]
    InvalidInvoice,
    #[serde(rename = "invalid-invoice-type")]
    InvalidInvoiceType,
    #[serde(rename = "invalid-issue-date")]
    InvalidIssueDate,
    #[serde(rename = "invalid-label-params")]
    InvalidLabelParams,
    #[serde(rename = "invalid-main-account-user-id")]
    InvalidMainAccountUserId,
    #[serde(rename = "invalid-oauth-app")]
    InvalidOauthApp,
    #[serde(rename = "invalid-oauth-scope")]
    InvalidOauthScope,
    #[serde(rename = "invalid-one-time-invoice")]
    InvalidOneTimeInvoice,
    #[serde(rename = "invalid-parameter")]
    InvalidParameter,
    #[serde(rename = "invalid-payout-status")]
    InvalidPayoutStatus,
    #[serde(rename = "invalid-picture-url")]
    InvalidPictureUrl,
    #[serde(rename = "invalid-record-key")]
    InvalidRecordKey,
    #[serde(rename = "invalid-request")]
    InvalidRequest,
    #[serde(rename = "invalid-resource-type")]
    InvalidResourceType,
    #[serde(rename = "invalid-signature")]
    InvalidSignature,
    #[serde(rename = "invalid-subscription-plan")]
    InvalidSubscriptionPlan,
    #[serde(rename = "invalid-tax-number")]
    InvalidTaxNumber,
    #[serde(rename = "invalid-tax-number-format")]
    InvalidTaxNumberFormat,
    #[serde(rename = "invalid-token")]
    InvalidToken,
    #[serde(rename = "invalid-token-type")]
    InvalidTokenType,
    #[serde(rename = "invalid-two-factor-code")]
    InvalidTwoFactorCode,
    #[serde(rename = "invalid-two-factor-code-or-recovery-code")]
    InvalidTwoFactorCodeOrRecoveryCode,
    #[serde(rename = "invalid-two-factor-recovery-code")]
    InvalidTwoFactorRecoveryCode,
    #[serde(rename = "invalid-username")]
    InvalidUsername,
    #[serde(rename = "invalid-value")]
    InvalidValue,
    #[serde(rename = "invitation-invalid-resource-type")]
    InvitationInvalidResourceType,
    #[serde(rename = "invitation-no-longer-valid")]
    InvitationNoLongerValid,
    #[serde(rename = "invoice-canceled")]
    InvoiceCanceled,
    #[serde(rename = "invoice-cannot-be-refunded-due-to-too-high-amount")]
    InvoiceCannotBeRefundedDueToTooHighAmount,
    #[serde(rename = "invoice-incomplete")]
    InvoiceIncomplete,
    #[serde(rename = "invoice-is-draft")]
    InvoiceIsDraft,
    #[serde(rename = "invoice-locked")]
    InvoiceLocked,
    #[serde(rename = "invoice-must-be-buffer")]
    InvoiceMustBeBuffer,
    #[serde(rename = "invoice-not-canceled")]
    InvoiceNotCanceled,
    #[serde(rename = "invoice-not-draft")]
    InvoiceNotDraft,
    #[serde(rename = "invoice-not-found")]
    InvoiceNotFound,
    #[serde(rename = "invoice-outdated")]
    InvoiceOutdated,
    #[serde(rename = "invoice-paid-already")]
    InvoicePaidAlready,
    #[serde(rename = "issue-already-connected-to-github")]
    IssueAlreadyConnectedToGithub,
    #[serde(rename = "issue-not-found")]
    IssueNotFound,
    #[serde(rename = "issues-bad-request")]
    IssuesBadRequest,
    #[serde(rename = "issuer-not-registered")]
    IssuerNotRegistered,
    #[serde(rename = "job-finished")]
    JobFinished,
    #[serde(rename = "label-already-linked")]
    LabelAlreadyLinked,
    #[serde(rename = "last-api-token")]
    LastApiToken,
    #[serde(rename = "limit-reached")]
    LimitReached,
    #[serde(rename = "max-items-must-be-greater-than-zero")]
    MaxItemsMustBeGreaterThanZero,
    #[serde(rename = "max-metamorphs-exceeded")]
    MaxMetamorphsExceeded,
    #[serde(rename = "max-total-charge-usd-below-minimum")]
    MaxTotalChargeUsdBelowMinimum,
    #[serde(rename = "max-total-charge-usd-must-be-greater-than-zero")]
    MaxTotalChargeUsdMustBeGreaterThanZero,
    #[serde(rename = "method-not-allowed")]
    MethodNotAllowed,
    #[serde(rename = "migration-disabled")]
    MigrationDisabled,
    #[serde(rename = "missing-actor-rights")]
    MissingActorRights,
    #[serde(rename = "missing-api-token")]
    MissingApiToken,
    #[serde(rename = "missing-billing-info")]
    MissingBillingInfo,
    #[serde(rename = "missing-line-items")]
    MissingLineItems,
    #[serde(rename = "missing-payment-date")]
    MissingPaymentDate,
    #[serde(rename = "missing-payout-billing-info")]
    MissingPayoutBillingInfo,
    #[serde(rename = "missing-proxy-password")]
    MissingProxyPassword,
    #[serde(rename = "missing-reporting-fields")]
    MissingReportingFields,
    #[serde(rename = "missing-resource-name")]
    MissingResourceName,
    #[serde(rename = "missing-settings")]
    MissingSettings,
    #[serde(rename = "missing-username")]
    MissingUsername,
    #[serde(rename = "monthly-usage-limit-too-low")]
    MonthlyUsageLimitTooLow,
    #[serde(rename = "more-than-one-update-not-allowed")]
    MoreThanOneUpdateNotAllowed,
    #[serde(rename = "multiple-records-found")]
    MultipleRecordsFound,
    #[serde(rename = "must-be-admin")]
    MustBeAdmin,
    #[serde(rename = "name-not-unique")]
    NameNotUnique,
    #[serde(rename = "next-runtime-computation-failed")]
    NextRuntimeComputationFailed,
    #[serde(rename = "no-columns-in-exported-dataset")]
    NoColumnsInExportedDataset,
    #[serde(rename = "no-payment-attempt-for-refund-found")]
    NoPaymentAttemptForRefundFound,
    #[serde(rename = "no-payment-method-available")]
    NoPaymentMethodAvailable,
    #[serde(rename = "no-team-account-seats-available")]
    NoTeamAccountSeatsAvailable,
    #[serde(rename = "non-temporary-email")]
    NonTemporaryEmail,
    #[serde(rename = "not-enough-usage-to-run-paid-actor")]
    NotEnoughUsageToRunPaidActor,
    #[serde(rename = "not-implemented")]
    NotImplemented,
    #[serde(rename = "not-supported-currencies")]
    NotSupportedCurrencies,
    #[serde(rename = "o-auth-service-already-connected")]
    OAuthServiceAlreadyConnected,
    #[serde(rename = "o-auth-service-not-connected")]
    OAuthServiceNotConnected,
    #[serde(rename = "oauth-resource-access-failed")]
    OauthResourceAccessFailed,
    #[serde(rename = "one-time-invoice-already-marked-paid")]
    OneTimeInvoiceAlreadyMarkedPaid,
    #[serde(rename = "only-drafts-can-be-deleted")]
    OnlyDraftsCanBeDeleted,
    #[serde(rename = "operation-canceled")]
    OperationCanceled,
    #[serde(rename = "operation-not-allowed")]
    OperationNotAllowed,
    #[serde(rename = "operation-timed-out")]
    OperationTimedOut,
    #[serde(rename = "organization-cannot-own-itself")]
    OrganizationCannotOwnItself,
    #[serde(rename = "organization-role-not-found")]
    OrganizationRoleNotFound,
    #[serde(rename = "overlapping-payout-billing-periods")]
    OverlappingPayoutBillingPeriods,
    #[serde(rename = "own-token-required")]
    OwnTokenRequired,
    #[serde(rename = "page-not-found")]
    PageNotFound,
    #[serde(rename = "param-not-one-of")]
    ParamNotOneOf,
    #[serde(rename = "parameter-required")]
    ParameterRequired,
    #[serde(rename = "parameters-mismatched")]
    ParametersMismatched,
    #[serde(rename = "password-reset-email-already-sent")]
    PasswordResetEmailAlreadySent,
    #[serde(rename = "password-reset-token-expired")]
    PasswordResetTokenExpired,
    #[serde(rename = "pay-as-you-go-without-monthly-interval")]
    PayAsYouGoWithoutMonthlyInterval,
    #[serde(rename = "payment-attempt-status-message-required")]
    PaymentAttemptStatusMessageRequired,
    #[serde(rename = "payout-already-paid")]
    PayoutAlreadyPaid,
    #[serde(rename = "payout-canceled")]
    PayoutCanceled,
    #[serde(rename = "payout-invalid-state")]
    PayoutInvalidState,
    #[serde(rename = "payout-must-be-approved-to-be-marked-paid")]
    PayoutMustBeApprovedToBeMarkedPaid,
    #[serde(rename = "payout-not-found")]
    PayoutNotFound,
    #[serde(rename = "payout-number-already-exists")]
    PayoutNumberAlreadyExists,
    #[serde(rename = "phone-number-invalid")]
    PhoneNumberInvalid,
    #[serde(rename = "phone-number-landline")]
    PhoneNumberLandline,
    #[serde(rename = "phone-number-opted-out")]
    PhoneNumberOptedOut,
    #[serde(rename = "phone-verification-disabled")]
    PhoneVerificationDisabled,
    #[serde(rename = "platform-feature-disabled")]
    PlatformFeatureDisabled,
    #[serde(rename = "price-overrides-validation-failed")]
    PriceOverridesValidationFailed,
    #[serde(rename = "pricing-model-not-supported")]
    PricingModelNotSupported,
    #[serde(rename = "promotional-plan-not-available")]
    PromotionalPlanNotAvailable,
    #[serde(rename = "proxy-auth-ip-not-unique")]
    ProxyAuthIpNotUnique,
    #[serde(rename = "public-actor-disabled")]
    PublicActorDisabled,
    #[serde(rename = "query-timeout")]
    QueryTimeout,
    #[serde(rename = "quoted-price-outdated")]
    QuotedPriceOutdated,
    #[serde(rename = "rate-limit-exceeded")]
    RateLimitExceeded,
    #[serde(rename = "recaptcha-invalid")]
    RecaptchaInvalid,
    #[serde(rename = "recaptcha-required")]
    RecaptchaRequired,
    #[serde(rename = "record-not-found")]
    RecordNotFound,
    #[serde(rename = "record-not-public")]
    RecordNotPublic,
    #[serde(rename = "record-or-token-not-found")]
    RecordOrTokenNotFound,
    #[serde(rename = "record-too-large")]
    RecordTooLarge,
    #[serde(rename = "redirect-uri-mismatch")]
    RedirectUriMismatch,
    #[serde(rename = "reduced-plan-not-available")]
    ReducedPlanNotAvailable,
    #[serde(rename = "rental-charge-already-reimbursed")]
    RentalChargeAlreadyReimbursed,
    #[serde(rename = "rental-not-allowed")]
    RentalNotAllowed,
    #[serde(rename = "request-aborted-prematurely")]
    RequestAbortedPrematurely,
    #[serde(rename = "request-handled-or-locked")]
    RequestHandledOrLocked,
    #[serde(rename = "request-id-invalid")]
    RequestIdInvalid,
    #[serde(rename = "request-queue-duplicate-requests")]
    RequestQueueDuplicateRequests,
    #[serde(rename = "request-too-large")]
    RequestTooLarge,
    #[serde(rename = "requested-dataset-view-does-not-exist")]
    RequestedDatasetViewDoesNotExist,
    #[serde(rename = "resume-token-expired")]
    ResumeTokenExpired,
    #[serde(rename = "run-failed")]
    RunFailed,
    #[serde(rename = "run-input-body-not-valid-json")]
    RunInputBodyNotValidJson,
    #[serde(rename = "run-timeout-exceeded")]
    RunTimeoutExceeded,
    #[serde(rename = "russia-is-evil")]
    RussiaIsEvil,
    #[serde(rename = "same-user")]
    SameUser,
    #[serde(rename = "schedule-actor-not-found")]
    ScheduleActorNotFound,
    #[serde(rename = "schedule-actor-task-not-found")]
    ScheduleActorTaskNotFound,
    #[serde(rename = "schedule-name-not-unique")]
    ScheduleNameNotUnique,
    #[serde(rename = "schema-validation")]
    SchemaValidation,
    #[serde(rename = "schema-validation-error")]
    SchemaValidationError,
    #[serde(rename = "schema-validation-failed")]
    SchemaValidationFailed,
    #[serde(rename = "sign-up-method-not-allowed")]
    SignUpMethodNotAllowed,
    #[serde(rename = "slack-integration-not-custom")]
    SlackIntegrationNotCustom,
    #[serde(rename = "socket-closed")]
    SocketClosed,
    #[serde(rename = "socket-destroyed")]
    SocketDestroyed,
    #[serde(rename = "store-schema-invalid")]
    StoreSchemaInvalid,
    #[serde(rename = "store-terms-not-accepted")]
    StoreTermsNotAccepted,
    #[serde(rename = "stripe-enabled")]
    StripeEnabled,
    #[serde(rename = "stripe-generic-decline")]
    StripeGenericDecline,
    #[serde(rename = "stripe-not-enabled")]
    StripeNotEnabled,
    #[serde(rename = "stripe-not-enabled-for-user")]
    StripeNotEnabledForUser,
    #[serde(rename = "tagged-build-required")]
    TaggedBuildRequired,
    #[serde(rename = "tax-country-invalid")]
    TaxCountryInvalid,
    #[serde(rename = "tax-number-invalid")]
    TaxNumberInvalid,
    #[serde(rename = "tax-number-validation-failed")]
    TaxNumberValidationFailed,
    #[serde(rename = "taxamo-call-failed")]
    TaxamoCallFailed,
    #[serde(rename = "taxamo-request-failed")]
    TaxamoRequestFailed,
    #[serde(rename = "testing-error")]
    TestingError,
    #[serde(rename = "token-not-provided")]
    TokenNotProvided,
    #[serde(rename = "too-few-versions")]
    TooFewVersions,
    #[serde(rename = "too-many-actor-tasks")]
    TooManyActorTasks,
    #[serde(rename = "too-many-actors")]
    TooManyActors,
    #[serde(rename = "too-many-labels-on-resource")]
    TooManyLabelsOnResource,
    #[serde(rename = "too-many-mcp-connectors")]
    TooManyMcpConnectors,
    #[serde(rename = "too-many-o-auth-apps")]
    TooManyOAuthApps,
    #[serde(rename = "too-many-organizations")]
    TooManyOrganizations,
    #[serde(rename = "too-many-requests")]
    TooManyRequests,
    #[serde(rename = "too-many-schedules")]
    TooManySchedules,
    #[serde(rename = "too-many-ui-access-keys")]
    TooManyUiAccessKeys,
    #[serde(rename = "too-many-user-labels")]
    TooManyUserLabels,
    #[serde(rename = "too-many-values")]
    TooManyValues,
    #[serde(rename = "too-many-versions")]
    TooManyVersions,
    #[serde(rename = "too-many-webhooks")]
    TooManyWebhooks,
    #[serde(rename = "unexpected-route")]
    UnexpectedRoute,
    #[serde(rename = "unknown-build-tag")]
    UnknownBuildTag,
    #[serde(rename = "unknown-payment-provider")]
    UnknownPaymentProvider,
    #[serde(rename = "unsubscribe-token-invalid")]
    UnsubscribeTokenInvalid,
    #[serde(rename = "unsupported-actor-pricing-model-for-agentic-payments")]
    UnsupportedActorPricingModelForAgenticPayments,
    #[serde(rename = "unsupported-content-encoding")]
    UnsupportedContentEncoding,
    #[serde(rename = "unsupported-file-type-for-issue")]
    UnsupportedFileTypeForIssue,
    #[serde(rename = "unsupported-file-type-image-expected")]
    UnsupportedFileTypeImageExpected,
    #[serde(rename = "unsupported-file-type-text-or-json-expected")]
    UnsupportedFileTypeTextOrJsonExpected,
    #[serde(rename = "unsupported-permission")]
    UnsupportedPermission,
    #[serde(rename = "upcoming-subscription-bill-not-up-to-date")]
    UpcomingSubscriptionBillNotUpToDate,
    #[serde(rename = "user-already-exists")]
    UserAlreadyExists,
    #[serde(rename = "user-already-verified")]
    UserAlreadyVerified,
    #[serde(rename = "user-creates-organizations-too-fast")]
    UserCreatesOrganizationsTooFast,
    #[serde(rename = "user-disabled")]
    UserDisabled,
    #[serde(rename = "user-email-is-disposable")]
    UserEmailIsDisposable,
    #[serde(rename = "user-email-not-set")]
    UserEmailNotSet,
    #[serde(rename = "user-email-not-verified")]
    UserEmailNotVerified,
    #[serde(rename = "user-has-no-subscription")]
    UserHasNoSubscription,
    #[serde(rename = "user-integration-not-found")]
    UserIntegrationNotFound,
    #[serde(rename = "user-is-already-invited")]
    UserIsAlreadyInvited,
    #[serde(rename = "user-is-already-organization-member")]
    UserIsAlreadyOrganizationMember,
    #[serde(rename = "user-is-not-member-of-organization")]
    UserIsNotMemberOfOrganization,
    #[serde(rename = "user-is-not-organization")]
    UserIsNotOrganization,
    #[serde(rename = "user-is-organization")]
    UserIsOrganization,
    #[serde(rename = "user-is-organization-owner")]
    UserIsOrganizationOwner,
    #[serde(rename = "user-is-removed")]
    UserIsRemoved,
    #[serde(rename = "user-not-found")]
    UserNotFound,
    #[serde(rename = "user-not-logged-in")]
    UserNotLoggedIn,
    #[serde(rename = "user-not-verified")]
    UserNotVerified,
    #[serde(rename = "user-or-token-not-found")]
    UserOrTokenNotFound,
    #[serde(rename = "user-plan-not-allowed-for-coupon")]
    UserPlanNotAllowedForCoupon,
    #[serde(rename = "user-problem-with-card")]
    UserProblemWithCard,
    #[serde(rename = "user-record-not-found")]
    UserRecordNotFound,
    #[serde(rename = "username-already-taken")]
    UsernameAlreadyTaken,
    #[serde(rename = "username-missing")]
    UsernameMissing,
    #[serde(rename = "username-not-allowed")]
    UsernameNotAllowed,
    #[serde(rename = "username-removal-forbidden")]
    UsernameRemovalForbidden,
    #[serde(rename = "username-required")]
    UsernameRequired,
    #[serde(rename = "verification-email-already-sent")]
    VerificationEmailAlreadySent,
    #[serde(rename = "verification-token-expired")]
    VerificationTokenExpired,
    #[serde(rename = "version-already-exists")]
    VersionAlreadyExists,
    #[serde(rename = "versions-size-exceeded")]
    VersionsSizeExceeded,
    #[serde(rename = "weak-password")]
    WeakPassword,
    #[serde(rename = "x402-agentic-payment-already-finalized")]
    X402AgenticPaymentAlreadyFinalized,
    #[serde(rename = "x402-agentic-payment-insufficient-amount")]
    X402AgenticPaymentInsufficientAmount,
    #[serde(rename = "x402-agentic-payment-malformed-token")]
    X402AgenticPaymentMalformedToken,
    #[serde(rename = "x402-agentic-payment-settlement-failed")]
    X402AgenticPaymentSettlementFailed,
    #[serde(rename = "x402-agentic-payment-settlement-in-progress")]
    X402AgenticPaymentSettlementInProgress,
    #[serde(rename = "x402-agentic-payment-settlement-stuck")]
    X402AgenticPaymentSettlementStuck,
    #[serde(rename = "x402-agentic-payment-unauthorized")]
    X402AgenticPaymentUnauthorized,
    #[serde(rename = "x402-payment-required")]
    X402PaymentRequired,
    #[serde(rename = "zero-invoice")]
    ZeroInvoice,

}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Variant3dSecureAuthFailed => write!(f, "3d-secure-auth-failed"),
            Self::AccessRightAlreadyExists => write!(f, "access-right-already-exists"),
            Self::ActionNotFound => write!(f, "action-not-found"),
            Self::ActorAlreadyRented => write!(f, "actor-already-rented"),
            Self::ActorCanNotBeRented => write!(f, "actor-can-not-be-rented"),
            Self::ActorDisabled => write!(f, "actor-disabled"),
            Self::ActorIsNotRented => write!(f, "actor-is-not-rented"),
            Self::ActorMemoryLimitExceeded => write!(f, "actor-memory-limit-exceeded"),
            Self::ActorNameExistsNewOwner => write!(f, "actor-name-exists-new-owner"),
            Self::ActorNameNotUnique => write!(f, "actor-name-not-unique"),
            Self::ActorNotFound => write!(f, "actor-not-found"),
            Self::ActorNotGithubActor => write!(f, "actor-not-github-actor"),
            Self::ActorNotPublic => write!(f, "actor-not-public"),
            Self::ActorPermissionLevelNotSupportedForAgenticPayments => write!(f, "actor-permission-level-not-supported-for-agentic-payments"),
            Self::ActorReviewAlreadyExists => write!(f, "actor-review-already-exists"),
            Self::ActorRunFailed => write!(f, "actor-run-failed"),
            Self::ActorStandbyNotSupportedForAgenticPayments => write!(f, "actor-standby-not-supported-for-agentic-payments"),
            Self::ActorTaskNameNotUnique => write!(f, "actor-task-name-not-unique"),
            Self::AgenticPaymentInfoRetrievalError => write!(f, "agentic-payment-info-retrieval-error"),
            Self::AgenticPaymentInformationMissing => write!(f, "agentic-payment-information-missing"),
            Self::AgenticPaymentInsufficientAmount => write!(f, "agentic-payment-insufficient-amount"),
            Self::AgenticPaymentProviderInternalError => write!(f, "agentic-payment-provider-internal-error"),
            Self::AgenticPaymentProviderUnauthorized => write!(f, "agentic-payment-provider-unauthorized"),
            Self::AirtableWebhookDeprecated => write!(f, "airtable-webhook-deprecated"),
            Self::AlreadySubscribedToPaidActor => write!(f, "already-subscribed-to-paid-actor"),
            Self::ApifyPlanRequiredToUsePaidActor => write!(f, "apify-plan-required-to-use-paid-actor"),
            Self::ApifySignupNotAllowed => write!(f, "apify-signup-not-allowed"),
            Self::AuthMethodNotSupported => write!(f, "auth-method-not-supported"),
            Self::AuthorizationServerNotFound => write!(f, "authorization-server-not-found"),
            Self::AutoIssueDateInvalid => write!(f, "auto-issue-date-invalid"),
            Self::BackgroundCheckRequired => write!(f, "background-check-required"),
            Self::BillingSystemError => write!(f, "billing-system-error"),
            Self::BlackFridayPlanExpired => write!(f, "black-friday-plan-expired"),
            Self::BraintreeError => write!(f, "braintree-error"),
            Self::BraintreeNotLinked => write!(f, "braintree-not-linked"),
            Self::BraintreeOperationTimedOut => write!(f, "braintree-operation-timed-out"),
            Self::BraintreeUnsupportedCurrency => write!(f, "braintree-unsupported-currency"),
            Self::BuildNotFound => write!(f, "build-not-found"),
            Self::BuildOutdated => write!(f, "build-outdated"),
            Self::CannotAddApifyEventsToPpeActor => write!(f, "cannot-add-apify-events-to-ppe-actor"),
            Self::CannotAddMultiplePricingInfos => write!(f, "cannot-add-multiple-pricing-infos"),
            Self::CannotAddPricingInfoThatAltersPast => write!(f, "cannot-add-pricing-info-that-alters-past"),
            Self::CannotAddSecondFuturePricingInfo => write!(f, "cannot-add-second-future-pricing-info"),
            Self::CannotBuildActorFromWebhook => write!(f, "cannot-build-actor-from-webhook"),
            Self::CannotChangeBillingInterval => write!(f, "cannot-change-billing-interval"),
            Self::CannotChangeOwner => write!(f, "cannot-change-owner"),
            Self::CannotChargeApifyEvent => write!(f, "cannot-charge-apify-event"),
            Self::CannotChargeNonPayPerEventActor => write!(f, "cannot-charge-non-pay-per-event-actor"),
            Self::CannotCommentAsOtherUser => write!(f, "cannot-comment-as-other-user"),
            Self::CannotCopyActorTask => write!(f, "cannot-copy-actor-task"),
            Self::CannotCreatePayout => write!(f, "cannot-create-payout"),
            Self::CannotCreatePublicActor => write!(f, "cannot-create-public-actor"),
            Self::CannotCreateTaxTransaction => write!(f, "cannot-create-tax-transaction"),
            Self::CannotDeleteCriticalActor => write!(f, "cannot-delete-critical-actor"),
            Self::CannotDeleteInvoice => write!(f, "cannot-delete-invoice"),
            Self::CannotDeletePaidActor => write!(f, "cannot-delete-paid-actor"),
            Self::CannotDisableOneTimeEventForApifyStartEvent => write!(f, "cannot-disable-one-time-event-for-apify-start-event"),
            Self::CannotDisableOrganizationWithEnabledMembers => write!(f, "cannot-disable-organization-with-enabled-members"),
            Self::CannotDisableUserWithSubscription => write!(f, "cannot-disable-user-with-subscription"),
            Self::CannotLinkOauthToUnverifiedEmail => write!(f, "cannot-link-oauth-to-unverified-email"),
            Self::CannotMetamorphToPayPerResultActor => write!(f, "cannot-metamorph-to-pay-per-result-actor"),
            Self::CannotModifyActorPricingTooFrequently => write!(f, "cannot-modify-actor-pricing-too-frequently"),
            Self::CannotModifyActorPricingWithImmediateEffect => write!(f, "cannot-modify-actor-pricing-with-immediate-effect"),
            Self::CannotOverridePaidActorTrial => write!(f, "cannot-override-paid-actor-trial"),
            Self::CannotPermanentlyDeleteSubscription => write!(f, "cannot-permanently-delete-subscription"),
            Self::CannotPublishActor => write!(f, "cannot-publish-actor"),
            Self::CannotReduceLastFullToken => write!(f, "cannot-reduce-last-full-token"),
            Self::CannotReimburseMoreThanOriginalCharge => write!(f, "cannot-reimburse-more-than-original-charge"),
            Self::CannotReimburseNonRentalCharge => write!(f, "cannot-reimburse-non-rental-charge"),
            Self::CannotRemoveOwnActorFromRecentlyUsed => write!(f, "cannot-remove-own-actor-from-recently-used"),
            Self::CannotRemovePaymentMethod => write!(f, "cannot-remove-payment-method"),
            Self::CannotRemovePricingInfo => write!(f, "cannot-remove-pricing-info"),
            Self::CannotRemoveRunningRun => write!(f, "cannot-remove-running-run"),
            Self::CannotRemoveUserWithPublicActors => write!(f, "cannot-remove-user-with-public-actors"),
            Self::CannotRemoveUserWithSubscription => write!(f, "cannot-remove-user-with-subscription"),
            Self::CannotRemoveUserWithUnpaidInvoice => write!(f, "cannot-remove-user-with-unpaid-invoice"),
            Self::CannotRenameEnvVar => write!(f, "cannot-rename-env-var"),
            Self::CannotRentPaidActor => write!(f, "cannot-rent-paid-actor"),
            Self::CannotReviewOwnActor => write!(f, "cannot-review-own-actor"),
            Self::CannotSetAccessRightsForOwner => write!(f, "cannot-set-access-rights-for-owner"),
            Self::CannotSetIsStatusMessageTerminal => write!(f, "cannot-set-is-status-message-terminal"),
            Self::CannotUnpublishCriticalActor => write!(f, "cannot-unpublish-critical-actor"),
            Self::CannotUnpublishPaidActor => write!(f, "cannot-unpublish-paid-actor"),
            Self::CannotUnpublishProfile => write!(f, "cannot-unpublish-profile"),
            Self::CannotUpdateInvoiceField => write!(f, "cannot-update-invoice-field"),
            Self::ConcurrentRunsLimitExceeded => write!(f, "concurrent-runs-limit-exceeded"),
            Self::ConcurrentUpdateDetected => write!(f, "concurrent-update-detected"),
            Self::ConferenceTokenNotFound => write!(f, "conference-token-not-found"),
            Self::ContentEncodingForbiddenForHtml => write!(f, "content-encoding-forbidden-for-html"),
            Self::CouponAlreadyRedeemed => write!(f, "coupon-already-redeemed"),
            Self::CouponExpired => write!(f, "coupon-expired"),
            Self::CouponForNewCustomers => write!(f, "coupon-for-new-customers"),
            Self::CouponForSubscribedUsers => write!(f, "coupon-for-subscribed-users"),
            Self::CouponLimitsAreInConflictWithCurrentLimits => write!(f, "coupon-limits-are-in-conflict-with-current-limits"),
            Self::CouponMaxNumberOfRedemptionsReached => write!(f, "coupon-max-number-of-redemptions-reached"),
            Self::CouponNotFound => write!(f, "coupon-not-found"),
            Self::CouponNotUnique => write!(f, "coupon-not-unique"),
            Self::CouponsDisabled => write!(f, "coupons-disabled"),
            Self::CreateGithubIssueNotAllowed => write!(f, "create-github-issue-not-allowed"),
            Self::CreatorPlanNotAvailable => write!(f, "creator-plan-not-available"),
            Self::CronExpressionInvalid => write!(f, "cron-expression-invalid"),
            Self::DailyAiTokenLimitExceeded => write!(f, "daily-ai-token-limit-exceeded"),
            Self::DailyPublicationLimitExceeded => write!(f, "daily-publication-limit-exceeded"),
            Self::DatasetDoesNotHaveFieldsSchema => write!(f, "dataset-does-not-have-fields-schema"),
            Self::DatasetDoesNotHaveSchema => write!(f, "dataset-does-not-have-schema"),
            Self::DatasetLocked => write!(f, "dataset-locked"),
            Self::DatasetSchemaInvalid => write!(f, "dataset-schema-invalid"),
            Self::DcrNotSupported => write!(f, "dcr-not-supported"),
            Self::DefaultDatasetNotFound => write!(f, "default-dataset-not-found"),
            Self::DeletingDefaultBuild => write!(f, "deleting-default-build"),
            Self::DeletingUnfinishedBuild => write!(f, "deleting-unfinished-build"),
            Self::EmailAlreadyTaken => write!(f, "email-already-taken"),
            Self::EmailAlreadyTakenRemovedUser => write!(f, "email-already-taken-removed-user"),
            Self::EmailDomainNotAllowedForCoupon => write!(f, "email-domain-not-allowed-for-coupon"),
            Self::EmailInvalid => write!(f, "email-invalid"),
            Self::EmailNotAllowed => write!(f, "email-not-allowed"),
            Self::EmailNotValid => write!(f, "email-not-valid"),
            Self::EmailUpdateTooSoon => write!(f, "email-update-too-soon"),
            Self::ElevatedPermissionsNeeded => write!(f, "elevated-permissions-needed"),
            Self::EnvVarAlreadyExists => write!(f, "env-var-already-exists"),
            Self::ExchangeRateFetchFailed => write!(f, "exchange-rate-fetch-failed"),
            Self::ExpiredConferenceToken => write!(f, "expired-conference-token"),
            Self::FailedToChargeUser => write!(f, "failed-to-charge-user"),
            Self::FinalInvoiceNegative => write!(f, "final-invoice-negative"),
            Self::FullPermissionActorNotApproved => write!(f, "full-permission-actor-not-approved"),
            Self::GithubBranchEmpty => write!(f, "github-branch-empty"),
            Self::GithubIssueAlreadyExists => write!(f, "github-issue-already-exists"),
            Self::GithubPublicKeyNotFound => write!(f, "github-public-key-not-found"),
            Self::GithubRepositoryNotFound => write!(f, "github-repository-not-found"),
            Self::GithubSignatureDoesNotMatchPayload => write!(f, "github-signature-does-not-match-payload"),
            Self::GithubUserNotAuthorizedForIssues => write!(f, "github-user-not-authorized-for-issues"),
            Self::GmailNotAllowed => write!(f, "gmail-not-allowed"),
            Self::IdDoesNotMatch => write!(f, "id-does-not-match"),
            Self::IncompatibleBillingInterval => write!(f, "incompatible-billing-interval"),
            Self::IncompletePayoutBillingInfo => write!(f, "incomplete-payout-billing-info"),
            Self::InconsistentCurrencies => write!(f, "inconsistent-currencies"),
            Self::IncorrectPricingModifierPrefix => write!(f, "incorrect-pricing-modifier-prefix"),
            Self::InputJsonInvalidCharacters => write!(f, "input-json-invalid-characters"),
            Self::InputJsonNotObject => write!(f, "input-json-not-object"),
            Self::InputJsonTooLong => write!(f, "input-json-too-long"),
            Self::InputUpdateCollision => write!(f, "input-update-collision"),
            Self::InsufficientPermissions => write!(f, "insufficient-permissions"),
            Self::InsufficientPermissionsToChangeField => write!(f, "insufficient-permissions-to-change-field"),
            Self::InsufficientSecurityMeasures => write!(f, "insufficient-security-measures"),
            Self::InsufficientTaxCountryEvidence => write!(f, "insufficient-tax-country-evidence"),
            Self::IntegrationAuthError => write!(f, "integration-auth-error"),
            Self::InternalServerError => write!(f, "internal-server-error"),
            Self::InvalidBillingInfo => write!(f, "invalid-billing-info"),
            Self::InvalidBillingPeriodForPayout => write!(f, "invalid-billing-period-for-payout"),
            Self::InvalidBuild => write!(f, "invalid-build"),
            Self::InvalidClientKey => write!(f, "invalid-client-key"),
            Self::InvalidCollection => write!(f, "invalid-collection"),
            Self::InvalidConferenceLoginPassword => write!(f, "invalid-conference-login-password"),
            Self::InvalidContentTypeHeader => write!(f, "invalid-content-type-header"),
            Self::InvalidCredentials => write!(f, "invalid-credentials"),
            Self::InvalidGitAuthToken => write!(f, "invalid-git-auth-token"),
            Self::InvalidGithubIssueUrl => write!(f, "invalid-github-issue-url"),
            Self::InvalidHeader => write!(f, "invalid-header"),
            Self::InvalidId => write!(f, "invalid-id"),
            Self::InvalidIdempotencyKey => write!(f, "invalid-idempotency-key"),
            Self::InvalidInput => write!(f, "invalid-input"),
            Self::InvalidInputSchema => write!(f, "invalid-input-schema"),
            Self::InvalidInvoice => write!(f, "invalid-invoice"),
            Self::InvalidInvoiceType => write!(f, "invalid-invoice-type"),
            Self::InvalidIssueDate => write!(f, "invalid-issue-date"),
            Self::InvalidLabelParams => write!(f, "invalid-label-params"),
            Self::InvalidMainAccountUserId => write!(f, "invalid-main-account-user-id"),
            Self::InvalidOauthApp => write!(f, "invalid-oauth-app"),
            Self::InvalidOauthScope => write!(f, "invalid-oauth-scope"),
            Self::InvalidOneTimeInvoice => write!(f, "invalid-one-time-invoice"),
            Self::InvalidParameter => write!(f, "invalid-parameter"),
            Self::InvalidPayoutStatus => write!(f, "invalid-payout-status"),
            Self::InvalidPictureUrl => write!(f, "invalid-picture-url"),
            Self::InvalidRecordKey => write!(f, "invalid-record-key"),
            Self::InvalidRequest => write!(f, "invalid-request"),
            Self::InvalidResourceType => write!(f, "invalid-resource-type"),
            Self::InvalidSignature => write!(f, "invalid-signature"),
            Self::InvalidSubscriptionPlan => write!(f, "invalid-subscription-plan"),
            Self::InvalidTaxNumber => write!(f, "invalid-tax-number"),
            Self::InvalidTaxNumberFormat => write!(f, "invalid-tax-number-format"),
            Self::InvalidToken => write!(f, "invalid-token"),
            Self::InvalidTokenType => write!(f, "invalid-token-type"),
            Self::InvalidTwoFactorCode => write!(f, "invalid-two-factor-code"),
            Self::InvalidTwoFactorCodeOrRecoveryCode => write!(f, "invalid-two-factor-code-or-recovery-code"),
            Self::InvalidTwoFactorRecoveryCode => write!(f, "invalid-two-factor-recovery-code"),
            Self::InvalidUsername => write!(f, "invalid-username"),
            Self::InvalidValue => write!(f, "invalid-value"),
            Self::InvitationInvalidResourceType => write!(f, "invitation-invalid-resource-type"),
            Self::InvitationNoLongerValid => write!(f, "invitation-no-longer-valid"),
            Self::InvoiceCanceled => write!(f, "invoice-canceled"),
            Self::InvoiceCannotBeRefundedDueToTooHighAmount => write!(f, "invoice-cannot-be-refunded-due-to-too-high-amount"),
            Self::InvoiceIncomplete => write!(f, "invoice-incomplete"),
            Self::InvoiceIsDraft => write!(f, "invoice-is-draft"),
            Self::InvoiceLocked => write!(f, "invoice-locked"),
            Self::InvoiceMustBeBuffer => write!(f, "invoice-must-be-buffer"),
            Self::InvoiceNotCanceled => write!(f, "invoice-not-canceled"),
            Self::InvoiceNotDraft => write!(f, "invoice-not-draft"),
            Self::InvoiceNotFound => write!(f, "invoice-not-found"),
            Self::InvoiceOutdated => write!(f, "invoice-outdated"),
            Self::InvoicePaidAlready => write!(f, "invoice-paid-already"),
            Self::IssueAlreadyConnectedToGithub => write!(f, "issue-already-connected-to-github"),
            Self::IssueNotFound => write!(f, "issue-not-found"),
            Self::IssuesBadRequest => write!(f, "issues-bad-request"),
            Self::IssuerNotRegistered => write!(f, "issuer-not-registered"),
            Self::JobFinished => write!(f, "job-finished"),
            Self::LabelAlreadyLinked => write!(f, "label-already-linked"),
            Self::LastApiToken => write!(f, "last-api-token"),
            Self::LimitReached => write!(f, "limit-reached"),
            Self::MaxItemsMustBeGreaterThanZero => write!(f, "max-items-must-be-greater-than-zero"),
            Self::MaxMetamorphsExceeded => write!(f, "max-metamorphs-exceeded"),
            Self::MaxTotalChargeUsdBelowMinimum => write!(f, "max-total-charge-usd-below-minimum"),
            Self::MaxTotalChargeUsdMustBeGreaterThanZero => write!(f, "max-total-charge-usd-must-be-greater-than-zero"),
            Self::MethodNotAllowed => write!(f, "method-not-allowed"),
            Self::MigrationDisabled => write!(f, "migration-disabled"),
            Self::MissingActorRights => write!(f, "missing-actor-rights"),
            Self::MissingApiToken => write!(f, "missing-api-token"),
            Self::MissingBillingInfo => write!(f, "missing-billing-info"),
            Self::MissingLineItems => write!(f, "missing-line-items"),
            Self::MissingPaymentDate => write!(f, "missing-payment-date"),
            Self::MissingPayoutBillingInfo => write!(f, "missing-payout-billing-info"),
            Self::MissingProxyPassword => write!(f, "missing-proxy-password"),
            Self::MissingReportingFields => write!(f, "missing-reporting-fields"),
            Self::MissingResourceName => write!(f, "missing-resource-name"),
            Self::MissingSettings => write!(f, "missing-settings"),
            Self::MissingUsername => write!(f, "missing-username"),
            Self::MonthlyUsageLimitTooLow => write!(f, "monthly-usage-limit-too-low"),
            Self::MoreThanOneUpdateNotAllowed => write!(f, "more-than-one-update-not-allowed"),
            Self::MultipleRecordsFound => write!(f, "multiple-records-found"),
            Self::MustBeAdmin => write!(f, "must-be-admin"),
            Self::NameNotUnique => write!(f, "name-not-unique"),
            Self::NextRuntimeComputationFailed => write!(f, "next-runtime-computation-failed"),
            Self::NoColumnsInExportedDataset => write!(f, "no-columns-in-exported-dataset"),
            Self::NoPaymentAttemptForRefundFound => write!(f, "no-payment-attempt-for-refund-found"),
            Self::NoPaymentMethodAvailable => write!(f, "no-payment-method-available"),
            Self::NoTeamAccountSeatsAvailable => write!(f, "no-team-account-seats-available"),
            Self::NonTemporaryEmail => write!(f, "non-temporary-email"),
            Self::NotEnoughUsageToRunPaidActor => write!(f, "not-enough-usage-to-run-paid-actor"),
            Self::NotImplemented => write!(f, "not-implemented"),
            Self::NotSupportedCurrencies => write!(f, "not-supported-currencies"),
            Self::OAuthServiceAlreadyConnected => write!(f, "o-auth-service-already-connected"),
            Self::OAuthServiceNotConnected => write!(f, "o-auth-service-not-connected"),
            Self::OauthResourceAccessFailed => write!(f, "oauth-resource-access-failed"),
            Self::OneTimeInvoiceAlreadyMarkedPaid => write!(f, "one-time-invoice-already-marked-paid"),
            Self::OnlyDraftsCanBeDeleted => write!(f, "only-drafts-can-be-deleted"),
            Self::OperationCanceled => write!(f, "operation-canceled"),
            Self::OperationNotAllowed => write!(f, "operation-not-allowed"),
            Self::OperationTimedOut => write!(f, "operation-timed-out"),
            Self::OrganizationCannotOwnItself => write!(f, "organization-cannot-own-itself"),
            Self::OrganizationRoleNotFound => write!(f, "organization-role-not-found"),
            Self::OverlappingPayoutBillingPeriods => write!(f, "overlapping-payout-billing-periods"),
            Self::OwnTokenRequired => write!(f, "own-token-required"),
            Self::PageNotFound => write!(f, "page-not-found"),
            Self::ParamNotOneOf => write!(f, "param-not-one-of"),
            Self::ParameterRequired => write!(f, "parameter-required"),
            Self::ParametersMismatched => write!(f, "parameters-mismatched"),
            Self::PasswordResetEmailAlreadySent => write!(f, "password-reset-email-already-sent"),
            Self::PasswordResetTokenExpired => write!(f, "password-reset-token-expired"),
            Self::PayAsYouGoWithoutMonthlyInterval => write!(f, "pay-as-you-go-without-monthly-interval"),
            Self::PaymentAttemptStatusMessageRequired => write!(f, "payment-attempt-status-message-required"),
            Self::PayoutAlreadyPaid => write!(f, "payout-already-paid"),
            Self::PayoutCanceled => write!(f, "payout-canceled"),
            Self::PayoutInvalidState => write!(f, "payout-invalid-state"),
            Self::PayoutMustBeApprovedToBeMarkedPaid => write!(f, "payout-must-be-approved-to-be-marked-paid"),
            Self::PayoutNotFound => write!(f, "payout-not-found"),
            Self::PayoutNumberAlreadyExists => write!(f, "payout-number-already-exists"),
            Self::PhoneNumberInvalid => write!(f, "phone-number-invalid"),
            Self::PhoneNumberLandline => write!(f, "phone-number-landline"),
            Self::PhoneNumberOptedOut => write!(f, "phone-number-opted-out"),
            Self::PhoneVerificationDisabled => write!(f, "phone-verification-disabled"),
            Self::PlatformFeatureDisabled => write!(f, "platform-feature-disabled"),
            Self::PriceOverridesValidationFailed => write!(f, "price-overrides-validation-failed"),
            Self::PricingModelNotSupported => write!(f, "pricing-model-not-supported"),
            Self::PromotionalPlanNotAvailable => write!(f, "promotional-plan-not-available"),
            Self::ProxyAuthIpNotUnique => write!(f, "proxy-auth-ip-not-unique"),
            Self::PublicActorDisabled => write!(f, "public-actor-disabled"),
            Self::QueryTimeout => write!(f, "query-timeout"),
            Self::QuotedPriceOutdated => write!(f, "quoted-price-outdated"),
            Self::RateLimitExceeded => write!(f, "rate-limit-exceeded"),
            Self::RecaptchaInvalid => write!(f, "recaptcha-invalid"),
            Self::RecaptchaRequired => write!(f, "recaptcha-required"),
            Self::RecordNotFound => write!(f, "record-not-found"),
            Self::RecordNotPublic => write!(f, "record-not-public"),
            Self::RecordOrTokenNotFound => write!(f, "record-or-token-not-found"),
            Self::RecordTooLarge => write!(f, "record-too-large"),
            Self::RedirectUriMismatch => write!(f, "redirect-uri-mismatch"),
            Self::ReducedPlanNotAvailable => write!(f, "reduced-plan-not-available"),
            Self::RentalChargeAlreadyReimbursed => write!(f, "rental-charge-already-reimbursed"),
            Self::RentalNotAllowed => write!(f, "rental-not-allowed"),
            Self::RequestAbortedPrematurely => write!(f, "request-aborted-prematurely"),
            Self::RequestHandledOrLocked => write!(f, "request-handled-or-locked"),
            Self::RequestIdInvalid => write!(f, "request-id-invalid"),
            Self::RequestQueueDuplicateRequests => write!(f, "request-queue-duplicate-requests"),
            Self::RequestTooLarge => write!(f, "request-too-large"),
            Self::RequestedDatasetViewDoesNotExist => write!(f, "requested-dataset-view-does-not-exist"),
            Self::ResumeTokenExpired => write!(f, "resume-token-expired"),
            Self::RunFailed => write!(f, "run-failed"),
            Self::RunInputBodyNotValidJson => write!(f, "run-input-body-not-valid-json"),
            Self::RunTimeoutExceeded => write!(f, "run-timeout-exceeded"),
            Self::RussiaIsEvil => write!(f, "russia-is-evil"),
            Self::SameUser => write!(f, "same-user"),
            Self::ScheduleActorNotFound => write!(f, "schedule-actor-not-found"),
            Self::ScheduleActorTaskNotFound => write!(f, "schedule-actor-task-not-found"),
            Self::ScheduleNameNotUnique => write!(f, "schedule-name-not-unique"),
            Self::SchemaValidation => write!(f, "schema-validation"),
            Self::SchemaValidationError => write!(f, "schema-validation-error"),
            Self::SchemaValidationFailed => write!(f, "schema-validation-failed"),
            Self::SignUpMethodNotAllowed => write!(f, "sign-up-method-not-allowed"),
            Self::SlackIntegrationNotCustom => write!(f, "slack-integration-not-custom"),
            Self::SocketClosed => write!(f, "socket-closed"),
            Self::SocketDestroyed => write!(f, "socket-destroyed"),
            Self::StoreSchemaInvalid => write!(f, "store-schema-invalid"),
            Self::StoreTermsNotAccepted => write!(f, "store-terms-not-accepted"),
            Self::StripeEnabled => write!(f, "stripe-enabled"),
            Self::StripeGenericDecline => write!(f, "stripe-generic-decline"),
            Self::StripeNotEnabled => write!(f, "stripe-not-enabled"),
            Self::StripeNotEnabledForUser => write!(f, "stripe-not-enabled-for-user"),
            Self::TaggedBuildRequired => write!(f, "tagged-build-required"),
            Self::TaxCountryInvalid => write!(f, "tax-country-invalid"),
            Self::TaxNumberInvalid => write!(f, "tax-number-invalid"),
            Self::TaxNumberValidationFailed => write!(f, "tax-number-validation-failed"),
            Self::TaxamoCallFailed => write!(f, "taxamo-call-failed"),
            Self::TaxamoRequestFailed => write!(f, "taxamo-request-failed"),
            Self::TestingError => write!(f, "testing-error"),
            Self::TokenNotProvided => write!(f, "token-not-provided"),
            Self::TooFewVersions => write!(f, "too-few-versions"),
            Self::TooManyActorTasks => write!(f, "too-many-actor-tasks"),
            Self::TooManyActors => write!(f, "too-many-actors"),
            Self::TooManyLabelsOnResource => write!(f, "too-many-labels-on-resource"),
            Self::TooManyMcpConnectors => write!(f, "too-many-mcp-connectors"),
            Self::TooManyOAuthApps => write!(f, "too-many-o-auth-apps"),
            Self::TooManyOrganizations => write!(f, "too-many-organizations"),
            Self::TooManyRequests => write!(f, "too-many-requests"),
            Self::TooManySchedules => write!(f, "too-many-schedules"),
            Self::TooManyUiAccessKeys => write!(f, "too-many-ui-access-keys"),
            Self::TooManyUserLabels => write!(f, "too-many-user-labels"),
            Self::TooManyValues => write!(f, "too-many-values"),
            Self::TooManyVersions => write!(f, "too-many-versions"),
            Self::TooManyWebhooks => write!(f, "too-many-webhooks"),
            Self::UnexpectedRoute => write!(f, "unexpected-route"),
            Self::UnknownBuildTag => write!(f, "unknown-build-tag"),
            Self::UnknownPaymentProvider => write!(f, "unknown-payment-provider"),
            Self::UnsubscribeTokenInvalid => write!(f, "unsubscribe-token-invalid"),
            Self::UnsupportedActorPricingModelForAgenticPayments => write!(f, "unsupported-actor-pricing-model-for-agentic-payments"),
            Self::UnsupportedContentEncoding => write!(f, "unsupported-content-encoding"),
            Self::UnsupportedFileTypeForIssue => write!(f, "unsupported-file-type-for-issue"),
            Self::UnsupportedFileTypeImageExpected => write!(f, "unsupported-file-type-image-expected"),
            Self::UnsupportedFileTypeTextOrJsonExpected => write!(f, "unsupported-file-type-text-or-json-expected"),
            Self::UnsupportedPermission => write!(f, "unsupported-permission"),
            Self::UpcomingSubscriptionBillNotUpToDate => write!(f, "upcoming-subscription-bill-not-up-to-date"),
            Self::UserAlreadyExists => write!(f, "user-already-exists"),
            Self::UserAlreadyVerified => write!(f, "user-already-verified"),
            Self::UserCreatesOrganizationsTooFast => write!(f, "user-creates-organizations-too-fast"),
            Self::UserDisabled => write!(f, "user-disabled"),
            Self::UserEmailIsDisposable => write!(f, "user-email-is-disposable"),
            Self::UserEmailNotSet => write!(f, "user-email-not-set"),
            Self::UserEmailNotVerified => write!(f, "user-email-not-verified"),
            Self::UserHasNoSubscription => write!(f, "user-has-no-subscription"),
            Self::UserIntegrationNotFound => write!(f, "user-integration-not-found"),
            Self::UserIsAlreadyInvited => write!(f, "user-is-already-invited"),
            Self::UserIsAlreadyOrganizationMember => write!(f, "user-is-already-organization-member"),
            Self::UserIsNotMemberOfOrganization => write!(f, "user-is-not-member-of-organization"),
            Self::UserIsNotOrganization => write!(f, "user-is-not-organization"),
            Self::UserIsOrganization => write!(f, "user-is-organization"),
            Self::UserIsOrganizationOwner => write!(f, "user-is-organization-owner"),
            Self::UserIsRemoved => write!(f, "user-is-removed"),
            Self::UserNotFound => write!(f, "user-not-found"),
            Self::UserNotLoggedIn => write!(f, "user-not-logged-in"),
            Self::UserNotVerified => write!(f, "user-not-verified"),
            Self::UserOrTokenNotFound => write!(f, "user-or-token-not-found"),
            Self::UserPlanNotAllowedForCoupon => write!(f, "user-plan-not-allowed-for-coupon"),
            Self::UserProblemWithCard => write!(f, "user-problem-with-card"),
            Self::UserRecordNotFound => write!(f, "user-record-not-found"),
            Self::UsernameAlreadyTaken => write!(f, "username-already-taken"),
            Self::UsernameMissing => write!(f, "username-missing"),
            Self::UsernameNotAllowed => write!(f, "username-not-allowed"),
            Self::UsernameRemovalForbidden => write!(f, "username-removal-forbidden"),
            Self::UsernameRequired => write!(f, "username-required"),
            Self::VerificationEmailAlreadySent => write!(f, "verification-email-already-sent"),
            Self::VerificationTokenExpired => write!(f, "verification-token-expired"),
            Self::VersionAlreadyExists => write!(f, "version-already-exists"),
            Self::VersionsSizeExceeded => write!(f, "versions-size-exceeded"),
            Self::WeakPassword => write!(f, "weak-password"),
            Self::X402AgenticPaymentAlreadyFinalized => write!(f, "x402-agentic-payment-already-finalized"),
            Self::X402AgenticPaymentInsufficientAmount => write!(f, "x402-agentic-payment-insufficient-amount"),
            Self::X402AgenticPaymentMalformedToken => write!(f, "x402-agentic-payment-malformed-token"),
            Self::X402AgenticPaymentSettlementFailed => write!(f, "x402-agentic-payment-settlement-failed"),
            Self::X402AgenticPaymentSettlementInProgress => write!(f, "x402-agentic-payment-settlement-in-progress"),
            Self::X402AgenticPaymentSettlementStuck => write!(f, "x402-agentic-payment-settlement-stuck"),
            Self::X402AgenticPaymentUnauthorized => write!(f, "x402-agentic-payment-unauthorized"),
            Self::X402PaymentRequired => write!(f, "x402-payment-required"),
            Self::ZeroInvoice => write!(f, "zero-invoice"),
        }
    }
}

impl Default for ErrorType {
    fn default() -> ErrorType {
        Self::Variant3dSecureAuthFailed
    }
}

