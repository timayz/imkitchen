//! The settings of a user, through the api's routes: there from the start
//! as the defaults, read and set by the user they are of, and by nobody
//! else.

mod helpers;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use helpers::api::{Api, Outbox, invalid, refusal, register, start};
use helpers::{TestCluster, eventually};
use imkitchen_aggregates::settings::{ListSources, Settings};
use imkitchen_api::settings::{self, ListSettings};
use serde_json::{Value, json};

const SETTINGS: &str = "/v1/settings";
const LIST: &str = "/v1/settings/list";
const LANGUAGE: &str = "/v1/settings/language";

/// What a user who set nothing reads.
fn defaults() -> Value {
    json!({ "list": { "servings": 4, "meals": 7, "sources": "plats_and_recipes" } })
}

/// The settings of the user `token` signs in, as the api answers them now.
async fn read(api: &Api, token: &str) -> Value {
    let (status, body) = api.get(SETTINGS, Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// Sets the part of the settings at `path` to `body`.
async fn set(api: &Api, token: &str, path: &str, body: Value) {
    let (status, body) = api.call(Method::PUT, path, Some(token), Some(body)).await;
    assert_eq!((status, &body), (StatusCode::NO_CONTENT, &Value::Null));
}

#[tokio::test]
async fn a_user_reads_and_sets_their_settings() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, user_id) = register(&api, &outbox, "alice@example.com").await;

    // There from the start, with nothing written.
    assert_eq!(read(&api, &token).await, defaults());
    let executor = &api.state.executor;
    assert_eq!(
        settings::of(executor, &user_id).await.unwrap(),
        ListSettings::default()
    );

    // How the list is generated, as a whole.
    let list = json!({ "servings": 2, "meals": 5, "sources": "plats" });
    set(&api, &token, LIST, list.clone()).await;
    assert_eq!(read(&api, &token).await, json!({ "list": list }));
    assert_eq!(
        settings::of(executor, &user_id).await.unwrap(),
        ListSettings {
            servings: 2,
            meals: 5,
            sources: ListSources::Plats,
        },
        "what generates the list reads the same"
    );

    // The language, the list as it was.
    set(&api, &token, LANGUAGE, json!({ "language": "fr" })).await;
    let both = json!({ "list": list, "language": "fr" });
    assert_eq!(read(&api, &token).await, both);

    // The same again writes nothing.
    set(&api, &token, LIST, list.clone()).await;
    set(&api, &token, LANGUAGE, json!({ "language": "fr" })).await;
    let events = evento::read::<Settings>(&user_id)
        .execute(executor)
        .await
        .unwrap();
    assert_eq!(events.len(), 2, "the list once, the language once");

    // Each source the list may be drawn from.
    for sources in ["recipes", "plats_and_recipes"] {
        let list = json!({ "servings": 1, "meals": 31, "sources": sources });
        set(&api, &token, LIST, list.clone()).await;
        assert_eq!(read(&api, &token).await["list"], list);
    }

    // Without a language again, left out or `null`: the one of the device.
    let list = read(&api, &token).await["list"].take();
    set(&api, &token, LANGUAGE, json!({})).await;
    assert_eq!(read(&api, &token).await, json!({ "list": list }));
    set(&api, &token, LANGUAGE, json!({ "language": "en" })).await;
    set(&api, &token, LANGUAGE, json!({ "language": null })).await;
    assert_eq!(read(&api, &token).await, json!({ "list": list }));

    api.shutdown().await;
}

#[tokio::test]
async fn settings_are_their_user_s_alone() {
    let (api, outbox, _node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;

    let list = json!({ "servings": 6, "meals": 3, "sources": "recipes" });
    set(&api, &alice, LIST, list.clone()).await;
    set(&api, &alice, LANGUAGE, json!({ "language": "fr" })).await;
    assert_eq!(
        read(&api, &alice).await,
        json!({ "list": list, "language": "fr" })
    );
    assert_eq!(read(&api, &bob).await, defaults(), "bob set nothing");

    // Nobody signed in reads or sets anything.
    for (method, path, body) in [
        (Method::GET, SETTINGS, None),
        (Method::PUT, LIST, Some(list)),
        (Method::PUT, LANGUAGE, Some(json!({ "language": "en" }))),
    ] {
        let (status, body) = api.call(method.clone(), path, None, body).await;
        assert_eq!(
            (status, body),
            (StatusCode::UNAUTHORIZED, refusal("sign_in_required")),
            "{method} {path}"
        );
    }

    api.shutdown().await;
}

#[tokio::test]
async fn settings_that_do_not_hold_are_refused() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let put = |path: &'static str, body: Value| {
        let (api, token) = (&api, &token);
        async move { api.call(Method::PUT, path, Some(token), Some(body)).await }
    };

    // Each number under its field, with its bounds.
    let nobody = json!({ "servings": 0, "meals": 0, "sources": "plats" });
    assert_eq!(
        put(LIST, nobody).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([
                {
                    "field": "meals",
                    "code": "range",
                    "message": "This must be between 1 and 31.",
                    "params": { "min": 1, "max": 31 },
                },
                {
                    "field": "servings",
                    "code": "range",
                    "message": "This must be between 1 and 1000.",
                    "params": { "min": 1, "max": 1000 },
                },
            ]))
        )
    );
    let crowd = json!({ "servings": 1001, "meals": 32, "sources": "plats" });
    let (status, body) = put(LIST, crowd).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["fields"].as_array().unwrap().len(), 2);

    // A part is set as a whole, of what the api knows.
    for (path, body) in [
        (LIST, json!({ "servings": 2, "meals": 5 })),
        (
            LIST,
            json!({ "servings": 2, "meals": 5, "sources": "takeaway" }),
        ),
        (LANGUAGE, json!({ "language": "de" })),
    ] {
        assert_eq!(
            put(path, body.clone()).await,
            (StatusCode::UNPROCESSABLE_ENTITY, refusal("invalid_request")),
            "{path} {body}"
        );
    }

    assert_eq!(read(&api, &token).await, defaults(), "nothing was set");

    api.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn settings_set_on_a_node_are_read_on_the_others() {
    let cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let (token, user_id) = register(&apis[0], &outbox, "alice@example.com").await;

    // Each part on a node of its own, the second over the first.
    let list = json!({ "servings": 2, "meals": 5, "sources": "recipes" });
    set(&apis[1], &token, LIST, list.clone()).await;
    set(&apis[2], &token, LANGUAGE, json!({ "language": "fr" })).await;

    let expected = json!({ "list": list, "language": "fr" });
    for api in &apis {
        eventually("the settings to be read on the node", || async {
            (read(api, &token).await == expected).then_some(())
        })
        .await;
    }
    let stream = evento::read::<Settings>(&user_id)
        .execute(&apis[0].state.executor)
        .await
        .unwrap();
    assert_eq!(stream.len(), 2);

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}
