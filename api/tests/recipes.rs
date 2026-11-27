//! A user's own recipes, through the api's routes: written, read, rewritten
//! and deleted by their owner, and by nobody else.

mod helpers;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use helpers::api::{Api, Outbox, invalid, refusal, register, start};
use helpers::{TestCluster, eventually};
use serde_json::{Value, json};

fn pancakes() -> Value {
    json!({
        "title": "Pancakes",
        "description": "Easy and quick.",
        "ingredients": [
            { "name": "flour", "quantity": 250.0, "unit": "g" },
            { "name": "eggs", "quantity": 4.0 },
            { "name": "salt" },
        ],
        "instructions": [
            { "text": "Mix everything." },
            { "text": "Let the batter rest.", "timer_minutes": 60 },
            { "text": "Cook each one two minutes a side." },
        ],
        "prep_minutes": 10,
        "cook_minutes": 20,
        "servings": 4,
    })
}

/// Writes a recipe of the user `token` signs in and answers its id.
async fn write(api: &Api, token: &str, recipe: Value) -> String {
    let (status, body) = api.post("/v1/recipes", Some(token), recipe).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["id"].as_str().unwrap().to_owned()
}

/// The list at `path`, as the api answers it now.
async fn list(api: &Api, token: &str, path: &str) -> Value {
    let (status, body) = api.get(path, Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// The titles of the recipes a list answers, in its order.
fn titles(list: &Value) -> Vec<&str> {
    list["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|recipe| recipe["title"].as_str().unwrap())
        .collect()
}

/// The list at `path`, once it holds the recipes titled `expected`, in any
/// order: it follows the commands a moment later.
async fn listed(api: &Api, token: &str, path: &str, expected: &[&str]) -> Value {
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    eventually(&format!("{path} to list {expected:?}"), || async {
        let list = list(api, token, path).await;
        let mut titles = titles(&list);
        titles.sort_unstable();
        (titles == expected).then_some(list)
    })
    .await
}

#[tokio::test]
async fn a_user_writes_reads_rewrites_and_deletes_a_recipe() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;

    let mut written = pancakes();
    written["title"] = json!("  Pancakes ");
    let (status, created) = api.post("/v1/recipes", Some(&token), written).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["id"].as_str().unwrap().to_owned();
    let created_at = created["created_at"].as_u64().unwrap();
    assert!(created_at > 0);
    let mut expected = pancakes();
    expected["id"] = json!(id);
    expected["created_at"] = json!(created_at);
    expected["updated_at"] = json!(created_at);
    assert_eq!(created, expected, "as written, without the blanks");

    // Read back at once, and listed a moment later.
    let path = format!("/v1/recipes/{id}");
    let (status, read) = api.get(&path, Some(&token)).await;
    assert_eq!((status, &read), (StatusCode::OK, &created));
    let list = listed(&api, &token, "/v1/recipes", &["Pancakes"]).await;
    assert_eq!(
        list,
        json!({ "recipes": [{
            "id": id,
            "title": "Pancakes",
            "description": "Easy and quick.",
            "prep_minutes": 10,
            "cook_minutes": 20,
            "servings": 4,
            "created_at": created_at,
            "updated_at": created_at,
        }] })
    );

    // Rewritten as a whole: what the body leaves out is gone.
    let rewritten = json!({ "title": "Crêpes", "servings": 6, "description": null });
    let (status, body) = api
        .call(Method::PUT, &path, Some(&token), Some(rewritten.clone()))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, read) = api.get(&path, Some(&token)).await;
    assert_eq!(
        read,
        json!({
            "id": id,
            "title": "Crêpes",
            "ingredients": [],
            "instructions": [],
            "prep_minutes": 0,
            "cook_minutes": 0,
            "servings": 6,
            "created_at": created_at,
            "updated_at": read["updated_at"],
        })
    );
    listed(&api, &token, "/v1/recipes", &["Crêpes"]).await;

    // The same again writes nothing.
    let (status, _) = api
        .call(Method::PUT, &path, Some(&token), Some(rewritten))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let events = evento::read::<imkitchen_aggregates::recipe::Recipe>(&id)
        .execute(&api.state.executor)
        .await
        .unwrap();
    assert_eq!(events.len(), 2, "created, edited once");

    let (status, _) = api.call(Method::DELETE, &path, Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    for method in [Method::GET, Method::DELETE] {
        let (status, body) = api.call(method, &path, Some(&token), None).await;
        assert_eq!(
            (status, body),
            (StatusCode::NOT_FOUND, refusal("recipe_not_found"))
        );
    }
    let (status, body) = api
        .call(Method::PUT, &path, Some(&token), Some(pancakes()))
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, refusal("recipe_not_found"))
    );
    listed(&api, &token, "/v1/recipes", &[]).await;

    api.shutdown().await;
}

