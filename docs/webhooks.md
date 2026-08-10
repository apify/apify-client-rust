# Webhooks and dispatches

Obtained via `client.webhooks()` / `client.webhook(id)` and
`client.webhook_dispatches()` / `client.webhook_dispatch(id)`. Actor- and task-scoped
webhook collections are available via `actor.webhooks()` and `task.webhooks()`.

> Code blocks below use the rustdoc `# `-hidden-line convention — see
> [`docs/README.md`](README.md) for what those lines are.

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
| `test()` | — | `Option<WebhookDispatch>` | Triggers a test dispatch, or `None` if the webhook no longer exists. |
| `dispatches()` | — | `WebhookDispatchCollectionClient` | This webhook's dispatches. |

## Webhook dispatches

`WebhookDispatchCollectionClient`: `list(options: ListOptions)`, `iterate(options: ListOptions)`
(lazy `ListIterator<WebhookDispatch>` auto-pagination).
`WebhookDispatchClient::get() -> Option<WebhookDispatch>`: fetches the dispatch (`None` if
missing), mirroring `WebhookClient::get()`'s `Option<Webhook>` shape.

### Creating a webhook

`create` takes the same shape as the [Create webhook API](https://docs.apify.com/api/v2/webhooks-post):
`eventTypes`, a `condition` (`{ actorId }`, `{ actorTaskId }` or `{ actorRunId }`, selecting what the
webhook watches) and a `requestUrl` to POST to.

```rust,no_run
use apify_client::ApifyClient;
use serde_json::json;

# async fn run(client: ApifyClient, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
let webhook = client
    .webhooks()
    .create(&json!({
        "eventTypes": ["ACTOR.RUN.SUCCEEDED"],
        "condition": { "actorTaskId": task_id },
        "requestUrl": "https://example.com/webhook"
    }))
    .await?;
println!("created webhook {}", webhook.id);
# Ok(())
# }
```

See the [`tasks_schedules_webhooks`](../examples/tasks_schedules_webhooks.rs) example for
`test()`-ing a webhook, listing its dispatches, `update` and `delete`.

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
