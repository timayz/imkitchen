//! Importing a recipe from a page of the web, through the api's routes: the
//! recipe the page describes becomes one of the user who asked, and theirs
//! alone.

mod helpers;

use axum::http::{Method, StatusCode};
use helpers::api::{Api, invalid, refusal, register, start};
use helpers::eventually;
use imkitchen_api::recipes::PageError;
use serde_json::{Value, json};

const MARMITON: &str =
    "https://www.marmiton.org/recettes/recette_pancakes-faciles-et-rapides_81518.aspx";
/// The page at [`MARMITON`], less what a browser shows of it.
const MARMITON_PAGE: &str = include_str!("fixtures/marmiton-pancakes.html");

const TATIE_MARYSE: &str = "https://www.tatiemaryse.com/dombres-au-poulet/";
/// The page at [`TATIE_MARYSE`], less what surrounds its article.
const TATIE_MARYSE_PAGE: &str = include_str!("fixtures/tatiemaryse-dombres.html");

async fn import(api: &Api, token: &str, url: &str) -> (StatusCode, Value) {
    api.post("/v1/recipes/import", Some(token), json!({ "url": url }))
        .await
}

/// The ids of the recipes the user's list holds, once it holds `count`.
async fn listed(api: &Api, token: &str, count: usize) -> Vec<String> {
    eventually(&format!("the list to hold {count} recipes"), || async {
        let (_, list) = api.get("/v1/recipes", Some(token)).await;
        let recipes = list["recipes"].as_array()?;
        (recipes.len() == count).then(|| {
            recipes
                .iter()
                .map(|recipe| recipe["id"].as_str().unwrap().to_owned())
                .collect()
        })
    })
    .await
}