#[tokio::test]
async fn a_recipe_is_its_owners_alone() {
    let (api, outbox, _node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    let id = write(&api, &alice, pancakes()).await;
    let path = format!("/v1/recipes/{id}");
    let (_, written) = api.get(&path, Some(&alice)).await;

    // To anyone else it does not exist, read or written.
    for (method, body) in [
        (Method::GET, None),
        (
            Method::PUT,
            Some(json!({ "title": "Mine now", "servings": 1 })),
        ),
        (Method::DELETE, None),
    ] {
        let (status, body) = api.call(method.clone(), &path, Some(&bob), body).await;
        assert_eq!(
            (status, body),
            (StatusCode::NOT_FOUND, refusal("recipe_not_found")),
            "{method}"
        );
    }
    let (status, read) = api.get(&path, Some(&alice)).await;
    assert_eq!((status, read), (StatusCode::OK, written), "untouched");

    // Each lists their own, and a search finds nothing of the other's.
    write(&api, &bob, json!({ "title": "Waffles", "servings": 2 })).await;
    listed(&api, &alice, "/v1/recipes", &["Pancakes"]).await;
    listed(&api, &bob, "/v1/recipes", &["Waffles"]).await;
    let found = list(&api, &bob, "/v1/recipes?q=pancakes").await;
    assert_eq!(found, json!({ "recipes": [] }));

    // Nobody signed in reads or writes anything.
    for (method, path, body) in [
        (Method::POST, "/v1/recipes", Some(pancakes())),
        (Method::GET, "/v1/recipes", None),
        (Method::GET, path.as_str(), None),
        (Method::PUT, path.as_str(), Some(pancakes())),
        (Method::DELETE, path.as_str(), None),
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
async fn what_a_recipe_does_not_hold_is_said_field_by_field() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;

    let (status, body) = api
        .post(
            "/v1/recipes",
            Some(&token),
            json!({
                "title": "   ",
                "ingredients": [
                    { "name": "flour", "quantity": 250, "unit": "g" },
                    { "name": "", "quantity": 0 },
                ],
                "instructions": [
                    { "text": " " },
                    { "text": "Wait.", "timer_minutes": 0 },
                    { "text": "Wait.", "timer_minutes": null },
                ],
                "prep_minutes": 100000,
                "servings": 0,
            }),
        )
        .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([
                {
                    "field": "ingredients[1].name",
                    "code": "length",
                    "message": "This must be between 1 and 200 characters.",
                    "params": { "min": 1, "max": 200 },
                },
                {
                    "field": "ingredients[1].quantity",
                    "code": "range",
                    "message": "This must be between 0.001 and 1000000.0.",
                    "params": { "min": 0.001, "max": 1000000.0 },
                },
                {
                    "field": "instructions[0].text",
                    "code": "length",
                    "message": "This must be between 1 and 4000 characters.",
                    "params": { "min": 1, "max": 4000 },
                },
                {
                    "field": "instructions[1].timer_minutes",
                    "code": "range",
                    "message": "This must be between 1 and 10080.",
                    "params": { "min": 1, "max": 10080 },
                },
                {
                    "field": "prep_minutes",
                    "code": "too_large",
                    "message": "This must be at most 10080.",
                    "params": { "max": 10080 },
                },
                {
                    "field": "servings",
                    "code": "range",
                    "message": "This must be between 1 and 1000.",
                    "params": { "min": 1, "max": 1000 },
                },
                {
                    "field": "title",
                    "code": "length",
                    "message": "This must be between 1 and 200 characters.",
                    "params": { "min": 1, "max": 200 },
                },
            ]))
        )
    );

    let ingredients: Vec<Value> = (0..101).map(|_| json!({ "name": "salt" })).collect();
    let (status, body) = api
        .post(
            "/v1/recipes",
            Some(&token),
            json!({ "title": "Salt", "servings": 1, "ingredients": ingredients }),
        )
        .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([{
                "field": "ingredients",
                "code": "too_many",
                "message": "There must be at most 100 of these.",
                "params": { "max": 100 },
            }]))
        )
    );

    // A body of another shape: no servings.
    let (status, body) = api
        .post("/v1/recipes", Some(&token), json!({ "title": "Soup" }))
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, refusal("invalid_request"))
    );

    // The list's parameters are held to theirs.
    let (status, body) = api
        .get("/v1/recipes?limit=0&after=nowhere", Some(&token))
        .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([
                {
                    "field": "after",
                    "code": "invalid_cursor",
                    "message": "This is not a place in the list.",
                },
                {
                    "field": "limit",
                    "code": "range",
                    "message": "This must be between 1 and 100.",
                    "params": { "min": 1, "max": 100 },
                },
            ]))
        )
    );
    let (status, body) = api.get("/v1/recipes?limit=many", Some(&token)).await;
    assert_eq!(
        (status, body),
        (StatusCode::BAD_REQUEST, refusal("invalid_request"))
    );

    assert_eq!(
        list(&api, &token, "/v1/recipes").await,
        json!({ "recipes": [] }),
        "nothing was written"
    );

    api.shutdown().await;
}

