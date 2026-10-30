//! Offline unit tests for `RunClient::iterate_dataset_items`, which reads the items of a run's
//! default dataset while the run is still producing them.
//!
//! These use a mock [`HttpBackend`](apify_client::http_client::HttpBackend) that answers the
//! requests in order, and require no network access or `APIFY_TOKEN`.

mod common;

use apify_client::clients::dataset::DatasetListItemsOptions;
use common::mock_backend::{client_with, MockBackend, MockOutcome};
use serde_json::{json, Value};

fn run(status: &str) -> MockOutcome {
    MockOutcome::Status(
        200,
        json!({ "data": { "id": "r", "status": status } })
            .to_string()
            .into_bytes(),
    )
}

fn dataset(item_count: i64) -> MockOutcome {
    MockOutcome::Status(
        200,
        json!({ "data": { "id": "d", "itemCount": item_count } })
            .to_string()
            .into_bytes(),
    )
}

fn items(items: Value) -> MockOutcome {
    MockOutcome::Status(200, items.to_string().into_bytes())
}

/// The path and the query of every request, without the origin.
fn requests(backend: &MockBackend) -> Vec<String> {
    backend
        .urls()
        .iter()
        .map(|url| {
            url.split_once("/v2/")
                .map(|(_, rest)| rest)
                .unwrap()
                .to_string()
        })
        .collect()
}

#[tokio::test]
async fn items_are_read_while_the_run_produces_them_and_after_it_finishes() {
    let backend = MockBackend::new(vec![
        run("RUNNING"),
        dataset(2),
        items(json!([{ "n": 1 }, { "n": 2 }])),
        run("SUCCEEDED"), // The poll that ends the wait.
        dataset(3),
        items(json!([{ "n": 3 }])),
        items(json!([])), // The tail past a lagging `itemCount`.
    ]);
    let client = client_with(backend.clone(), 0);

    let collected: Vec<Value> = client
        .run("r")
        .iterate_dataset_items::<Value>(Default::default())
        .collect_all()
        .await
        .unwrap();

    assert_eq!(
        collected,
        vec![json!({ "n": 1 }), json!({ "n": 2 }), json!({ "n": 3 })]
    );
    let requests = requests(&backend);
    assert_eq!(requests.len(), 7, "{requests:?}");
    assert_eq!(requests[0], "actor-runs/r");
    assert_eq!(requests[1], "actor-runs/r/dataset");
    assert_eq!(requests[2], "actor-runs/r/dataset/items?offset=0&limit=2");
    assert!(
        requests[3].starts_with("actor-runs/r?waitForFinish="),
        "{}",
        requests[3]
    );
    assert_eq!(requests[4], "actor-runs/r/dataset");
    assert_eq!(requests[5], "actor-runs/r/dataset/items?offset=2&limit=1");
    assert_eq!(
        requests[6],
        "actor-runs/r/dataset/items?offset=3&limit=1000"
    );
}

#[tokio::test]
async fn items_are_yielded_one_at_a_time_as_pages_arrive() {
    let backend = MockBackend::new(vec![
        run("SUCCEEDED"),
        dataset(3),
        items(json!([{ "n": 1 }, { "n": 2 }])),
        items(json!([{ "n": 3 }])),
        items(json!([])),
    ]);
    let client = client_with(backend.clone(), 0);
    let mut iterator = client
        .run("r")
        .iterate_dataset_items::<Value>(Default::default())
        .with_chunk_size(2);

    assert_eq!(iterator.next().await.unwrap(), Some(json!({ "n": 1 })));
    assert_eq!(backend.call_count(), 3, "only the first page was read");
    assert_eq!(iterator.next().await.unwrap(), Some(json!({ "n": 2 })));
    assert_eq!(backend.call_count(), 3, "the page is buffered");
    assert_eq!(iterator.next().await.unwrap(), Some(json!({ "n": 3 })));
    assert_eq!(iterator.next().await.unwrap(), None);
    assert_eq!(
        iterator.next().await.unwrap(),
        None,
        "the iterator stays ended"
    );

    let requests = requests(&backend);
    assert_eq!(requests[2], "actor-runs/r/dataset/items?offset=0&limit=2");
    assert_eq!(requests[3], "actor-runs/r/dataset/items?offset=2&limit=1");
}

/// `offset` and `limit` bound the rows that are read, however many items the dataset has.
#[tokio::test]
async fn the_range_of_rows_is_bounded_by_offset_and_limit() {
    let backend = MockBackend::new(vec![
        run("SUCCEEDED"),
        dataset(10),
        items(json!([{ "n": 2 }, { "n": 3 }])),
    ]);
    let client = client_with(backend.clone(), 0);

    let collected: Vec<Value> = client
        .run("r")
        .iterate_dataset_items::<Value>(DatasetListItemsOptions {
            offset: Some(1),
            limit: Some(2),
            ..Default::default()
        })
        .collect_all()
        .await
        .unwrap();

    assert_eq!(collected.len(), 2);
    let requests = requests(&backend);
    assert_eq!(
        requests.len(),
        3,
        "the range is complete, so nothing past it is read"
    );
    assert_eq!(requests[2], "actor-runs/r/dataset/items?offset=1&limit=2");
}