#[tokio::test]
async fn the_marmiton_pancakes_are_imported_as_a_complete_recipe() {
    let (api, outbox, node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    node.web.put(MARMITON, MARMITON_PAGE);

    // The address as it is pasted, blanks around it.
    let (status, imported) = import(&api, &token, &format!(" {MARMITON}\n")).await;
    assert_eq!(status, StatusCode::CREATED, "{imported}");
    let id = imported["id"].as_str().unwrap().to_owned();
    assert_eq!(
        imported,
        json!({
            "id": id,
            "title": "Pancakes faciles et rapides : la meilleure recette",
            "description": "Recette Pancakes faciles et rapides : la meilleure recette rapide \
                            et facile : 2 personnes, 15 min de préparation.",
            "ingredients": [
                { "name": "oeuf", "quantity": 1.0 },
                { "name": "sucre blanc", "quantity": 1.0, "unit": "cuillère à soupe" },
                { "name": "huile ou de beurre fondu", "quantity": 1.0, "unit": "cuillère à soupe" },
                { "name": "farine", "quantity": 150.0, "unit": "g" },
                { "name": "bonne cuillère à café de levure", "quantity": 1.0 },
                { "name": "lait", "quantity": 200.0, "unit": "ml" },
            ],
            "instructions": [
                { "text": "Battre grossièrement l'oeuf et le sucre dans un saladier." },
                { "text": "Y rajouter le beurre fondu ou l'huile." },
                { "text": "Mélanger à part la farine et la levure, puis en mélanger la moitié \
                           avec la préparation." },
                { "text": "Délayer progressivement avec le lait tout en rajoutant l'autre \
                           moitié de farine petit à petit (cela évitera de créer des grumeaux)." },
                { "text": "A l'aide d'une petite louche ou d'une grosse cuillère à sauce, \
                           déposer des \"ronds\" dans la poêle." },
                { "text": "Lorsque les bulles apparaissent et éclatent, retourner les pancakes \
                           et ne pas les laisser plus d'une min sur l'autre face." },
            ],
            "prep_minutes": 5,
            "cook_minutes": 10,
            "servings": 2,
            "created_at": imported["created_at"],
            "updated_at": imported["created_at"],
        })
    );

    // A recipe of the user like one they wrote: read, listed, rewritten.
    let path = format!("/v1/recipes/{id}");
    let (status, read) = api.get(&path, Some(&token)).await;
    assert_eq!((status, &read), (StatusCode::OK, &imported));
    assert_eq!(listed(&api, &token, 1).await, [id.as_str()]);
    let rewritten = json!({ "title": "Pancakes du dimanche", "servings": 4 });
    let (status, body) = api
        .call(Method::PUT, &path, Some(&token), Some(rewritten))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    api.shutdown().await;
}

#[tokio::test]
async fn a_recipe_is_imported_from_any_site_that_describes_one() {
    let (api, outbox, node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    // Microdata rather than JSON-LD, in English, and no word of the times.
    let url = "https://recipes.example.com/banana-bread";
    node.web.put(
        url,
        r#"<html><body>
           <article itemscope itemtype="https://schema.org/Recipe">
             <h1 itemprop="name">Banana bread</h1>
             <div itemprop="author" itemscope itemtype="https://schema.org/Person">
               By <span itemprop="name">Someone</span>
             </div>
             <span itemprop="recipeYield">Serves 8</span>
             <ul>
               <li itemprop="recipeIngredient">3 ripe bananas</li>
               <li itemprop="recipeIngredient">1 1/2 cups flour</li>
               <li itemprop="recipeIngredient">A pinch of salt</li>
             </ul>
             <div itemprop="recipeInstructions">
               <p>Mash the bananas.</p>
               <p>Bake for an hour.</p>
             </div>
           </article>
           </body></html>"#,
    );

    let (status, imported) = import(&api, &token, url).await;
    assert_eq!(status, StatusCode::CREATED, "{imported}");
    assert_eq!(
        imported,
        json!({
            "id": imported["id"],
            "title": "Banana bread",
            "ingredients": [
                { "name": "ripe bananas", "quantity": 3.0 },
                { "name": "flour", "quantity": 1.5, "unit": "cups" },
                { "name": "A pinch of salt" },
            ],
            "instructions": [
                { "text": "Mash the bananas." },
                { "text": "Bake for an hour." },
            ],
            "prep_minutes": 0,
            "cook_minutes": 0,
            "servings": 8,
            "created_at": imported["created_at"],
            "updated_at": imported["created_at"],
        })
    );

    api.shutdown().await;
}

#[tokio::test]
async fn a_recipe_is_imported_from_an_article_that_only_shows_it() {
    let (api, outbox, node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    // A blog: the page describes an article to machines, and shows the
    // recipe to people under its headings.
    node.web.put(TATIE_MARYSE, TATIE_MARYSE_PAGE);

    let (status, imported) = import(&api, &token, TATIE_MARYSE).await;
    assert_eq!(status, StatusCode::CREATED, "{imported}");
    let one = |name: &str, quantity: Option<f64>, unit: Option<&str>| {
        let mut ingredient = json!({ "name": name });
        if let Some(quantity) = quantity {
            ingredient["quantity"] = json!(quantity);
        }
        if let Some(unit) = unit {
            ingredient["unit"] = json!(unit);
        }
        ingredient
    };
    let steps = [
        "Citronnez les morceaux de poulet.",
        "Préparez la mixture aromatique en la hachant grossièrement.",
        "Préparez les boulettes de dombrés comme expliqué ici.",
        "Dans une cocotte, faites chauffer un filet d’huile.",
        "Une fois chaude, incorporez les morceaux de poulet.",
        "Faites colorer les morceaux sur feu vif, en déglaçant régulièrement avec le vin blanc.",
        "Au bout de 10 minutes, baissez le feu puis ajoutez la mixture aromatique, le sel et le \
         mélange bouquet garni.",
        "Ajoutez dans la foulée, la pulpe de tomate et ensuite l’eau et le gros thym.",
        "Une fois l’ébullition atteinte, ajoutez vos boulettes de dombrés au bout de 5 minutes.",
        "Baissez le feu pour maintenir une petite ébullition et laissez cuire 25 minutes.",
        "Arrêtez le feu. Pressez la dernière gousse d’ail et ciselez du persil frais.",
        "Ajoutez la cuillère à soupe de vinaigre.",
    ];
    assert_eq!(
        imported,
        json!({
            "id": imported["id"],
            "title": "Recette des DOMBRÉS au POULET – by Tatie Maryse",
            "description": "Cette recette de dombrés au poulet est une création Tatie Maryse, \
                            imaginée lors d'un \"marathon poulet\". Le résultat est 100% validé !",
            // Neither "Pour 3 personnes :" nor the names of its parts
            // ("Finition :"), nor the link under the list.
            "ingredients": [
                one("dombrés", Some(400.0), Some("g")),
                one("cuisses, soit 680 g de poulet", Some(3.0), None),
                one("Jus d’1 citron", None, None),
                one("dés de tomates", Some(342.0), Some("g")),
                one("vin blanc", Some(70.0), Some("ml")),
                one("sel", Some(7.0), Some("g")),
                one("eau", Some(450.0), Some("ml")),
                one("c à de mélange bouquet garni", Some(1.0), None),
                one("Poivre", None, None),
                one("piments végétariens", Some(2.0), None),
                one("branches de persil", Some(2.0), None),
                one("branches d’oignon pays", Some(2.0), None),
                one("gros thym", Some(2.0), Some("feuilles")),
                one("oignon soit 120g", Some(1.0), None),
                one("ail", Some(1.0), Some("gousse")),
                one("branches de persil", Some(3.0), None),
                one("vinaigre de vin", Some(1.0), Some("c à s")),
            ],
            // Up to the advice that follows the steps, under a heading of
            // its own.
            "instructions": steps.map(|text| json!({ "text": text })),
            "prep_minutes": 25,
            "cook_minutes": 30,
            "servings": 3,
            "created_at": imported["created_at"],
            "updated_at": imported["created_at"],
        })
    );

    api.shutdown().await;
}

#[tokio::test]
async fn a_page_the_client_sends_is_read_without_being_fetched() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    // Nothing answers the api at this address: a site that refuses it.
    let url = "https://captcha.example.com/pancakes";
    let send = |html: String| {
        let body = json!({ "url": url, "html": html });
        api.post("/v1/recipes/import", Some(&token), body)
    };

    let (status, imported) = send(MARMITON_PAGE.to_owned()).await;
    assert_eq!(status, StatusCode::CREATED, "{imported}");
    assert_eq!(
        imported["title"],
        "Pancakes faciles et rapides : la meilleure recette"
    );
    assert_eq!(imported["ingredients"].as_array().unwrap().len(), 6);

    // Held to what a recipe on a fetched page is held to.
    assert_eq!(
        send("<h1>Pancakes</h1><p>Mix everything.</p>".to_owned()).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            refusal("no_recipe_on_page")
        )
    );
    assert_eq!(
        send("a".repeat(4 * 1024 * 1024 + 1)).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([{
                "field": "html",
                "code": "too_long",
                "message": "This must be at most 4194304 characters.",
                "params": { "max": 4194304 },
            }]))
        )
    );
    // And the address is still one of a page.
    let body = json!({ "url": "pancakes", "html": MARMITON_PAGE });
    let (status, body) = api.post("/v1/recipes/import", Some(&token), body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["fields"][0]["code"], "invalid_url");

    api.shutdown().await;
}