#[tokio::test]
async fn the_list_is_read_a_page_at_a_time_and_searched() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let all = [
        "Pancakes",
        "Crème brûlée",
        "Gratin",
        "Ratatouille",
        "50% rye bread",
    ];
    for title in all {
        let mut recipe = json!({ "title": title, "servings": 4 });
        if title == "Gratin" {
            recipe["ingredients"] = json!([{ "name": "Crème fraîche" }, { "name": "Potatoes" }]);
        }
        write(&api, &token, recipe).await;
    }
    let whole = listed(&api, &token, "/v1/recipes?limit=100", &all).await;
    assert!(whole["next"].is_null(), "{whole}");
    let ordered = titles(&whole);

    // Two at a time, each page from where the one before stopped.
    let mut paged = Vec::new();
    let mut path = "/v1/recipes?limit=2".to_owned();
    for size in [2, 2, 1] {
        let page = list(&api, &token, &path).await;
        let on_page: Vec<String> = titles(&page).into_iter().map(str::to_owned).collect();
        assert_eq!(on_page.len(), size, "{page}");
        paged.extend(on_page);
        match page["next"].as_str() {
            Some(next) => path = format!("/v1/recipes?limit=2&after={next}"),
            None => assert_eq!(paged.len(), all.len(), "the last page says so"),
        }
    }
    assert_eq!(paged, ordered);

    // A title or an ingredient, whatever the case and the accents.
    for (q, found) in [
        ("creme", vec!["Crème brûlée", "Gratin"]),
        ("BRÛLÉE", vec!["Crème brûlée"]),
        ("potato", vec!["Gratin"]),
        (
            "a",
            vec!["Pancakes", "Gratin", "Ratatouille", "50% rye bread"],
        ),
        // Nothing of what was typed is a wildcard.
        ("50%25", vec!["50% rye bread"]),
        ("%25", vec!["50% rye bread"]),
        ("_", vec![]),
        ("waffle", vec![]),
    ] {
        let mut found = found;
        found.sort_unstable();
        let list = list(&api, &token, &format!("/v1/recipes?q={q}")).await;
        let mut titles = titles(&list);
        titles.sort_unstable();
        assert_eq!(titles, found, "{q}");
    }

    api.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_recipe_written_on_a_node_is_read_on_the_others() {
    let cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let (token, _) = register(&apis[0], &outbox, "alice@example.com").await;

    let id = write(&apis[0], &token, pancakes()).await;

    // The recipe itself at once, whichever node is asked; each node's list
    // once its own subscription got there.
    let path = format!("/v1/recipes/{id}");
    for api in &apis {
        let (status, read) = api.get(&path, Some(&token)).await;
        assert_eq!(status, StatusCode::OK, "{read}");
        assert_eq!(read["title"], "Pancakes");
        listed(api, &token, "/v1/recipes", &["Pancakes"]).await;
    }

    let (status, _) = apis[1]
        .call(Method::DELETE, &path, Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    for api in &apis {
        let (status, _) = api.get(&path, Some(&token)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        listed(api, &token, "/v1/recipes", &[]).await;
    }

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}
