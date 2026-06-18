# Webhooks and dispatches

Obtained via `client.webhooks()` / `client.webhook(id)` and
`client.webhook_dispatches()` / `client.webhook_dispatch(id)`. Actor- and task-scoped
webhook collections are available via `actor.webhooks()` and `task.webhooks()`.

## `WebhookCollectionClient`

| Method | Arguments | Returns | Description |
|---|---|---|---|
| `list(options)` | `ListOptions` | `PaginationList<Webhook>` | Lists webhooks. |
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

`WebhookDispatchCollectionClient`: `list(options)`.
`WebhookDispatchClient`: `get()`.
