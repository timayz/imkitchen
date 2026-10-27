use crate::helpers;
use evento::Sqlite;
use imkitchen_core::dish::ComposeInput;
use imkitchen_core::dish::query::CandidateQuery;
use imkitchen_types::recipe::RecipeType;
use temp_dir::TempDir;

struct Pool {
    main: String,
    other_main: String,
    starter: String,
    side_a: String,
    side_b: String,
    sauce: String,
}

/// John's pool: two mains, a starter, two sides and a sauce; plus a main
/// owned by Albert that John never saved.
async fn seed(state: &imkitchen_core::State<Sqlite>) -> anyhow::Result<(Pool, String)> {
    let recipe = imkitchen_core::recipe::Module::new(state.clone());
    let import = |name: &'static str, t: RecipeType| {
        let recipe = recipe.clone();
        async move { helpers::import_typed_recipe(&recipe, name, name, 100, 4, t, true, "john").await }
    };
    let pool = Pool {
        main: import("main", RecipeType::MainCourse).await?,
        other_main: import("other-main", RecipeType::MainCourse).await?,
        starter: import("starter", RecipeType::Appetizer).await?,
        side_a: import("side-a", RecipeType::Accompaniment).await?,
        side_b: import("side-b", RecipeType::Accompaniment).await?,
        sauce: import("sauce", RecipeType::Condiment).await?,
    };
    let alberts = helpers::import_typed_recipe(
        &recipe,
        "alberts-main",
        "alberts-main",
        100,
        4,
        RecipeType::MainCourse,
        false,
        "albert",
    )
    .await?;

    helpers::run_shopping_subscription(state).await?;
    helpers::run_pool_subscription(state).await?;

    Ok((pool, alberts))
}

