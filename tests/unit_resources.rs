//! Offline unit tests for how resource clients address resources and report a missing one: the
//! URL path of resource ids and keys, the clients chained from a parent, and the spec query
//! parameters of the list endpoints.
//!
//! These use a mock [`HttpBackend`](apify_client::http_client::HttpBackend) and require no network
//! access or `APIFY_TOKEN`.

mod common;

use apify_client::{
    ApifyClientError, LastRunOptions, ListRequestsOptions, LogOptions, RunListOptions,
    StoreListOptions,
};
use common::mock_backend::{client_with, MockBackend, MockOutcome};
use futures_util::StreamExt;
use serde_json::{json, Value};

fn not_found() -> MockOutcome {
    MockOutcome::Status(
        404,
        br#"{"error":{"type":"record-not-found","message":"missing"}}"#.to_vec(),
    )
}

fn ok_data(data: Value) -> MockOutcome {
    MockOutcome::Status(200, json!({ "data": data }).to_string().into_bytes())
}

fn is_not_found<T>(result: Result<T, ApifyClientError>) -> bool {
    result
        .err()
        .and_then(|err| err.as_api_error().map(|api| api.is_not_found()))
        .unwrap_or(false)
}

fn is_invalid_argument<T>(result: Result<T, ApifyClientError>) -> bool {
    matches!(result.err(), Some(ApifyClientError::InvalidArgument(_)))
}

// ----- 404 handling -----------------------------------------------------------------------

/// A `404` resolves to `None` whatever its error type, for a client that names its resource.
#[tokio::test]
async fn any_not_found_resolves_to_none_for_a_client_with_an_id() {
    let backend = MockBackend::new(vec![MockOutcome::Status(404, b"{}".to_vec())]);
    let client = client_with(backend.clone(), 0);
    assert!(client.actor("a").get().await.unwrap().is_none());
    assert!(client.dataset("d").get().await.unwrap().is_none());
    assert!(client.me().get().await.unwrap().is_none());
    client
        .dataset("d")
        .delete()
        .await
        .expect("idempotent delete");
}

/// A chained client without an id cannot tell a missing run from a missing dataset, so its `404`
/// is an error.
#[tokio::test]
async fn not_found_is_an_error_for_a_client_chained_from_a_run() {
    let backend = MockBackend::new(vec![not_found()]);
    let client = client_with(backend, 0);
    let run = client.run("r");
    assert!(is_not_found(run.dataset().get().await));
    assert!(is_not_found(run.dataset().delete().await));
    assert!(is_not_found(run.key_value_store().get().await));
    assert!(is_not_found(run.key_value_store().delete().await));
    assert!(is_not_found(run.request_queue().get().await));
    assert!(is_not_found(run.request_queue().delete().await));
    assert!(is_not_found(run.log().get().await));
    assert!(is_not_found(client.build("b").log().get().await));
}

/// A record or a request is addressed by its own key, so a `404` is that record being missing,
/// even on a store or queue chained from a run.
#[tokio::test]
async fn a_missing_record_or_request_is_none_even_on_a_chained_client() {
    let backend = MockBackend::new(vec![not_found()]);
    let client = client_with(backend, 0);
    let run = client.run("r");
    assert!(run
        .key_value_store()
        .get_record("key")
        .await
        .unwrap()
        .is_none());
    assert!(!run.key_value_store().record_exists("key").await.unwrap());
    assert!(run
        .request_queue()
        .get_request("id")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn a_log_is_none_for_a_client_with_an_id_and_an_error_when_chained() {
    let backend = MockBackend::new(vec![not_found()]);
    let client = client_with(backend, 0);

    assert!(client.log("r").get().await.unwrap().is_none());
    assert!(client.log("r").get_bytes().await.unwrap().is_none());
    assert!(client.log("r").stream().await.unwrap().is_none());

    assert!(is_not_found(client.run("r").log().get_bytes().await));
    assert!(is_not_found(
        client.run("r").log().stream().await.map(|_| ())
    ));
    assert!(client.run("r").get_streamed_log().await.unwrap().is_none());
}

#[tokio::test]
async fn a_log_stream_is_yielded_for_an_existing_log() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"line".to_vec())]);
    let client = client_with(backend, 0);
    let mut stream = client
        .log("r")
        .stream()
        .await
        .unwrap()
        .expect("the log exists");
    let chunk = stream.next().await.expect("a chunk").unwrap();
    assert_eq!(chunk, b"line");
}