/// A filter can empty a page past a lagging `itemCount` while rows are left behind it, so an
/// empty page ends the iteration only once a plain read confirms nothing is left.
#[tokio::test]
async fn a_filtered_empty_tail_page_is_checked_with_a_plain_read() {
    let backend = MockBackend::new(vec![
        run("SUCCEEDED"),
        dataset(0),
        items(json!([])),                // The tail page the filter emptied...
        items(json!([{ "hidden": 1 }])), // ...while the plain read finds a row behind it.
        items(json!([{ "n": 1 }])),      // The next tail page has an item.
        items(json!([])),
        items(json!([])), // The plain read finds nothing: this is the end.
    ]);
    let client = client_with(backend.clone(), 0);

    let collected: Vec<Value> = client
        .run("r")
        .iterate_dataset_items::<Value>(DatasetListItemsOptions {
            clean: Some(true),
            ..Default::default()
        })
        .with_chunk_size(5)
        .collect_all()
        .await
        .unwrap();

    assert_eq!(collected, vec![json!({ "n": 1 })]);
    let requests = requests(&backend);
    assert_eq!(
        requests[2],
        "actor-runs/r/dataset/items?offset=0&limit=5&clean=1"
    );
    assert_eq!(
        requests[3], "actor-runs/r/dataset/items?offset=0&limit=1",
        "the plain read has no filter"
    );
    assert_eq!(
        requests[4],
        "actor-runs/r/dataset/items?offset=5&limit=5&clean=1"
    );
}

/// The iterator sticks to the run that its first request resolves to, instead of asking for the
/// last run again.
#[tokio::test]
async fn a_last_run_client_sticks_to_the_run_it_resolved() {
    let backend = MockBackend::new(vec![
        MockOutcome::Status(
            200,
            json!({ "data": { "id": "resolved-run", "status": "SUCCEEDED" } })
                .to_string()
                .into_bytes(),
        ),
        dataset(1),
        items(json!([{ "n": 1 }])),
        items(json!([])),
    ]);
    let client = client_with(backend.clone(), 0);

    let collected: Vec<Value> = client
        .actor("a")
        .last_run(Some("SUCCEEDED"))
        .iterate_dataset_items::<Value>(Default::default())
        .collect_all()
        .await
        .unwrap();

    assert_eq!(collected.len(), 1);
    let requests = requests(&backend);
    assert_eq!(requests[0], "actors/a/runs/last?status=SUCCEEDED");
    for request in &requests[1..] {
        assert!(
            request.starts_with("actor-runs/resolved-run/dataset"),
            "{request}"
        );
    }
}

/// The requests use the `Long` tier unless the run client has a timeout of its own.
#[tokio::test]
async fn every_request_uses_the_long_tier_unless_overridden() {
    let script = || {
        vec![
            run("SUCCEEDED"),
            dataset(1),
            items(json!([{ "n": 1 }])),
            items(json!([])),
        ]
    };

    let backend = MockBackend::new(script());
    let client = client_with(backend.clone(), 0);
    client
        .run("r")
        .iterate_dataset_items::<Value>(Default::default())
        .collect_all()
        .await
        .unwrap();
    assert!(backend
        .timeouts()
        .iter()
        .all(|timeout| timeout.as_secs() == 360));

    let backend = MockBackend::new(script());
    let client = client_with(backend.clone(), 0);
    client
        .run("r")
        .with_timeout(apify_client::Timeout::Custom(
            std::time::Duration::from_secs(9),
        ))
        .iterate_dataset_items::<Value>(Default::default())
        .collect_all()
        .await
        .unwrap();
    assert!(backend
        .timeouts()
        .iter()
        .all(|timeout| timeout.as_secs() == 9));
}

/// The rows are counted from the oldest, so `desc` would mix up the offsets of a dataset that is
/// still growing.
#[tokio::test]
async fn descending_order_is_rejected_without_a_request() {
    let backend = MockBackend::new(vec![run("SUCCEEDED")]);
    let client = client_with(backend.clone(), 0);

    let error = client
        .run("r")
        .iterate_dataset_items::<Value>(DatasetListItemsOptions {
            desc: Some(true),
            ..Default::default()
        })
        .next()
        .await
        .expect_err("`desc` is not supported");

    assert!(
        matches!(error, apify_client::ApifyClientError::InvalidArgument(_)),
        "{error:?}"
    );
    assert_eq!(backend.call_count(), 0);
}