#[tokio::test]
async fn a_page_without_a_recipe_is_refused_and_writes_nothing() {
    let (api, outbox, node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let refused = |code| (StatusCode::UNPROCESSABLE_ENTITY, refusal(code));

    // A page that describes no recipe, and what is not a page.
    let article = "https://news.example.com/pancakes";
    node.web.put(
        article,
        r#"<html><head><script type="application/ld+json">
           { "@context": "https://schema.org", "@type": "NewsArticle", "headline": "Pancakes" }
           </script></head><body><h1>Pancakes</h1><p>Mix everything.</p></body></html>"#,
    );
    assert_eq!(
        import(&api, &token, article).await,
        refused("no_recipe_on_page")
    );
    let picture = "https://news.example.com/pancakes.jpg";
    node.web.answer(picture, Err(PageError::NotHtml));
    assert_eq!(
        import(&api, &token, picture).await,
        refused("no_recipe_on_page")
    );

    // Nothing answers there.
    assert_eq!(
        import(&api, &token, "https://nowhere.example.com/recipe").await,
        refused("page_unreachable")
    );

    // Not the address of a page.
    for url in [
        "pancakes",
        "ftp://example.com/pancakes",
        "file:///etc/passwd",
        "",
    ] {
        assert_eq!(
            import(&api, &token, url).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                invalid(json!([{
                    "field": "url",
                    "code": "invalid_url",
                    "message": "This is not the address of a web page.",
                }]))
            ),
            "{url}"
        );
    }

    // Nobody signed in.
    node.web.put(MARMITON, MARMITON_PAGE);
    let (status, body) = api
        .post("/v1/recipes/import", None, json!({ "url": MARMITON }))
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNAUTHORIZED, refusal("sign_in_required"))
    );

    // Then one that is, after which the list holds it and nothing else.
    let (status, imported) = import(&api, &token, MARMITON).await;
    assert_eq!(status, StatusCode::CREATED, "{imported}");
    assert_eq!(
        listed(&api, &token, 1).await,
        [imported["id"].as_str().unwrap()]
    );

    api.shutdown().await;
}

#[tokio::test]
async fn each_user_who_imports_a_page_has_a_recipe_of_their_own() {
    let (api, outbox, node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    node.web.put(MARMITON, MARMITON_PAGE);

    let (status, hers) = import(&api, &alice, MARMITON).await;
    assert_eq!(status, StatusCode::CREATED, "{hers}");
    let (status, his) = import(&api, &bob, MARMITON).await;
    assert_eq!(status, StatusCode::CREATED, "{his}");
    let (her_id, his_id) = (hers["id"].as_str().unwrap(), his["id"].as_str().unwrap());
    assert_ne!(her_id, his_id);

    // Each reads and lists their own, and only it.
    assert_eq!(listed(&api, &alice, 1).await, [her_id]);
    assert_eq!(listed(&api, &bob, 1).await, [his_id]);
    let (status, body) = api
        .get(&format!("/v1/recipes/{his_id}"), Some(&alice))
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, refusal("recipe_not_found"))
    );

    // What one does with theirs leaves the other's as it was imported.
    let rewritten = json!({ "title": "Crêpes", "servings": 6 });
    let (status, _) = api
        .call(
            Method::PUT,
            &format!("/v1/recipes/{her_id}"),
            Some(&alice),
            Some(rewritten),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = api
        .call(
            Method::DELETE,
            &format!("/v1/recipes/{her_id}"),
            Some(&alice),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, read) = api.get(&format!("/v1/recipes/{his_id}"), Some(&bob)).await;
    assert_eq!((status, &read), (StatusCode::OK, &his));

    // The same page again is one more recipe, not the same one.
    let (status, again) = import(&api, &bob, MARMITON).await;
    assert_eq!(status, StatusCode::CREATED, "{again}");
    assert_ne!(again["id"], his["id"]);
    assert_eq!(listed(&api, &bob, 2).await.len(), 2);

    api.shutdown().await;
}
