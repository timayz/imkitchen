//! The list of a user, through the api's routes: generated as their
//! settings say or picked by hand, theirs alone, and rid of the recipes and
//! the plats that are deleted.

mod helpers;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use helpers::api::{Api, Outbox, invalid, refusal, register, start};
use helpers::{TestCluster, eventually};
use imkitchen_aggregates::list::List;
use serde_json::{Value, json};

const LIST: &str = "/v1/list";
const ENTRIES: &str = "/v1/list/entries";
const GENERATE: &str = "/v1/list/generate";

fn id(thing: &Value) -> &str {
    thing["id"].as_str().unwrap()
}

/// Writes a recipe of the user `token` signs in, titled `title`, and
/// answers it as a list shows it, once the list of the recipes does.
async fn recipe(api: &Api, token: &str, title: &str) -> Value {
    let body = json!({ "title": title, "servings": 4 });
    let (status, body) = api.post("/v1/recipes", Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = id(&body).to_owned();
    eventually(&format!("the recipe {title} to be listed"), || async {
        let (_, list) = api.get("/v1/recipes?limit=100", Some(token)).await;
        let listed = list["recipes"].as_array()?;
        listed.iter().find(|recipe| recipe["id"] == id).cloned()
    })
    .await
}

/// Composes a plat of the user `token` signs in and answers it, once the
/// list of the plats shows it.
async fn plat(api: &Api, token: &str, name: &str, recipes: &[&Value]) -> Value {
    let recipe_ids: Vec<&str> = recipes.iter().map(|recipe| id(recipe)).collect();
    let body = json!({ "name": name, "recipe_ids": recipe_ids });
    let (status, body) = api.post("/v1/plats", Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = id(&body).to_owned();
    eventually(&format!("the plat {name} to be listed"), || async {
        let (_, list) = api.get("/v1/plats?limit=100", Some(token)).await;
        let listed = list["plats"].as_array()?;
        listed.iter().find(|plat| plat["id"] == id).cloned()
    })
    .await
}

/// The entries of the list of the user `token` signs in, as the api
/// answers them now.
async fn entries(api: &Api, token: &str) -> Vec<Value> {
    let (status, body) = api.get(LIST, Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["entries"].as_array().unwrap().clone()
}

/// Puts a meal on the list and answers its entry.
async fn pick(api: &Api, token: &str, body: Value) -> Value {
    let (status, body) = api.post(ENTRIES, Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body
}

/// Generates the list and answers its entries.
async fn generate(api: &Api, token: &str) -> Vec<Value> {
    let (status, body) = api.post(GENERATE, Some(token), json!({})).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["entries"].as_array().unwrap().clone()
}

async fn set_list(api: &Api, token: &str, settings: Value) {
    let (status, body) = api
        .call(
            Method::PUT,
            "/v1/settings/list",
            Some(token),
            Some(settings),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

/// What a meal is, whatever its entry: the id of its recipe or of its
/// plat.
fn meal(entry: &Value) -> (&str, &str) {
    match entry.get("recipe") {
        Some(recipe) => ("recipe", id(recipe)),
        None => ("plat", id(&entry["plat"])),
    }
}

async fn events(api: &Api, user_id: &str) -> usize {
    let stream = evento::read::<List>(user_id)
        .execute(&api.state.executor)
        .await
        .unwrap();
    stream.len()
}

#[tokio::test]
async fn a_user_picks_replaces_and_removes_the_meals_of_their_list() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, user_id) = register(&api, &outbox, "alice@example.com").await;
    let soup = recipe(&api, &token, "Soup").await;
    let chicken = recipe(&api, &token, "Roast chicken").await;
    let roast = plat(&api, &token, "Sunday roast", &[&chicken]).await;

    // There from the start, empty.
    let (status, list) = api.get(LIST, Some(&token)).await;
    assert_eq!((status, list), (StatusCode::OK, json!({ "entries": [] })));

    // A recipe, for as many as the settings say; a plat, for as many as
    // asked; and the recipe once more.
    let first = pick(&api, &token, json!({ "recipe_id": id(&soup) })).await;
    assert_eq!(
        first,
        json!({ "id": id(&first), "servings": 4, "recipe": soup })
    );
    let second = pick(
        &api,
        &token,
        json!({ "plat_id": id(&roast), "servings": 2 }),
    )
    .await;
    assert_eq!(
        second,
        json!({ "id": id(&second), "servings": 2, "plat": roast })
    );
    let third = pick(&api, &token, json!({ "recipe_id": id(&soup) })).await;
    assert_ne!(id(&third), id(&first));
    assert_eq!(
        entries(&api, &token).await,
        [first.clone(), second.clone(), third.clone()],
        "in the order they were planned"
    );

    // Replaced as a whole, in the place it has: without servings, as many
    // as the settings now say.
    set_list(
        &api,
        &token,
        json!({ "servings": 6, "meals": 7, "sources": "plats_and_recipes" }),
    )
    .await;
    let path = format!("{ENTRIES}/{}", id(&second));
    let replacing = json!({ "recipe_id": id(&chicken) });
    let (status, body) = api
        .call(Method::PUT, &path, Some(&token), Some(replacing.clone()))
        .await;
    assert_eq!((status, body), (StatusCode::NO_CONTENT, Value::Null));
    let replaced = json!({ "id": id(&second), "servings": 6, "recipe": chicken });
    assert_eq!(
        entries(&api, &token).await,
        [first.clone(), replaced.clone(), third.clone()]
    );

    // The same again writes nothing.
    let (status, _) = api
        .call(Method::PUT, &path, Some(&token), Some(replacing))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(events(&api, &user_id).await, 4, "three added, one replaced");

    // Taken off, once.
    let path = format!("{ENTRIES}/{}", id(&first));
    let (status, body) = api.call(Method::DELETE, &path, Some(&token), None).await;
    assert_eq!((status, body), (StatusCode::NO_CONTENT, Value::Null));
    assert_eq!(entries(&api, &token).await, [replaced, third]);
    let (status, body) = api.call(Method::DELETE, &path, Some(&token), None).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, refusal("list_entry_not_found"))
    );

    api.shutdown().await;
}

#[tokio::test]
async fn a_list_is_generated_as_the_settings_say() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;

    // Nothing to draw yet.
    let (status, body) = api.post(GENERATE, Some(&token), json!({})).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            refusal("nothing_to_generate")
        )
    );

    let chicken = recipe(&api, &token, "Roast chicken").await;
    let gravy = recipe(&api, &token, "Gravy").await;
    let soup = recipe(&api, &token, "Soup").await;
    let roast = plat(&api, &token, "Sunday roast", &[&chicken, &gravy]).await;

    // By default: the plats and the recipes that are part of none, for
    // four; as many as there are when there are fewer than seven.
    let generated = generate(&api, &token).await;
    let mut meals: Vec<_> = generated.iter().map(meal).collect();
    meals.sort_unstable();
    assert_eq!(meals, [("plat", id(&roast)), ("recipe", id(&soup))]);
    for entry in &generated {
        assert_eq!(entry["servings"], 4);
        match meal(entry).0 {
            "plat" => assert_eq!(entry["plat"], roast, "the plat with its recipes"),
            _ => assert_eq!(entry["recipe"], soup),
        }
    }
    assert_eq!(entries(&api, &token).await, generated, "read back the same");

    // Only recipes, one meal, for two: in place of what the list held,
    // and another recipe each time, for as long as there is one.
    set_list(
        &api,
        &token,
        json!({ "servings": 2, "meals": 1, "sources": "recipes" }),
    )
    .await;
    let mut before = None;
    for _ in 0..6 {
        let generated = generate(&api, &token).await;
        assert_eq!(generated.len(), 1, "{generated:?}");
        assert_eq!(generated[0]["servings"], 2);
        let (kind, drawn) = meal(&generated[0]);
        assert_eq!(kind, "recipe");
        assert!([id(&chicken), id(&gravy), id(&soup)].contains(&drawn));
        assert_ne!(before.as_deref(), Some(drawn), "not the one it held");
        before = Some(drawn.to_owned());
        assert_eq!(entries(&api, &token).await, generated);
    }

    // Only plats: the one there is, whatever is asked. What was picked by
    // hand goes with the rest.
    pick(&api, &token, json!({ "recipe_id": id(&soup) })).await;
    set_list(
        &api,
        &token,
        json!({ "servings": 8, "meals": 5, "sources": "plats" }),
    )
    .await;
    let generated = generate(&api, &token).await;
    assert_eq!(generated.len(), 1, "{generated:?}");
    assert_eq!(meal(&generated[0]), ("plat", id(&roast)));
    assert_eq!(generated[0]["servings"], 8);

    // Someone with recipes and no plat has nothing to draw plats from.
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    recipe(&api, &bob, "Waffles").await;
    set_list(
        &api,
        &bob,
        json!({ "servings": 4, "meals": 7, "sources": "plats" }),
    )
    .await;
    let (status, body) = api.post(GENERATE, Some(&bob), json!({})).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            refusal("nothing_to_generate")
        )
    );

    api.shutdown().await;
}

