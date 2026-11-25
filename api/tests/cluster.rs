//! Aggregates and subscriptions on the store the api runs on: a node alone,
//! then three nodes writing at once.

mod helpers;

use std::sync::{Arc, Mutex};

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use evento::{Executor, Projection, WriteError};
use helpers::{TestCluster, TestNode, eventually};
use imkitchen_api::db::Store;

#[evento::aggregate(name = "imkitchen-test/Counter")]
pub enum Counter {
    Started { label: String },
    Incremented { by: u32 },
}

#[evento::projection]
#[evento::snapshot(none)]
pub struct CounterView {
    pub label: String,
    pub total: u32,
}

#[evento::handler]
async fn on_started(event: Event<Started>, view: &mut CounterView) -> anyhow::Result<()> {
    view.label = event.data.label.clone();
    Ok(())
}

#[evento::handler]
async fn on_incremented(event: Event<Incremented>, view: &mut CounterView) -> anyhow::Result<()> {
    view.total += event.data.by;
    Ok(())
}

fn counter<E: Executor>() -> Projection<E, CounterView> {
    Projection::new::<Counter>()
        .handler(on_started())
        .handler(on_incremented())
        .strict()
}

/// The increments a subscription saw, in the order it saw them.
type Seen = Arc<Mutex<Vec<u32>>>;

#[evento::subscription]
async fn record_increment<E: Executor>(
    ctx: &Context<'_, E>,
    event: Event<Incremented>,
) -> anyhow::Result<()> {
    let seen: Seen = ctx.extract();
    seen.lock().unwrap().push(event.data.by);
    Ok(())
}

async fn start_counter<E: Executor>(executor: &E, label: &str) -> String {
    evento::create()
        .event(&Started {
            label: label.to_owned(),
        })
        .commit(executor)
        .await
        .unwrap()
}

/// Appends one increment at the version the stream is read at, again when
/// another writer got there first.
async fn increment<E: Executor>(executor: &E, id: &str, by: u32) {
    loop {
        let version = evento::read::<Counter>(id)
            .execute(executor)
            .await
            .unwrap()
            .len();
        match evento::append(id)
            .original_version(version as u16)
            .event(&Incremented { by })
            .commit(executor)
            .await
        {
            Ok(_) => return,
            Err(WriteError::InvalidOriginalVersion) => continue,
            Err(err) => panic!("increment failed: {err}"),
        }
    }
}

/// The events of `id` this node holds in its own file.
async fn held(store: &Store, id: &str) -> Vec<evento::Event> {
    evento::read::<Counter>(id).execute(store).await.unwrap()
}

#[tokio::test]
async fn a_node_alone_writes_and_loads_an_aggregate() {
    let (node, _dir) = TestNode::alone().await;

    let id = start_counter(&node.executor, "alone").await;
    increment(&node.executor, &id, 2).await;
    increment(&node.executor, &id, 3).await;

    let view = counter()
        .load(&id)
        .execute(&node.executor)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.label, "alone");
    assert_eq!(view.total, 5);

    node.cluster.shutdown().await;
}

#[tokio::test]
async fn a_write_at_a_stale_version_is_refused() {
    let (node, _dir) = TestNode::alone().await;
    let id = start_counter(&node.executor, "stale").await;
    increment(&node.executor, &id, 1).await;

    let err = evento::append(&id)
        .original_version(1)
        .event(&Incremented { by: 1 })
        .commit(&node.executor)
        .await
        .unwrap_err();

    assert!(matches!(err, WriteError::InvalidOriginalVersion), "{err}");
    assert_eq!(held(&node.store, &id).await.len(), 2);

    node.cluster.shutdown().await;
}

#[tokio::test]
async fn a_subscription_handles_the_events_written() {
    let (node, _dir) = TestNode::alone().await;
    let seen = Seen::default();
    let id = start_counter(&node.executor, "subscribed").await;
    for by in 1..=3 {
        increment(&node.executor, &id, by).await;
    }

    SubscriptionBuilder::new("record-increment")
        .handler(record_increment())
        .data(seen.clone())
        .no_retry()
        .run_once(&node.executor)
        .await
        .unwrap();

    assert_eq!(*seen.lock().unwrap(), vec![1, 2, 3]);

    node.cluster.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn nodes_writing_at_once_lose_no_event() {
    const PER_NODE: usize = 20;
    let cluster = TestCluster::start(3).await;

    // Every node creates aggregates of its own, all at the same time.
    let writers: Vec<_> = cluster
        .nodes
        .iter()
        .enumerate()
        .map(|(n, node)| {
            let executor = node.executor.clone();
            tokio::spawn(async move {
                let mut ids = Vec::new();
                for i in 0..PER_NODE {
                    ids.push(start_counter(&executor, &format!("node{n}-{i}")).await);
                }
                ids
            })
        })
        .collect();
    let mut ids = Vec::new();
    for writer in writers {
        ids.extend(writer.await.unwrap());
    }
    assert_eq!(ids.len(), 3 * PER_NODE);

    // Every node ends up holding every one of them in its own file.
    for (n, node) in cluster.nodes.iter().enumerate() {
        for id in &ids {
            eventually(&format!("node {n} to hold {id}"), || async {
                (held(&node.store, id).await.len() == 1).then_some(())
            })
            .await;
        }
    }

    cluster.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn nodes_writing_one_aggregate_at_once_keep_one_order() {
    const PER_NODE: u32 = 10;
    let cluster = TestCluster::start(3).await;
    let id = start_counter(&cluster.nodes[0].executor, "shared").await;

    // Three nodes race for each next version of the same stream.
    let writers: Vec<_> = cluster
        .nodes
        .iter()
        .map(|node| {
            let executor = node.executor.clone();
            let id = id.clone();
            tokio::spawn(async move {
                for _ in 0..PER_NODE {
                    increment(&executor, &id, 1).await;
                }
            })
        })
        .collect();
    for writer in writers {
        writer.await.unwrap();
    }

    let expected = 1 + 3 * PER_NODE as usize;
    let mut streams = Vec::new();
    for (n, node) in cluster.nodes.iter().enumerate() {
        let events = eventually(&format!("node {n} to hold every increment"), || async {
            let events = held(&node.store, &id).await;
            (events.len() == expected).then_some(events)
        })
        .await;
        // No version taken twice, none skipped.
        let versions: Vec<u16> = events.iter().map(|event| event.version).collect();
        assert_eq!(versions, (1..=expected as u16).collect::<Vec<_>>());
        streams.push(events.into_iter().map(|event| event.id).collect::<Vec<_>>());

        let view = counter()
            .load(&id)
            .execute(&node.executor)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(view.total, 3 * PER_NODE);
    }
    // The same events in the same order on every node.
    assert_eq!(streams[0], streams[1]);
    assert_eq!(streams[0], streams[2]);

    cluster.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_subscription_follows_what_other_nodes_write() {
    let cluster = TestCluster::start(3).await;
    let seen = Seen::default();
    let subscription = SubscriptionBuilder::new("record-increment")
        .handler(record_increment())
        .data(seen.clone())
        .start(&cluster.nodes[0].executor)
        .await
        .unwrap();

    let id = start_counter(&cluster.nodes[1].executor, "followed").await;
    increment(&cluster.nodes[1].executor, &id, 1).await;
    increment(&cluster.nodes[2].executor, &id, 2).await;

    eventually("node 0's subscription to see both increments", || async {
        (seen.lock().unwrap().len() == 2).then_some(())
    })
    .await;
    assert_eq!(*seen.lock().unwrap(), vec![1, 2]);

    subscription.shutdown().await.unwrap();
    cluster.shutdown().await;
}
