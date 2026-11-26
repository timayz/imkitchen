//! The tutorial of a user, through the api's routes: its steps done by the
//! commands they name, skipped and started over, the same on every node.

mod helpers;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, StatusCode};
use helpers::api::{Api, Outbox, refusal, register, start};
use helpers::{TestCluster, eventually};
use imkitchen_aggregates::tutorial::Tutorial;
use serde_json::{Value, json};

const TUTORIAL: &str = "/v1/tutorial";
const SKIP: &str = "/v1/tutorial/skip";
const RESTART: &str = "/v1/tutorial/restart";
const STEPS: [&str; 5] = ["recipe", "plat", "settings", "list", "groceries"];

fn id(thing: &Value) -> &str {
    thing["id"].as_str().unwrap()
}

/// The tutorial as it is answered when the steps `done` are, and no other.
fn tutorial(done: &[&str], skipped: bool) -> Value {
    let steps: Vec<Value> = STEPS
        .iter()
        .map(|step| json!({ "step": step, "completed": done.contains(step) }))
        .collect();
    json!({ "steps": steps, "skipped": skipped, "completed": done.len() == STEPS.len() })
}

async fn read(api: &Api, token: &str) -> Value {
    let (status, body) = api.get(TUTORIAL, Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// Waits for the tutorial of the user `token` signs in to be `expected`.
async fn reaches(api: &Api, token: &str, expected: Value) {
    let what = format!("the tutorial to be {expected}");
    eventually(&what, || async {
        (read(api, token).await == expected).then_some(())
    })
    .await;
}

async fn post(api: &Api, token: &str, path: &str) {
    let (status, body) = api.call(Method::POST, path, Some(token), None).await;
    assert_eq!((status, body), (StatusCode::NO_CONTENT, Value::Null));
}

/// Writes a recipe of the user `token` signs in and answers its id.
async fn recipe(api: &Api, token: &str, title: &str) -> String {
    let body = json!({
        "title": title,
        "servings": 4,
        "ingredients": [{ "name": "flour", "quantity": 250, "unit": "g" }],
    });
    let (status, body) = api.post("/v1/recipes", Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    id(&body).to_owned()
}

async fn plat(api: &Api, token: &str, name: &str, recipe_id: &str) {
    let body = json!({ "name": name, "recipe_ids": [recipe_id] });
    let (status, body) = api.post("/v1/plats", Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

async fn pick(api: &Api, token: &str, recipe_id: &str) {
    let body = json!({ "recipe_id": recipe_id });
    let (status, body) = api.post("/v1/list/entries", Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

/// Checks the first item of the groceries.
async fn check(api: &Api, token: &str) {
    let (_, groceries) = api.get("/v1/groceries", Some(token)).await;
    let path = format!("/v1/groceries/items/{}/check", id(&groceries["items"][0]));
    let (status, body) = api.call(Method::PUT, &path, Some(token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

async fn events(api: &Api, user_id: &str) -> usize {
    let stream = evento::read::<Tutorial>(user_id)
        .execute(&api.state.executor)
        .await
        .unwrap();
    stream.len()
}

#[tokio::test]
async fn each_step_is_done_by_the_command_it_names() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, user_id) = register(&api, &outbox, "alice@example.com").await;

    // There from the start, every step ahead.
    assert_eq!(read(&api, &token).await, tutorial(&[], false));

    let soup = recipe(&api, &token, "Soup").await;
    reaches(&api, &token, tutorial(&["recipe"], false)).await;

    // In any order: the language is a setting.
    let language = json!({ "language": "fr" });
    let (status, body) = api
        .call(
            Method::PUT,
            "/v1/settings/language",
            Some(&token),
            Some(language),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    reaches(&api, &token, tutorial(&["recipe", "settings"], false)).await;

    plat(&api, &token, "Sunday roast", &soup).await;
    reaches(
        &api,
        &token,
        tutorial(&["recipe", "plat", "settings"], false),
    )
    .await;

    pick(&api, &token, &soup).await;
    let all_but_groceries = tutorial(&["recipe", "plat", "settings", "list"], false);
    reaches(&api, &token, all_but_groceries).await;

    // Done again, a step is noted no more: the last one comes after them.
    recipe(&api, &token, "Stew").await;
    pick(&api, &token, &soup).await;
    check(&api, &token).await;
    reaches(&api, &token, tutorial(&STEPS, false)).await;
    assert_eq!(events(&api, &user_id).await, 5, "each step once");

    // Nobody signed in reads or writes anything.
    for (method, path) in [
        (Method::GET, TUTORIAL),
        (Method::POST, SKIP),
        (Method::POST, RESTART),
    ] {
        let (status, body) = api.call(method.clone(), path, None, None).await;
        assert_eq!(
            (status, body),
            (StatusCode::UNAUTHORIZED, refusal("sign_in_required")),
            "{method} {path}"
        );
    }

    api.shutdown().await;
}

#[tokio::test]
async fn a_tutorial_is_skipped_and_started_over() {
    let (api, outbox, _node, _dir) = start().await;
    let (alice, alice_id) = register(&api, &outbox, "alice@example.com").await;
    let (bob, bob_id) = register(&api, &outbox, "bob@example.com").await;

    // Nothing to start over yet.
    post(&api, &alice, RESTART).await;
    assert_eq!(events(&api, &alice_id).await, 0);

    let soup = recipe(&api, &alice, "Soup").await;
    reaches(&api, &alice, tutorial(&["recipe"], false)).await;

    // Skipped, once: what was done stays done, and nothing else is noted.
    // What Bob does after her is seen after it, and is his alone.
    post(&api, &alice, SKIP).await;
    post(&api, &alice, SKIP).await;
    assert_eq!(read(&api, &alice).await, tutorial(&["recipe"], true));
    plat(&api, &alice, "Sunday roast", &soup).await;
    recipe(&api, &bob, "Waffles").await;
    reaches(&api, &bob, tutorial(&["recipe"], false)).await;
    assert_eq!(read(&api, &alice).await, tutorial(&["recipe"], true));
    assert_eq!(events(&api, &alice_id).await, 2, "a step and a skip");
    assert_eq!(events(&api, &bob_id).await, 1);

    // Started over: no step is done, whatever she already has, and each
    // is done by the next command that does it. A generated list is one.
    post(&api, &alice, RESTART).await;
    assert_eq!(read(&api, &alice).await, tutorial(&[], false));
    eventually("the list to be generated", || async {
        let (status, _) = api.post("/v1/list/generate", Some(&alice), json!({})).await;
        (status == StatusCode::CREATED).then_some(())
    })
    .await;
    reaches(&api, &alice, tutorial(&["list"], false)).await;
    check(&api, &alice).await;
    reaches(&api, &alice, tutorial(&["list", "groceries"], false)).await;

    // And over again, not skipped this time.
    post(&api, &alice, RESTART).await;
    assert_eq!(read(&api, &alice).await, tutorial(&[], false));
    assert_eq!(read(&api, &bob).await, tutorial(&["recipe"], false));

    api.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_step_done_on_a_node_is_noted_once_and_read_on_the_others() {
    let cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let (token, user_id) = register(&apis[0], &outbox, "alice@example.com").await;

    // Every node sees the commands go by, and one of them notes the step.
    let soup = recipe(&apis[0], &token, "Soup").await;
    for api in &apis {
        reaches(api, &token, tutorial(&["recipe"], false)).await;
    }
    plat(&apis[1], &token, "Sunday roast", &soup).await;
    for api in &apis {
        reaches(api, &token, tutorial(&["recipe", "plat"], false)).await;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(events(&apis[2], &user_id).await, 2, "each step once");

    // Skipped on a node, skipped on all of them.
    post(&apis[2], &token, SKIP).await;
    for api in &apis {
        reaches(api, &token, tutorial(&["recipe", "plat"], true)).await;
    }

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}