/// The fixed sub-paths of a resource cannot tell a missing resource from a missing sub-resource,
/// so a `404` is an error even for a client with an id.
#[tokio::test]
async fn fixed_sub_paths_return_not_found_as_an_error() {
    let backend = MockBackend::new(vec![not_found()]);
    let client = client_with(backend, 0);
    assert!(is_not_found(client.dataset("d").get_statistics().await));
    assert!(is_not_found(client.task("t").get_input().await));
    assert!(is_not_found(client.schedule("s").get_log().await));
    assert!(is_not_found(
        client.build("b").get_openapi_definition().await
    ));
    assert!(is_not_found(client.me().limits().await));
    assert!(is_not_found(client.me().monthly_usage().await));
}

// ----- resource ids and keys --------------------------------------------------------------

/// An id that cannot be placed in a URL path fails every request without sending one.
#[tokio::test]
async fn an_empty_or_dot_segment_id_fails_without_a_request() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "x" }))]);
    let client = client_with(backend.clone(), 0);
    for id in ["", ".", ".."] {
        assert!(is_invalid_argument(client.actor(id).get().await), "{id:?}");
        assert!(is_invalid_argument(client.dataset(id).get().await));
        assert!(is_invalid_argument(client.run(id).get().await));
        assert!(is_invalid_argument(client.user(id).get().await));
    }
    assert_eq!(backend.call_count(), 0);
}

/// The failure follows a client chained from a client with an invalid id.
#[tokio::test]
async fn a_client_chained_from_an_invalid_id_fails_too() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "x", "items": [] }))]);
    let client = client_with(backend.clone(), 0);
    let actor = client.actor("..");
    assert!(is_invalid_argument(
        actor
            .runs()
            .list(Default::default(), Default::default())
            .await
    ));
    assert!(is_invalid_argument(actor.version("0.1").get().await));
    assert!(is_invalid_argument(
        actor.version("0.1").env_var("A").get().await
    ));
    assert!(is_invalid_argument(client.run("..").dataset().get().await));
    assert!(is_invalid_argument(client.run("..").log().get().await));
    assert!(is_invalid_argument(client.build("..").log().get().await));
    assert!(is_invalid_argument(
        client.run("..").log().stream().await.map(|_| ())
    ));
    assert_eq!(backend.call_count(), 0);
}

#[tokio::test]
async fn an_empty_version_or_env_var_name_is_rejected() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "x" }))]);
    let client = client_with(backend.clone(), 0);
    assert!(is_invalid_argument(
        client.actor("a").version("").get().await
    ));
    assert!(is_invalid_argument(
        client.actor("a").version("0.1").env_var("").get().await
    ));
    assert_eq!(backend.call_count(), 0);
}

/// Every `/` of an id is the `~` separator of the `username~name` form.
#[tokio::test]
async fn every_slash_of_an_id_becomes_a_tilde() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "x" }))]);
    let client = client_with(backend.clone(), 0);
    client.actor("user/name/extra").get().await.unwrap();
    assert!(backend
        .last_url()
        .unwrap()
        .ends_with("/v2/actors/user~name~extra"));
    assert_eq!(apify_client::common::to_safe_id("a/b/c"), "a~b~c");
}

/// An id is percent-encoded, so it cannot add a path segment or a query string.
#[tokio::test]
async fn an_id_is_percent_encoded_in_the_path() {
    let backend = MockBackend::new(vec![ok_data(json!({ "id": "x" }))]);
    let client = client_with(backend.clone(), 0);
    client.dataset("a b?x=1#y").get().await.unwrap();
    assert!(backend
        .last_url()
        .unwrap()
        .ends_with("/v2/datasets/a%20b%3Fx%3D1%23y"));
}

#[tokio::test]
async fn a_record_key_is_validated_and_percent_encoded() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, Vec::new())]);
    let client = client_with(backend.clone(), 0);
    let store = client.key_value_store("s");

    for key in ["", ".", ".."] {
        assert!(is_invalid_argument(store.get_record(key).await), "{key:?}");
        assert!(is_invalid_argument(store.record_exists(key).await));
        assert!(is_invalid_argument(store.delete_record(key).await));
        assert!(is_invalid_argument(
            store.set_record_raw(key, Vec::new(), "text/plain").await
        ));
        assert!(is_invalid_argument(store.get_record_public_url(key).await));
    }
    assert_eq!(backend.call_count(), 0);

    store.get_record("../a b").await.unwrap();
    assert!(backend.last_url().unwrap().contains("/records/..%2Fa%20b?"));
}

#[tokio::test]
async fn a_request_id_is_validated_and_percent_encoded() {
    let backend = MockBackend::new(vec![ok_data(
        json!({ "id": "x", "url": "https://example.com" }),
    )]);
    let client = client_with(backend.clone(), 0);
    let queue = client.request_queue("q");

    for id in ["", ".", ".."] {
        assert!(is_invalid_argument(queue.get_request(id).await), "{id:?}");
        assert!(is_invalid_argument(queue.delete_request(id).await));
        assert!(is_invalid_argument(
            queue.prolong_request_lock(id, 60, false).await
        ));
        assert!(is_invalid_argument(
            queue.delete_request_lock(id, false).await
        ));
    }
    assert_eq!(backend.call_count(), 0);

    queue.get_request("a/b").await.unwrap();
    assert!(backend.last_url().unwrap().contains("/requests/a%2Fb"));
}