#[tokio::test]
async fn composes_a_dish_in_meal_order_and_lists_it() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let dish = imkitchen_core::dish::Module::new(state.clone());
    let (pool, _) = seed(&state).await?;

    let id = dish
        .create(
            ComposeInput {
                name: "  Sunday roast ".to_owned(),
                recipe_ids: vec![
                    pool.sauce.clone(),
                    pool.side_b.clone(),
                    pool.main.clone(),
                    pool.side_a.clone(),
                    pool.starter.clone(),
                ],
            },
            "john",
        )
        .await?;

    let loaded = dish.load(&id).await?.expect("dish aggregate");
    assert_eq!(loaded.owner_id, "john");
    assert_eq!(loaded.name, "Sunday roast");
    assert_eq!(
        loaded.recipe_ids,
        vec![
            pool.starter.clone(),
            pool.main.clone(),
            pool.side_b.clone(),
            pool.side_a.clone(),
            pool.sauce.clone()
        ]
    );

    helpers::run_dish_subscription(&state).await?;
    let rows = dish.list("john").await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, id);
    assert_eq!(rows[0].name, "Sunday roast");
    assert_eq!(rows[0].recipe_ids.0, loaded.recipe_ids);
    assert!(dish.list("albert").await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn rejects_dishes_that_break_the_composition_rule() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let dish = imkitchen_core::dish::Module::new(state.clone());
    let (pool, alberts) = seed(&state).await?;

    let attempt = |name: &str, ids: Vec<String>| {
        let dish = dish.clone();
        let name = name.to_owned();
        async move {
            dish.create(
                ComposeInput {
                    name,
                    recipe_ids: ids,
                },
                "john",
            )
            .await
        }
    };

    // Two mains.
    assert!(matches!(
        attempt("x", vec![pool.main.clone(), pool.other_main.clone()]).await,
        Err(imkitchen_core::Error::User(_))
    ));
    // No main.
    assert!(matches!(
        attempt("x", vec![pool.side_a.clone()]).await,
        Err(imkitchen_core::Error::User(_))
    ));
    // Someone else's recipe.
    assert!(matches!(
        attempt("x", vec![alberts]).await,
        Err(imkitchen_core::Error::User(_))
    ));
    // Empty name.
    assert!(matches!(
        attempt("", vec![pool.main.clone()]).await,
        Err(imkitchen_core::Error::Validate(_))
    ));

    helpers::run_dish_subscription(&state).await?;
    assert!(dish.list("john").await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn updates_and_deletes_an_owned_dish_only() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let dish = imkitchen_core::dish::Module::new(state.clone());
    let (pool, _) = seed(&state).await?;

    let id = dish
        .create(
            ComposeInput {
                name: "Roast".to_owned(),
                recipe_ids: vec![pool.main.clone()],
            },
            "john",
        )
        .await?;

    let recompose = ComposeInput {
        name: "Roast with sides".to_owned(),
        recipe_ids: vec![pool.side_a.clone(), pool.main.clone()],
    };
    assert!(matches!(
        dish.update(&id, recompose.clone(), "albert").await,
        Err(imkitchen_core::Error::Forbidden(_))
    ));
    dish.update(&id, recompose, "john").await?;

    let loaded = dish.load(&id).await?.expect("dish aggregate");
    assert_eq!(loaded.name, "Roast with sides");
    assert_eq!(
        loaded.recipe_ids,
        vec![pool.main.clone(), pool.side_a.clone()]
    );

    helpers::run_dish_subscription(&state).await?;
    let row = dish.find(&id).await?.expect("dish row");
    assert_eq!(row.name, "Roast with sides");
    assert_eq!(row.recipe_ids.0, loaded.recipe_ids);

    assert!(matches!(
        dish.delete(&id, "albert").await,
        Err(imkitchen_core::Error::Forbidden(_))
    ));
    dish.delete(&id, "john").await?;
    assert!(dish.load(&id).await?.is_none());
    assert!(matches!(
        dish.delete(&id, "john").await,
        Err(imkitchen_core::Error::NotFound(_))
    ));

    helpers::run_dish_subscription(&state).await?;
    assert!(dish.find(&id).await?.is_none());
    assert!(dish.list("john").await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn candidates_are_searched_by_course_and_name() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let dish = imkitchen_core::dish::Module::new(state.clone());
    let (pool, _) = seed(&state).await?;

    // Everything, by course then name; Albert's main is not John's.
    let page = dish
        .candidates(
            "john",
            CandidateQuery {
                limit: 100,
                ..Default::default()
            },
        )
        .await?;
    assert_eq!(page.total, 6);
    assert_eq!(page.items.len(), 6);
    let types: Vec<String> = page
        .items
        .iter()
        .map(|c| c.recipe_type.0.to_string())
        .collect();
    let mut sorted = types.clone();
    sorted.sort();
    assert_eq!(types, sorted);

    // One course, paged, with the total of the whole match.
    let page = dish
        .candidates(
            "john",
            CandidateQuery::course(RecipeType::Accompaniment, None, 1),
        )
        .await?;
    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, pool.side_a);

    // Name search is a case-insensitive substring; `%` and `_` are literal.
    let page = dish
        .candidates(
            "john",
            CandidateQuery::course(RecipeType::Accompaniment, Some("DE-B".to_owned()), 10),
        )
        .await?;
    assert_eq!(
        page.items.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        vec![pool.side_b.as_str()]
    );
    let page = dish
        .candidates(
            "john",
            CandidateQuery::course(RecipeType::Accompaniment, Some("%".to_owned()), 10),
        )
        .await?;
    assert_eq!(page.total, 0);

    // Excluded ids (the ones already picked) are left out of the page.
    let page = dish
        .candidates(
            "john",
            CandidateQuery {
                recipe_type: Some(RecipeType::MainCourse),
                search: None,
                exclude_ids: vec![pool.main.clone()],
                limit: 10,
            },
        )
        .await?;
    assert_eq!(
        page.items.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        vec![pool.other_main.as_str()]
    );
    assert_eq!(page.total, 1);

    Ok(())
}
