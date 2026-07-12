# Webhooks and dispatches

Obtained via `client.webhooks()` / `client.webhook(id)` and
`client.webhook_dispatches()` / `client.webhook_dispatch(id)`. Actor- and task-scoped
webhook collections are available via `actor.webhooks()` and `task.webhooks()`.

## `WebhookCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions` | `PaginationList<Webhook>` | Lists webhooks. |
| `iterate(options)` | `ListOptions` | `ListIterator<Webhook>` | Lazily iterates all webhooks across pages (auto-pagination). |
| `create(webhook)` | `&impl Serialize` | `Webhook` | Creates a webhook. |

## `WebhookClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `get()` | — | `Option<Webhook>` | Fetches the webhook. |
| `update(fields)` | `&impl Serialize` | `Webhook` | Updates the webhook. |
| `delete()` | — | `()` | Deletes the webhook. |
| `test()` | — | `WebhookDispatch` | Triggers a test dispatch. |
| `dispatches()` | — | `WebhookDispatchCollectionClient` | This webhook's dispatches. |

## Webhook dispatches

`WebhookDispatchCollectionClient`: `list(options)`, `iterate(options)` (lazy
`ListIterator<WebhookDispatch>` auto-pagination).
`WebhookDispatchClient`: `get()`.

## The `Webhook` model

`Webhook` and `WebhookDispatch` live in `apify_client::models`
(`use apify_client::models::{Webhook, WebhookDispatch};`). Returned by `get`, `create`, `update`,
`test` (via the resulting dispatch's `webhook_id`) and (as `PaginationList<Webhook>`) by `list`.
Unknown fields returned by the API are preserved in `extra`.

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique webhook ID (always present); used to build a `client.webhook(&webhook.id)` client. |
| `user_id` | `Option<String>` | ID of the user who owns the webhook. |
| `request_url` | `Option<String>` | The URL that receives the webhook POST request. |
| `event_types` | `Vec<String>` | Event types that trigger this webhook (e.g. `ACTOR.RUN.SUCCEEDED`). |
| `extra` | `Extra` | Any other fields returned by the API (e.g. `condition`, `isAdHoc`, `payloadTemplate`). |

## The `WebhookDispatch` model

Returned by `WebhookClient::test`, `WebhookDispatchClient::get`, and (as
`PaginationList<WebhookDispatch>`) by `WebhookDispatchCollectionClient::list`. Unknown fields
returned by the API are preserved in `extra`.

| Field | Type | Description |
|---|---|---|
| `id` | `String` | Unique dispatch ID (always present). |
| `webhook_id` | `Option<String>` | ID of the webhook that produced this dispatch. |
| `extra` | `Extra` | Any other fields returned by the API (e.g. `status`, `attempts`, `eventType`). |
