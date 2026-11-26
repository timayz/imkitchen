//! The plats a user composes of their recipes, through the api's routes:
//! composed, read, recomposed and deleted by their owner, and by nobody
//! else, and rid of the recipes that are deleted.

mod helpers;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use helpers::api::{Api, Outbox, invalid, refusal, register, start};
use helpers::{TestCluster, eventually};
use imkitchen_aggregates::plat::{Created, Plat};
use serde_json::{Value, json};

/// Writes a recipe of the user `token` signs in, titled `title`, and
/// answers it as a list shows it, once the list of the recipes does.
async fn recipe(api: &Api, token: &str, title: &str) -> Value {
    let body = json!({ "title": title, "servings": 4, "prep_minutes": 10 });
    let (status, body) = api.post("/v1/recipes", Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = body["id"].as_str().unwrap().to_owned();
    eventually(&format!("the recipe {title} to be listed"), || async {
        let (_, list) = api.get("/v1/recipes?limit=100", Some(token)).await;
        let listed = list["recipes"].as_array()?;
        listed.iter().find(|recipe| recipe["id"] == id).cloned()
    })
    .await
}

fn id(thing: &Value) -> &str {
    thing["id"].as_str().unwrap()
}

fn composition(name: &str, recipes: &[&Value]) -> Value {
    let recipe_ids: Vec<&str> = recipes.iter().map(|recipe| id(recipe)).collect();
    json!({ "name": name, "recipe_ids": recipe_ids })
}

/// Composes a plat of the user `token` signs in and answers its id.
async fn compose(api: &Api, token: &str, name: &str, recipes: &[&Value]) -> String {
    let body = composition(name, recipes);
    let (status, body) = api.post("/v1/plats", Some(token), body).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    id(&body).to_owned()
}

/// The list at `path`, as the api answers it now.
async fn list(api: &Api, token: &str, path: &str) -> Value {
    let (status, body) = api.get(path, Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// What a list answers of each plat: its name and the titles of its
/// recipes, in its order.
fn composed(list: &Value) -> Vec<(String, Vec<String>)> {
    let plats = list["plats"].as_array().unwrap();
    plats.iter().map(named).collect()
}

/// The name of a plat and the titles of its recipes, in order.
fn named(plat: &Value) -> (String, Vec<String>) {
    let recipes = plat["recipes"].as_array().unwrap();
    (
        plat["name"].as_str().unwrap().to_owned(),
        recipes
            .iter()
            .map(|recipe| recipe["title"].as_str().unwrap().to_owned())
            .collect(),
    )
}

/// The list at `path`, once it holds the plats `expected`, in any order,
/// each with its recipes in theirs: it follows the commands a moment later.
async fn listed(api: &Api, token: &str, path: &str, expected: &[(&str, &[&str])]) -> Value {
    let mut expected: Vec<(String, Vec<String>)> = expected
        .iter()
        .map(|(name, titles)| {
            let titles = titles.iter().map(|title| (*title).to_owned()).collect();
            ((*name).to_owned(), titles)
        })
        .collect();
    expected.sort_unstable();
    eventually(&format!("{path} to list {expected:?}"), || async {
        let list = list(api, token, path).await;
        let mut composed = composed(&list);
        composed.sort_unstable();
        (composed == expected).then_some(list)
    })
    .await
}

/// The plat at `path`, once it is made of the recipes titled `expected`.
async fn made_of(api: &Api, token: &str, path: &str, expected: &[&str]) -> Value {
    eventually(&format!("{path} to be made of {expected:?}"), || async {
        let (status, plat) = api.get(path, Some(token)).await;
        assert_eq!(status, StatusCode::OK, "{plat}");
        (named(&plat).1 == expected).then_some(plat)
    })
    .await
}

#[tokio::test]
async fn a_user_composes_reads_recomposes_and_deletes_a_plat() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let roast = recipe(&api, &token, "Roast chicken").await;
    let potatoes = recipe(&api, &token, "Potatoes").await;
    let gravy = recipe(&api, &token, "Gravy").await;

    let mut composing = composition("  Sunday roast ", &[&roast, &potatoes]);
    let (status, created) = api.post("/v1/plats", Some(&token), composing.take()).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = id(&created).to_owned();
    let created_at = created["created_at"].as_u64().unwrap();
    assert!(created_at > 0);
    assert_eq!(
        created,
        json!({
            "id": id,
            "name": "Sunday roast",
            "recipes": [roast, potatoes],
            "created_at": created_at,
            "updated_at": created_at,
        }),
        "as composed, without the blanks, each recipe as its list shows it"
    );

    // Read back at once, and listed a moment later, the same.
    let path = format!("/v1/plats/{id}");
    let (status, read) = api.get(&path, Some(&token)).await;
    assert_eq!((status, &read), (StatusCode::OK, &created));
    let expected: &[&str] = &["Roast chicken", "Potatoes"];
    let list = listed(&api, &token, "/v1/plats", &[("Sunday roast", expected)]).await;
    assert_eq!(list, json!({ "plats": [created] }));

    // Recomposed as a whole: its name, and its recipes in the order given.
    let recomposed = composition("Roast", &[&gravy, &roast]);
    let (status, body) = api
        .call(Method::PUT, &path, Some(&token), Some(recomposed.clone()))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, read) = api.get(&path, Some(&token)).await;
    assert_eq!(
        read,
        json!({
            "id": id,
            "name": "Roast",
            "recipes": [gravy, roast],
            "created_at": created_at,
            "updated_at": read["updated_at"],
        })
    );
    let expected: &[&str] = &["Gravy", "Roast chicken"];
    listed(&api, &token, "/v1/plats", &[("Roast", expected)]).await;

    // The same again writes nothing.
    let (status, _) = api
        .call(Method::PUT, &path, Some(&token), Some(recomposed.clone()))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let events = evento::read::<Plat>(&id)
        .execute(&api.state.executor)
        .await
        .unwrap();
    assert_eq!(events.len(), 2, "created, edited once");

    // A plat holds nothing of its recipes: it shows each as it now stands.
    let rewritten = json!({ "title": "Jus", "servings": 2 });
    let gravy_path = format!("/v1/recipes/{}", gravy["id"].as_str().unwrap());
    let (status, _) = api
        .call(Method::PUT, &gravy_path, Some(&token), Some(rewritten))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let read = made_of(&api, &token, &path, &["Jus", "Roast chicken"]).await;
    assert_eq!(read["recipes"][0]["servings"], 2);
    let expected: &[&str] = &["Jus", "Roast chicken"];
    listed(&api, &token, "/v1/plats", &[("Roast", expected)]).await;

    let (status, _) = api.call(Method::DELETE, &path, Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    for (method, body) in [
        (Method::GET, None),
        (Method::PUT, Some(recomposed)),
        (Method::DELETE, None),
    ] {
        let (status, body) = api.call(method, &path, Some(&token), body).await;
        assert_eq!(
            (status, body),
            (StatusCode::NOT_FOUND, refusal("plat_not_found"))
        );
    }
    listed(&api, &token, "/v1/plats", &[]).await;

    // The recipes it was made of stay.
    let (_, recipes) = api.get("/v1/recipes", Some(&token)).await;
    assert_eq!(recipes["recipes"].as_array().unwrap().len(), 3);

    api.shutdown().await;
}

#[tokio::test]
async fn a_plat_is_its_owners_alone_and_so_are_its_recipes() {
    let (api, outbox, _node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    let roast = recipe(&api, &alice, "Roast chicken").await;
    let waffles = recipe(&api, &bob, "Waffles").await;
    let id = compose(&api, &alice, "Sunday roast", &[&roast]).await;
    let path = format!("/v1/plats/{id}");
    let (_, composed) = api.get(&path, Some(&alice)).await;

    // To anyone else it does not exist, read or written.
    for (method, body) in [
        (Method::GET, None),
        (Method::PUT, Some(composition("Mine now", &[&waffles]))),
        (Method::DELETE, None),
    ] {
        let (status, body) = api.call(method.clone(), &path, Some(&bob), body).await;
        assert_eq!(
            (status, body),
            (StatusCode::NOT_FOUND, refusal("plat_not_found")),
            "{method}"
        );
    }
    let (status, read) = api.get(&path, Some(&alice)).await;
    assert_eq!((status, read), (StatusCode::OK, composed), "untouched");

    // Nobody composes a plat of someone else's recipes, or of none that
    // exists: each is said where it is in the body.
    let not_theirs = invalid(json!([{
        "field": "recipe_ids[1]",
        "code": "recipe_not_found",
        "message": "This is not one of your recipes.",
    }]));
    let nowhere = json!({ "id": "01JDQ0B6V5K1X5C3E8W4T9Z2YH" });
    for other in [&roast, &nowhere] {
        let body = composition("Brunch", &[&waffles, other]);
        let (status, answer) = api.post("/v1/plats", Some(&bob), body.clone()).await;
        assert_eq!(
            (status, &answer),
            (StatusCode::UNPROCESSABLE_ENTITY, &not_theirs)
        );
    }
    let brunch = compose(&api, &bob, "Brunch", &[&waffles]).await;
    let (status, answer) = api
        .call(
            Method::PUT,
            &format!("/v1/plats/{brunch}"),
            Some(&bob),
            Some(composition("Brunch", &[&waffles, &roast])),
        )
        .await;
    assert_eq!(
        (status, answer),
        (StatusCode::UNPROCESSABLE_ENTITY, not_theirs)
    );

    // Each lists their own, and a search finds nothing of the other's.
    let expected: &[&str] = &["Roast chicken"];
    listed(&api, &alice, "/v1/plats", &[("Sunday roast", expected)]).await;
    let expected: &[&str] = &["Waffles"];
    listed(&api, &bob, "/v1/plats", &[("Brunch", expected)]).await;
    let found = list(&api, &bob, "/v1/plats?q=sunday").await;
    assert_eq!(found, json!({ "plats": [] }));

    // Nobody signed in reads or writes anything.
    let body = composition("Sunday roast", &[&roast]);
    for (method, path, body) in [
        (Method::POST, "/v1/plats", Some(body.clone())),
        (Method::GET, "/v1/plats", None),
        (Method::GET, path.as_str(), None),
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

#[tokio::test]
async fn a_deleted_recipe_leaves_the_plats_it_was_part_of() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let roast = recipe(&api, &token, "Roast chicken").await;
    let potatoes = recipe(&api, &token, "Potatoes").await;
    let gravy = recipe(&api, &token, "Gravy").await;
    let sunday = compose(&api, &token, "Sunday roast", &[&roast, &potatoes, &gravy]).await;
    let side = compose(&api, &token, "Side", &[&potatoes]).await;
    let sauce = compose(&api, &token, "Sauce", &[&gravy]).await;
    let all: &[&str] = &["Roast chicken", "Potatoes", "Gravy"];
    let listing = [
        ("Sunday roast", all),
        ("Side", &["Potatoes"]),
        ("Sauce", &["Gravy"]),
    ];
    listed(&api, &token, "/v1/plats", &listing).await;

    let potatoes_path = format!("/v1/recipes/{}", id(&potatoes));
    let (status, _) = api
        .call(Method::DELETE, &potatoes_path, Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Out of every plat it was part of, the others as they were; a plat
    // left with no recipe stays.
    let rest: &[&str] = &["Roast chicken", "Gravy"];
    let listing = [("Sunday roast", rest), ("Side", &[]), ("Sauce", &["Gravy"])];
    listed(&api, &token, "/v1/plats", &listing).await;
    made_of(&api, &token, &format!("/v1/plats/{sunday}"), rest).await;
    let side_path = format!("/v1/plats/{side}");
    made_of(&api, &token, &side_path, &[]).await;

    // Said by the plats themselves, once each.
    for (plat, events) in [(&sunday, 2), (&side, 2), (&sauce, 1)] {
        let stream = evento::read::<Plat>(plat)
            .execute(&api.state.executor)
            .await
            .unwrap();
        assert_eq!(stream.len(), events, "{plat}");
    }

    // The plat left with none is recomposed like any other, though not of
    // the recipe that is gone.
    let (status, _) = api
        .call(
            Method::PUT,
            &side_path,
            Some(&token),
            Some(composition("Side", &[&potatoes])),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = api
        .call(
            Method::PUT,
            &side_path,
            Some(&token),
            Some(composition("Side", &[&gravy])),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    made_of(&api, &token, &side_path, &["Gravy"]).await;

    api.shutdown().await;
}

#[tokio::test]
async fn a_plat_composed_of_a_recipe_being_deleted_is_rid_of_it_too() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, owner_id) = register(&api, &outbox, "alice@example.com").await;
    let roast = recipe(&api, &token, "Roast chicken").await;
    let potatoes = recipe(&api, &token, "Potatoes").await;
    let potatoes_path = format!("/v1/recipes/{}", id(&potatoes));
    let (status, _) = api
        .call(Method::DELETE, &potatoes_path, Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // What a command that checked the recipe a moment before its deletion
    // writes after it: the deletion came by with no plat to take it out of.
    let mut write = evento::create();
    write.event(&Created {
        owner_id,
        name: "Sunday roast".to_owned(),
        recipe_ids: vec![id(&roast).to_owned(), id(&potatoes).to_owned()],
    });
    let plat = write.commit(&api.state.executor).await.unwrap();

    let expected: &[&str] = &["Roast chicken"];
    listed(&api, &token, "/v1/plats", &[("Sunday roast", expected)]).await;
    eventually("the plat to say the recipe is out of it", || async {
        let stream = evento::read::<Plat>(&plat)
            .execute(&api.state.executor)
            .await
            .unwrap();
        (stream.len() == 2).then_some(())
    })
    .await;

    api.shutdown().await;
}

#[tokio::test]
async fn what_a_plat_does_not_hold_is_said_field_by_field() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let roast = recipe(&api, &token, "Roast chicken").await;

    let count = json!({
        "field": "recipe_ids",
        "code": "count",
        "message": "There must be between 1 and 20 of these.",
        "params": { "min": 1, "max": 20 },
    });
    let (status, body) = api
        .post(
            "/v1/plats",
            Some(&token),
            json!({ "name": "   ", "recipe_ids": [] }),
        )
        .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([
                {
                    "field": "name",
                    "code": "length",
                    "message": "This must be between 1 and 200 characters.",
                    "params": { "min": 1, "max": 200 },
                },
                count,
            ]))
        )
    );

    let many: Vec<String> = (0..21).map(|nth| format!("recipe-{nth}")).collect();
    let (status, body) = api
        .post(
            "/v1/plats",
            Some(&token),
            json!({ "name": "Feast", "recipe_ids": many }),
        )
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, invalid(json!([count])))
    );

    let (status, body) = api
        .post(
            "/v1/plats",
            Some(&token),
            composition("Twice", &[&roast, &roast]),
        )
        .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([{
                "field": "recipe_ids",
                "code": "duplicate",
                "message": "This is there more than once.",
            }]))
        )
    );

    // An id no recipe has, however long.
    let (status, body) = api
        .post(
            "/v1/plats",
            Some(&token),
            json!({ "name": "Roast", "recipe_ids": [roast["id"], "x".repeat(1000), ""] }),
        )
        .await;
    let unknown = |nth: u32| {
        json!({
            "field": format!("recipe_ids[{nth}]"),
            "code": "recipe_not_found",
            "message": "This is not one of your recipes.",
        })
    };
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([unknown(1), unknown(2)]))
        )
    );

    // A body of another shape: no recipes.
    let (status, body) = api
        .post("/v1/plats", Some(&token), json!({ "name": "Roast" }))
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, refusal("invalid_request"))
    );

    // The list's parameters are held to theirs.
    let (status, body) = api
        .get("/v1/plats?limit=0&after=nowhere", Some(&token))
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

    assert_eq!(
        list(&api, &token, "/v1/plats").await,
        json!({ "plats": [] }),
        "nothing was composed"
    );

    api.shutdown().await;
}