// ----- query parameters of the spec -------------------------------------------------------

#[tokio::test]
async fn run_list_filters_by_standby_deployment() {
    let backend = MockBackend::new(vec![ok_data(json!({ "items": [] }))]);
    let client = client_with(backend.clone(), 0);
    client
        .runs()
        .list(
            Default::default(),
            RunListOptions {
                status: vec!["RUNNING".to_string()],
                deployment: vec![
                    "SINGLE_TENANT".to_string(),
                    "MULTI_TENANT_OWNER_PAID".to_string(),
                ],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let url = backend.last_url().unwrap();
    assert!(
        url.contains("deployment=SINGLE_TENANT%2CMULTI_TENANT_OWNER_PAID"),
        "{url}"
    );

    client
        .runs()
        .list(Default::default(), Default::default())
        .await
        .unwrap();
    assert!(!backend.last_url().unwrap().contains("deployment"));
}

#[tokio::test]
async fn store_list_can_include_the_input_schema() {
    let backend = MockBackend::new(vec![ok_data(json!({ "items": [] }))]);
    let client = client_with(backend.clone(), 0);
    client
        .store()
        .list(StoreListOptions {
            include_input_schema: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(backend.last_url().unwrap().contains("includeInputSchema=1"));
}

#[tokio::test]
async fn log_download_is_sent_when_set() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"log".to_vec())]);
    let client = client_with(backend.clone(), 0);
    client
        .log("r")
        .get_with_options(LogOptions {
            download: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(backend.last_url().unwrap().contains("download=1"));
    client.log("r").get().await.unwrap();
    assert!(!backend.last_url().unwrap().contains("download"));
}

#[tokio::test]
#[allow(deprecated)]
async fn request_list_sends_the_deprecated_exclusive_start_id_only_when_set() {
    let backend = MockBackend::new(vec![ok_data(json!({ "items": [] }))]);
    let client = client_with(backend.clone(), 0);
    let queue = client.request_queue("q");

    queue
        .list_requests(ListRequestsOptions {
            cursor: Some("c".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    let url = backend.last_url().unwrap();
    assert!(url.contains("cursor=c"));
    assert!(!url.contains("exclusiveStartId"));

    queue
        .list_requests(ListRequestsOptions {
            exclusive_start_id: Some("request-id".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(backend
        .last_url()
        .unwrap()
        .contains("exclusiveStartId=request-id"));
}

/// The by-id user endpoints are not limited to the current user.
#[tokio::test]
async fn limits_and_usage_work_for_a_user_other_than_me() {
    let backend = MockBackend::new(vec![ok_data(json!({ "monthlyUsageCycle": {} }))]);
    let client = client_with(backend.clone(), 0);
    let user = client.user("someone");

    user.limits().await.unwrap();
    assert!(backend
        .last_url()
        .unwrap()
        .contains("/v2/users/someone/limits"));
    user.monthly_usage().await.unwrap();
    assert!(backend
        .last_url()
        .unwrap()
        .contains("/v2/users/someone/usage/monthly"));
    user.update_limits(&json!({ "dataRetentionDays": 7 }))
        .await
        .unwrap();
    assert!(backend
        .last_url()
        .unwrap()
        .contains("/v2/users/someone/limits"));
    assert_eq!(
        backend.last_body().unwrap(),
        br#"{"dataRetentionDays":7}"#.to_vec()
    );
}

// ----- dataset items and stored requests --------------------------------------------------

#[tokio::test]
async fn push_items_accepts_an_object_or_an_array_of_objects() {
    let backend = MockBackend::new(vec![MockOutcome::Status(201, Vec::new())]);
    let client = client_with(backend.clone(), 0);
    let dataset = client.dataset("d");

    dataset.push_items(&json!({ "a": 1 })).await.unwrap();
    dataset
        .push_items(&json!([{ "a": 1 }, { "b": { "c": [1, 2] } }]))
        .await
        .unwrap();
    dataset.push_items(&Vec::<Value>::new()).await.unwrap();
    assert_eq!(backend.call_count(), 3);
    assert_eq!(
        backend.last_body().unwrap(),
        b"[]".to_vec(),
        "the items are sent as serialized"
    );
}

/// The endpoint only accepts an object or an array of objects, so anything else fails at once
/// instead of at request time.
#[tokio::test]
async fn push_items_rejects_any_other_shape_without_a_request() {
    let backend = MockBackend::new(vec![MockOutcome::Status(201, Vec::new())]);
    let client = client_with(backend.clone(), 0);
    let dataset = client.dataset("d");

    assert!(is_invalid_argument(dataset.push_items(&"text").await));
    assert!(is_invalid_argument(
        dataset.push_items(&vec!["a", "b"]).await
    ));
    assert!(is_invalid_argument(dataset.push_items(&7).await));
    assert!(is_invalid_argument(dataset.push_items(&Value::Null).await));
    assert!(is_invalid_argument(
        dataset.push_items(&json!([{ "a": 1 }, 5])).await
    ));
    assert!(is_invalid_argument(
        dataset.push_items(&json!([[{ "a": 1 }]])).await
    ));
    assert_eq!(backend.call_count(), 0);
}

/// A request that was added but not handled yet comes back with a `null` `handledAt`, and with
/// only the fields the API requires.
#[tokio::test]
async fn a_pending_request_with_a_null_handled_at_deserializes() {
    let backend = MockBackend::new(vec![ok_data(json!({
        "id": "r1",
        "uniqueKey": "https://example.com",
        "url": "https://example.com",
        "handledAt": null
    }))]);
    let client = client_with(backend, 0);
    let request = client
        .request_queue("q")
        .get_request("r1")
        .await
        .unwrap()
        .expect("the request exists");
    assert_eq!(request.id.as_deref(), Some("r1"));
    assert_eq!(request.method, None);
    assert_eq!(request.extra.get("handledAt"), Some(&Value::Null));
}

/// The API returns a `null` `uiState` once an update changed the schedule's `cronExpression` or
/// `timezone`, although the spec types it as an object.
#[tokio::test]
async fn a_schedule_with_a_null_ui_state_deserializes() {
    let backend = MockBackend::new(vec![ok_data(json!({
        "id": "s1",
        "cronExpression": "0 * * * *",
        "uiState": null
    }))]);
    let client = client_with(backend, 0);
    let schedule = client
        .schedule("s1")
        .get()
        .await
        .unwrap()
        .expect("the schedule exists");
    assert_eq!(schedule.extra.get("uiState"), Some(&Value::Null));
}

// ----- clients chained from the last run --------------------------------------------------

/// The `status` and `origin` filters of `last_run` select the run, so the log and the storages of
/// that run have to carry them, or they would belong to the newest run of any status.
#[tokio::test]
async fn storages_and_log_chained_from_a_filtered_last_run_carry_its_filters() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"[]".to_vec())]);
    let client = client_with(backend.clone(), 0);
    let last_run = client.actor("a").last_run_with_options(LastRunOptions {
        status: Some("SUCCEEDED".to_string()),
        origin: Some("API".to_string()),
    });
    let carries_filters = |url: String, path: &str| {
        assert!(url.contains(path), "{url}");
        assert!(url.contains("status=SUCCEEDED"), "{url}");
        assert!(url.contains("origin=API"), "{url}");
    };

    last_run.log().get().await.unwrap();
    carries_filters(backend.last_url().unwrap(), "/runs/last/log");
    last_run.log().stream().await.unwrap();
    carries_filters(backend.last_url().unwrap(), "/runs/last/log");
    last_run
        .dataset()
        .list_items::<Value>(Default::default())
        .await
        .unwrap();
    carries_filters(backend.last_url().unwrap(), "/runs/last/dataset/items");
    last_run
        .dataset()
        .push_items(&json!({ "a": 1 }))
        .await
        .unwrap();
    carries_filters(backend.last_url().unwrap(), "/runs/last/dataset/items");
    last_run
        .key_value_store()
        .get_record("OUTPUT")
        .await
        .unwrap();
    carries_filters(
        backend.last_url().unwrap(),
        "/runs/last/key-value-store/records/OUTPUT",
    );
    last_run
        .key_value_store()
        .delete_record("OUTPUT")
        .await
        .unwrap();
    carries_filters(
        backend.last_url().unwrap(),
        "/runs/last/key-value-store/records/OUTPUT",
    );
    last_run
        .request_queue()
        .delete_request("request-id")
        .await
        .unwrap();
    carries_filters(
        backend.last_url().unwrap(),
        "/runs/last/request-queue/requests/request-id",
    );
}

/// The tasks' `last_run` filters apply the same way.
#[tokio::test]
async fn the_log_chained_from_a_task_last_run_carries_its_filters() {
    let backend = MockBackend::new(vec![MockOutcome::Status(200, b"log".to_vec())]);
    let client = client_with(backend.clone(), 0);
    client
        .task("t")
        .last_run(Some("FAILED"))
        .log()
        .get()
        .await
        .unwrap();
    let url = backend.last_url().unwrap();
    assert!(url.contains("/actor-tasks/t/runs/last/log"), "{url}");
    assert!(url.contains("status=FAILED"), "{url}");
}
