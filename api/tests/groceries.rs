//! The groceries of a user, through the api's routes: worked out from
//! their list as it stands, the same ingredients merged, and checked or
//! adjusted item by item for as long as the list leaves the item alone.

mod helpers;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use helpers::api::{Api, Outbox, invalid, refusal, register, start};
use helpers::{TestCluster, eventually};
use imkitchen_aggregates::groceries::Groceries;
use serde_json::{Value, json};

const GROCERIES: &str = "/v1/groceries";
const ENTRIES: &str = "/v1/list/entries";

fn id(thing: &Value) -> &str {
    thing["id"].as_str().unwrap()
}

/// Writes a recipe of the user `token` signs in, for `servings`, and
/// answers its id.
async fn recipe(api: &Api, token: &str, title: &str, servings: u32, ingredients: Value) -> String {
    let body = json!({ "title": title, "servings": servings, "ingredients": ingredients });
    let (status, body) = api.post("/v1/recipes", Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    id(&body).to_owned()
}

/// Puts a meal on the list and answers the id of its entry.
async fn pick(api: &Api, token: &str, body: Value) -> String {
    let (status, body) = api.post(ENTRIES, Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    id(&body).to_owned()
}

async fn items(api: &Api, token: &str) -> Vec<Value> {
    let (status, body) = api.get(GROCERIES, Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["items"].as_array().unwrap().clone()
}

/// The groceries of the user `token` signs in, without the ids of the
/// items.
async fn shown(api: &Api, token: &str) -> Vec<Value> {
    let mut items = items(api, token).await;
    for item in &mut items {
        let id = item.as_object_mut().unwrap().remove("id").unwrap();
        assert_eq!(id.as_str().unwrap().len(), 32, "{id}");
    }
    items
}

/// The id of the item `name` counted in `unit`.
async fn item_id(api: &Api, token: &str, name: &str, unit: Option<&str>) -> String {
    let items = items(api, token).await;
    let item = items
        .iter()
        .find(|item| item["name"] == name && item.get("unit").and_then(Value::as_str) == unit);
    id(item.unwrap_or_else(|| panic!("no {name} in {items:?}"))).to_owned()
}

async fn call(api: &Api, token: &str, method: Method, path: &str, body: Option<Value>) {
    let (status, body) = api.call(method, path, Some(token), body).await;
    assert_eq!((status, body), (StatusCode::NO_CONTENT, Value::Null));
}

async fn check(api: &Api, token: &str, item_id: &str) {
    let path = format!("{GROCERIES}/items/{item_id}/check");
    call(api, token, Method::PUT, &path, None).await;
}

async fn uncheck(api: &Api, token: &str, item_id: &str) {
    let path = format!("{GROCERIES}/items/{item_id}/check");
    call(api, token, Method::DELETE, &path, None).await;
}

async fn adjust(api: &Api, token: &str, item_id: &str, quantity: f64) {
    let path = format!("{GROCERIES}/items/{item_id}/quantity");
    let body = json!({ "quantity": quantity });
    call(api, token, Method::PUT, &path, Some(body)).await;
}

async fn events(api: &Api, user_id: &str) -> usize {
    let stream = evento::read::<Groceries>(user_id)
        .execute(&api.state.executor)
        .await
        .unwrap();
    stream.len()
}

fn crepes() -> Value {
    json!([
        { "name": "Flour", "quantity": 250, "unit": "g" },
        { "name": "eggs", "quantity": 3 },
        { "name": "milk", "quantity": 0.5, "unit": "l" },
        { "name": "salt" },
    ])
}

fn gravy() -> Value {
    json!([
        { "name": "flour", "quantity": 20, "unit": "G" },
        { "name": "butter", "quantity": 30, "unit": "g" },
        { "name": "Salt" },
    ])
}

#[tokio::test]
async fn the_groceries_are_what_the_list_takes() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let crepes = recipe(&api, &token, "Crêpes", 4, crepes()).await;
    let gravy = recipe(&api, &token, "Gravy", 2, gravy()).await;
    let body = json!({ "name": "Sunday roast", "recipe_ids": [gravy] });
    let (status, roast) = api.post("/v1/plats", Some(&token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{roast}");

    // Nothing to buy for an empty list.
    let (status, body) = api.get(GROCERIES, Some(&token)).await;
    assert_eq!((status, body), (StatusCode::OK, json!({ "items": [] })));

    // A recipe, scaled to the people it is cooked for: twice what it says.
    let first = pick(&api, &token, json!({ "recipe_id": crepes, "servings": 8 })).await;
    assert_eq!(
        shown(&api, &token).await,
        [
            json!({ "name": "eggs", "quantity": 6.0, "checked": false }),
            json!({ "name": "Flour", "unit": "g", "quantity": 500.0, "checked": false }),
            json!({ "name": "milk", "unit": "l", "quantity": 1.0, "checked": false }),
            json!({ "name": "salt", "checked": false }),
        ],
        "by name"
    );

    // A plat is its recipes; what two meals both take is one item, of the
    // sum, whatever the case it is written in.
    let second = pick(
        &api,
        &token,
        json!({ "plat_id": id(&roast), "servings": 3 }),
    )
    .await;
    let both = [
        json!({ "name": "butter", "unit": "g", "quantity": 45.0, "checked": false }),
        json!({ "name": "eggs", "quantity": 6.0, "checked": false }),
        json!({ "name": "Flour", "unit": "g", "quantity": 530.0, "checked": false }),
        json!({ "name": "milk", "unit": "l", "quantity": 1.0, "checked": false }),
        json!({ "name": "salt", "checked": false }),
    ];
    assert_eq!(shown(&api, &token).await, both);

    // The same meal twice is taken twice.
    let third = pick(&api, &token, json!({ "recipe_id": gravy })).await;
    let flour = &shown(&api, &token).await[2];
    assert_eq!(flour["quantity"], 570.0, "{flour}");
    let path = format!("{ENTRIES}/{third}");
    call(&api, &token, Method::DELETE, &path, None).await;
    assert_eq!(shown(&api, &token).await, both);

    // A recipe that is rewritten is bought as it now stands, at once.
    let rewritten = json!({
        "title": "Gravy",
        "servings": 3,
        "ingredients": [
            { "name": "flour", "quantity": 1, "unit": "tbsp" },
            { "name": "stock", "quantity": 0.25, "unit": "l" },
        ],
    });
    let path = format!("/v1/recipes/{gravy}");
    call(&api, &token, Method::PUT, &path, Some(rewritten)).await;
    assert_eq!(
        shown(&api, &token).await,
        [
            json!({ "name": "eggs", "quantity": 6.0, "checked": false }),
            json!({ "name": "Flour", "unit": "g", "quantity": 500.0, "checked": false }),
            json!({ "name": "flour", "unit": "tbsp", "quantity": 1.0, "checked": false }),
            json!({ "name": "milk", "unit": "l", "quantity": 1.0, "checked": false }),
            json!({ "name": "salt", "checked": false }),
            json!({ "name": "stock", "unit": "l", "quantity": 0.25, "checked": false }),
        ],
        "another unit is another item"
    );

    // A meal replaced or taken off the list is no longer bought for, and
    // neither is a recipe that is deleted, before it is off the list.
    let path = format!("{ENTRIES}/{first}");
    let replacing = json!({ "recipe_id": crepes, "servings": 2 });
    call(&api, &token, Method::PUT, &path, Some(replacing)).await;
    let eggs = &shown(&api, &token).await[0];
    assert_eq!(eggs["quantity"], 1.5, "{eggs}");
    let path = format!("/v1/recipes/{crepes}");
    call(&api, &token, Method::DELETE, &path, None).await;
    assert_eq!(
        shown(&api, &token).await,
        [
            json!({ "name": "flour", "unit": "tbsp", "quantity": 1.0, "checked": false }),
            json!({ "name": "stock", "unit": "l", "quantity": 0.25, "checked": false }),
        ]
    );
    let path = format!("{ENTRIES}/{second}");
    call(&api, &token, Method::DELETE, &path, None).await;
    assert_eq!(shown(&api, &token).await, [] as [Value; 0]);

    api.shutdown().await;
}

#[tokio::test]
async fn an_item_stays_checked_while_the_list_leaves_it_as_it_was() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, user_id) = register(&api, &outbox, "alice@example.com").await;
    let crepes = recipe(&api, &token, "Crêpes", 4, crepes()).await;
    let gravy = recipe(&api, &token, "Gravy", 2, gravy()).await;
    pick(&api, &token, json!({ "recipe_id": crepes })).await;

    let flour = item_id(&api, &token, "Flour", Some("g")).await;
    let eggs = item_id(&api, &token, "eggs", None).await;
    let milk = item_id(&api, &token, "milk", Some("l")).await;
    let salt = item_id(&api, &token, "salt", None).await;

    // Checked, once; and bought by the litre.
    for item in [&flour, &eggs, &salt, &eggs] {
        check(&api, &token, item).await;
    }
    adjust(&api, &token, &milk, 1.0).await;
    adjust(&api, &token, &milk, 1.0).await;
    assert_eq!(
        events(&api, &user_id).await,
        4,
        "the same again is not written"
    );
    let alone = [
        json!({ "name": "eggs", "quantity": 3.0, "checked": true }),
        json!({ "name": "Flour", "unit": "g", "quantity": 250.0, "checked": true }),
        json!({
            "name": "milk",
            "unit": "l",
            "quantity": 0.5,
            "adjusted_quantity": 1.0,
            "checked": false,
        }),
        json!({ "name": "salt", "checked": true }),
    ];
    assert_eq!(shown(&api, &token).await, alone);

    // Another meal on the list: what it takes more of is to buy again,
    // what it leaves as it was stays in the basket. The items are the
    // same ones.
    let second = pick(&api, &token, json!({ "recipe_id": gravy, "servings": 2 })).await;
    assert_eq!(
        shown(&api, &token).await,
        [
            json!({ "name": "butter", "unit": "g", "quantity": 30.0, "checked": false }),
            json!({ "name": "eggs", "quantity": 3.0, "checked": true }),
            json!({ "name": "Flour", "unit": "g", "quantity": 270.0, "checked": false }),
            json!({
                "name": "milk",
                "unit": "l",
                "quantity": 0.5,
                "adjusted_quantity": 1.0,
                "checked": false,
            }),
            json!({ "name": "salt", "checked": true }),
        ]
    );
    assert_eq!(item_id(&api, &token, "Flour", Some("g")).await, flour);

    // Checked as it now is, it is not for what it was: the list back to
    // what it called for has it to buy.
    check(&api, &token, &flour).await;
    let path = format!("{ENTRIES}/{second}");
    call(&api, &token, Method::DELETE, &path, None).await;
    let unchecked = &shown(&api, &token).await[1];
    assert_eq!(
        *unchecked,
        json!({ "name": "Flour", "unit": "g", "quantity": 250.0, "checked": false })
    );
    check(&api, &token, &flour).await;
    assert_eq!(shown(&api, &token).await, alone);

    // Out of the basket, once; and bought as the list calls for again.
    let before = events(&api, &user_id).await;
    uncheck(&api, &token, &eggs).await;
    uncheck(&api, &token, &eggs).await;
    let path = format!("{GROCERIES}/items/{milk}/quantity");
    call(&api, &token, Method::DELETE, &path, None).await;
    call(&api, &token, Method::DELETE, &path, None).await;
    assert_eq!(events(&api, &user_id).await, before + 2);
    assert_eq!(
        shown(&api, &token).await,
        [
            json!({ "name": "eggs", "quantity": 3.0, "checked": false }),
            json!({ "name": "Flour", "unit": "g", "quantity": 250.0, "checked": true }),
            json!({ "name": "milk", "unit": "l", "quantity": 0.5, "checked": false }),
            json!({ "name": "salt", "checked": true }),
        ]
    );

    // An adjusted item is checked and unchecked with its amount, and
    // loses it with the amount the list called for.
    adjust(&api, &token, &flour, 1000.0).await;
    let adjusted = &shown(&api, &token).await[1];
    assert_eq!(
        *adjusted,
        json!({
            "name": "Flour",
            "unit": "g",
            "quantity": 250.0,
            "adjusted_quantity": 1000.0,
            "checked": true,
        })
    );
    pick(&api, &token, json!({ "recipe_id": crepes })).await;
    let doubled = &shown(&api, &token).await[1];
    assert_eq!(
        *doubled,
        json!({ "name": "Flour", "unit": "g", "quantity": 500.0, "checked": false })
    );

    api.shutdown().await;
}

#[tokio::test]
async fn groceries_are_their_user_s_alone() {
    let (api, outbox, _node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, bob_id) = register(&api, &outbox, "bob@example.com").await;
    let crepes = recipe(&api, &alice, "Crêpes", 4, crepes()).await;
    pick(&api, &alice, json!({ "recipe_id": crepes })).await;
    let flour = item_id(&api, &alice, "Flour", Some("g")).await;
    let quantity = json!({ "quantity": 2 });

    // Bob's list calls for nothing: there is no item of his to say
    // anything of, and nothing is said of hers.
    assert_eq!(shown(&api, &bob).await, [] as [Value; 0]);
    let checking = format!("{GROCERIES}/items/{flour}/check");
    let adjusting = format!("{GROCERIES}/items/{flour}/quantity");
    let unknown = format!("{GROCERIES}/items/{}/check", "x".repeat(300));
    for (method, path, body) in [
        (Method::PUT, checking.as_str(), None),
        (Method::DELETE, checking.as_str(), None),
        (Method::PUT, adjusting.as_str(), Some(quantity.clone())),
        (Method::DELETE, adjusting.as_str(), None),
        (Method::PUT, unknown.as_str(), None),
    ] {
        let (status, body) = api.call(method.clone(), path, Some(&bob), body).await;
        assert_eq!(
            (status, body),
            (StatusCode::NOT_FOUND, refusal("grocery_item_not_found")),
            "{method} {path}"
        );
    }
    assert_eq!(events(&api, &bob_id).await, 0);
    let hers = shown(&api, &alice).await;
    assert!(hers.iter().all(|item| item["checked"] == false), "{hers:?}");

    // An amount is one a recipe could say.
    let (status, body) = api
        .call(
            Method::PUT,
            &adjusting,
            Some(&alice),
            Some(json!({ "quantity": 0 })),
        )
        .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([{
                "field": "quantity",
                "code": "range",
                "message": "This must be between 0.001 and 1000000.0.",
                "params": { "min": 0.001, "max": 1000000.0 },
            }]))
        )
    );
    let (status, body) = api
        .call(Method::PUT, &adjusting, Some(&alice), Some(json!({})))
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, refusal("invalid_request"))
    );

    // Nobody signed in reads or writes anything.
    for (method, path, body) in [
        (Method::GET, GROCERIES, None),
        (Method::PUT, checking.as_str(), None),
        (Method::DELETE, checking.as_str(), None),
        (Method::PUT, adjusting.as_str(), Some(quantity)),
        (Method::DELETE, adjusting.as_str(), None),
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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn groceries_checked_on_a_node_are_read_on_the_others() {
    let cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let (token, _) = register(&apis[0], &outbox, "alice@example.com").await;
    let crepes = recipe(&apis[0], &token, "Crêpes", 4, crepes()).await;
    pick(&apis[1], &token, json!({ "recipe_id": crepes })).await;

    // Worked out on every node from the list filled on one; checked on
    // one, adjusted on another.
    for api in &apis {
        eventually("the groceries to be read on the node", || async {
            (items(api, &token).await.len() == 4).then_some(())
        })
        .await;
    }
    let flour = item_id(&apis[2], &token, "Flour", Some("g")).await;
    check(&apis[2], &token, &flour).await;
    adjust(&apis[0], &token, &flour, 1000.0).await;
    for api in &apis {
        eventually("the item to be checked on the node", || async {
            let items = items(api, &token).await;
            let flour = items.iter().find(|item| item["id"] == flour.as_str())?;
            (flour["checked"] == true && flour["adjusted_quantity"] == 1000.0).then_some(())
        })
        .await;
    }

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}