#[tokio::test]
async fn the_list_is_read_a_page_at_a_time_and_searched() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let roast = recipe(&api, &token, "Roast chicken").await;
    let all = [
        "Sunday roast",
        "Crème brûlée",
        "Brunch",
        "Pique-nique",
        "50% rye",
    ];
    let expected: &[&str] = &["Roast chicken"];
    let mut listing = Vec::new();
    for name in all {
        compose(&api, &token, name, &[&roast]).await;
        listing.push((name, expected));
    }
    let whole = listed(&api, &token, "/v1/plats?limit=100", &listing).await;
    assert!(whole["next"].is_null(), "{whole}");
    let ordered: Vec<String> = composed(&whole).into_iter().map(|(name, _)| name).collect();

    // Two at a time, each page from where the one before stopped.
    let mut paged = Vec::new();
    let mut path = "/v1/plats?limit=2".to_owned();
    for size in [2, 2, 1] {
        let page = list(&api, &token, &path).await;
        let on_page: Vec<String> = composed(&page).into_iter().map(|(name, _)| name).collect();
        assert_eq!(on_page.len(), size, "{page}");
        paged.extend(on_page);
        match page["next"].as_str() {
            Some(next) => path = format!("/v1/plats?limit=2&after={next}"),
            None => assert_eq!(paged.len(), all.len(), "the last page says so"),
        }
    }
    assert_eq!(paged, ordered);

    // The name, whatever the case and the accents; not its recipes.
    for (q, found) in [
        ("creme", vec!["Crème brûlée"]),
        ("BRÛLÉE", vec!["Crème brûlée"]),
        (
            "r",
            vec!["Sunday roast", "Crème brûlée", "Brunch", "50% rye"],
        ),
        // Nothing of what was typed is a wildcard.
        ("%25", vec!["50% rye"]),
        ("_", vec![]),
        ("chicken", vec![]),
    ] {
        let mut found = found;
        found.sort_unstable();
        let list = list(&api, &token, &format!("/v1/plats?q={q}")).await;
        let mut names: Vec<String> = composed(&list).into_iter().map(|(name, _)| name).collect();
        names.sort_unstable();
        assert_eq!(names, found, "{q}");
    }

    api.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_plat_composed_on_a_node_is_read_on_the_others() {
    let cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let (token, _) = register(&apis[0], &outbox, "alice@example.com").await;
    let roast = recipe(&apis[0], &token, "Roast chicken").await;
    let potatoes = recipe(&apis[0], &token, "Potatoes").await;

    // Composed on a node of recipes written on another.
    let plat = compose(&apis[1], &token, "Sunday roast", &[&roast, &potatoes]).await;

    // The plat itself at once, whichever node is asked; each node's list
    // once its own subscription got there.
    let path = format!("/v1/plats/{plat}");
    let both: &[&str] = &["Roast chicken", "Potatoes"];
    for api in &apis {
        let (status, read) = api.get(&path, Some(&token)).await;
        assert_eq!(status, StatusCode::OK, "{read}");
        assert_eq!(named(&read).1, both);
        listed(api, &token, "/v1/plats", &[("Sunday roast", both)]).await;
    }

    // A recipe deleted on a node leaves the plat on all of them, once:
    // every node sees to it, and one of them writes it.
    let potatoes_path = format!("/v1/recipes/{}", id(&potatoes));
    let (status, _) = apis[2]
        .call(Method::DELETE, &potatoes_path, Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let rest: &[&str] = &["Roast chicken"];
    for api in &apis {
        listed(api, &token, "/v1/plats", &[("Sunday roast", rest)]).await;
        made_of(api, &token, &path, rest).await;
    }
    let stream = evento::read::<Plat>(&plat)
        .execute(&apis[0].state.executor)
        .await
        .unwrap();
    assert_eq!(stream.len(), 2, "created, one recipe removed once");

    let (status, _) = apis[0]
        .call(Method::DELETE, &path, Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    for api in &apis {
        let (status, _) = api.get(&path, Some(&token)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        listed(api, &token, "/v1/plats", &[]).await;
    }

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}