#[tokio::test]
async fn a_deleted_recipe_or_plat_leaves_the_list() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, user_id) = register(&api, &outbox, "alice@example.com").await;
    let soup = recipe(&api, &token, "Soup").await;
    let chicken = recipe(&api, &token, "Roast chicken").await;
    let roast = plat(&api, &token, "Sunday roast", &[&chicken]).await;

    let first = pick(&api, &token, json!({ "recipe_id": id(&soup) })).await;
    let second = pick(&api, &token, json!({ "plat_id": id(&roast) })).await;
    let third = pick(&api, &token, json!({ "recipe_id": id(&chicken) })).await;
    pick(&api, &token, json!({ "recipe_id": id(&soup) })).await;

    // The recipe leaves the list, wherever it is on it; the rest stays.
    let path = format!("/v1/recipes/{}", id(&soup));
    let (status, _) = api.call(Method::DELETE, &path, Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(
        entries(&api, &token)
            .await
            .iter()
            .all(|entry| entry["id"] != first["id"]),
        "never answered once deleted"
    );
    eventually("the deleted recipe to be off the list", || async {
        (events(&api, &user_id).await == 6).then_some(())
    })
    .await;
    assert_eq!(entries(&api, &token).await, [second.clone(), third.clone()]);

    // And so does the plat. The recipe it was made of stays.
    let path = format!("/v1/plats/{}", id(&roast));
    let (status, _) = api.call(Method::DELETE, &path, Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    eventually("the deleted plat to be off the list", || async {
        (events(&api, &user_id).await == 7).then_some(())
    })
    .await;
    assert_eq!(entries(&api, &token).await, [third]);

    api.shutdown().await;
}

#[tokio::test]
async fn a_list_is_its_user_s_alone_and_made_of_what_is_theirs() {
    let (api, outbox, _node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    let soup = recipe(&api, &alice, "Soup").await;
    let roast = plat(&api, &alice, "Sunday roast", &[&soup]).await;
    let waffles = recipe(&api, &bob, "Waffles").await;
    let hers = pick(&api, &alice, json!({ "recipe_id": id(&soup) })).await;
    let path = format!("{ENTRIES}/{}", id(&hers));

    // Bob reads his own list, and does nothing to hers.
    assert_eq!(entries(&api, &bob).await, [] as [Value; 0]);
    let his = json!({ "recipe_id": id(&waffles) });
    for (method, body) in [(Method::PUT, Some(his)), (Method::DELETE, None)] {
        let (status, body) = api.call(method, &path, Some(&bob), body).await;
        assert_eq!(
            (status, body),
            (StatusCode::NOT_FOUND, refusal("list_entry_not_found"))
        );
    }

    // A meal is a recipe or a plat of the user: one of the two, theirs.
    let not_found = |field: &str, code: &str, message: &str| {
        invalid(json!([{ "field": field, "code": code, "message": message }]))
    };
    let not_a_recipe = not_found(
        "recipe_id",
        "recipe_not_found",
        "This is not one of your recipes.",
    );
    let not_a_plat = not_found(
        "plat_id",
        "plat_not_found",
        "This is not one of your plats.",
    );
    for (body, refused) in [
        (json!({ "recipe_id": id(&soup) }), not_a_recipe.clone()),
        (json!({ "recipe_id": "x".repeat(65) }), not_a_recipe.clone()),
        // A plat is no recipe, and a recipe no plat.
        (json!({ "recipe_id": id(&roast) }), not_a_recipe),
        (json!({ "plat_id": id(&roast) }), not_a_plat.clone()),
        (json!({ "plat_id": id(&waffles) }), not_a_plat),
        (
            json!({ "servings": 2 }),
            not_found("recipe_id", "required", "This field is required."),
        ),
        (
            json!({ "recipe_id": id(&waffles), "plat_id": id(&roast) }),
            not_found(
                "plat_id",
                "exclusive",
                "A meal is a recipe or a plat, not both.",
            ),
        ),
        (
            json!({ "recipe_id": id(&waffles), "servings": 0 }),
            invalid(json!([{
                "field": "servings",
                "code": "range",
                "message": "This must be between 1 and 1000.",
                "params": { "min": 1, "max": 1000 },
            }])),
        ),
    ] {
        let (status, answer) = api.post(ENTRIES, Some(&bob), body.clone()).await;
        assert_eq!(
            (status, answer),
            (StatusCode::UNPROCESSABLE_ENTITY, refused),
            "{body}"
        );
    }
    assert_eq!(entries(&api, &bob).await, [] as [Value; 0]);
    assert_eq!(entries(&api, &alice).await, std::slice::from_ref(&hers));

    // Replacing is refused the same, and leaves the entry as it was.
    let theirs = json!({ "recipe_id": id(&waffles) });
    let (status, _) = api
        .call(Method::PUT, &path, Some(&alice), Some(theirs))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(entries(&api, &alice).await, [hers]);

    // A list holds 50 meals and no more.
    for _ in 0..50 {
        pick(&api, &bob, json!({ "recipe_id": id(&waffles) })).await;
    }
    let (status, body) = api
        .post(ENTRIES, Some(&bob), json!({ "recipe_id": id(&waffles) }))
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, refusal("list_full"))
    );

    // Nobody signed in reads or writes anything.
    let body = json!({ "recipe_id": id(&soup) });
    for (method, path, body) in [
        (Method::GET, LIST, None),
        (Method::POST, GENERATE, None),
        (Method::POST, ENTRIES, Some(body.clone())),
        (Method::PUT, path.as_str(), Some(body)),
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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_list_filled_on_a_node_is_read_on_the_others() {
    let cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let (token, user_id) = register(&apis[0], &outbox, "alice@example.com").await;
    let soup = recipe(&apis[0], &token, "Soup").await;
    let stew = recipe(&apis[0], &token, "Stew").await;

    // Picked on a node, of a recipe written on another; generated on a
    // third, once its own lists show the recipes.
    let picked = pick(&apis[1], &token, json!({ "recipe_id": id(&soup) })).await;
    for api in &apis {
        eventually("the entry to be read on the node", || async {
            (entries(api, &token).await == [picked.clone()]).then_some(())
        })
        .await;
    }
    eventually("both recipes to be listed on the node", || async {
        let (_, list) = apis[2].get("/v1/recipes", Some(&token)).await;
        (list["recipes"].as_array()?.len() == 2).then_some(())
    })
    .await;
    let generated = generate(&apis[2], &token).await;
    assert_eq!(generated.len(), 2, "{generated:?}");
    for api in &apis {
        eventually("the generated list to be read on the node", || async {
            (entries(api, &token).await == generated).then_some(())
        })
        .await;
    }

    // A recipe deleted on a node leaves the list on all of them, once:
    // every node sees to it, and one of them writes it.
    let path = format!("/v1/recipes/{}", id(&stew));
    let (status, _) = apis[0]
        .call(Method::DELETE, &path, Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    for api in &apis {
        eventually("the deleted recipe to be off the list", || async {
            let left = entries(api, &token).await;
            (left.len() == 1 && meal(&left[0]) == ("recipe", id(&soup))).then_some(())
        })
        .await;
    }
    let before = events(&apis[0], &user_id).await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert_eq!(events(&apis[0], &user_id).await, before, "taken off once");

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}
