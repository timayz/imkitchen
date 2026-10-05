use crate::helpers;
use evento::Sqlite;
use imkitchen_core::shopping::{GenerateList, Randomize};
use imkitchen_types::recipe::RecipeType;
use std::collections::HashMap;
use temp_dir::TempDir;

/// Ten mains (two accepting an accompaniment), three appetizers, three
/// desserts and two accompaniments for `john`, plus an appetizer for `albert`
/// that must never be picked. Returns id → recipe type.
async fn seed(
    state: &imkitchen_core::State<Sqlite>,
) -> anyhow::Result<HashMap<String, RecipeType>> {
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let mut types = HashMap::new();

    for i in 0..10 {
        let id = helpers::import_typed_recipe(
            &recipe_cmd,
            &format!("main-{i}"),
            &format!("main-ingredient-{i}"),
            100,
            4,
            RecipeType::MainCourse,
            i < 2,
            "john",
        )
        .await?;
        types.insert(id, RecipeType::MainCourse);
    }
    for (recipe_type, n) in [
        (RecipeType::Appetizer, 3),
        (RecipeType::Dessert, 3),
        (RecipeType::Accompaniment, 2),
    ] {
        for i in 0..n {
            let id = helpers::import_typed_recipe(
                &recipe_cmd,
                &format!("{recipe_type}-{i}"),
                &format!("{recipe_type}-ingredient-{i}"),
                100,
                4,
                recipe_type.clone(),
                false,
                "john",
            )
            .await?;
            types.insert(id, recipe_type.clone());
        }
    }
    helpers::import_typed_recipe(
        &recipe_cmd,
        "albert-appetizer",
        "albert-ingredient",
        100,
        4,
        RecipeType::Appetizer,
        false,
        "albert",
    )
    .await?;

    helpers::run_shopping_subscription(state).await?;
    helpers::run_pool_subscription(state).await?;

    Ok(types)
}

fn count(ids: &[String], types: &HashMap<String, RecipeType>, recipe_type: RecipeType) -> usize {
    ids.iter()
        .filter(|id| types.get(*id).expect("picked recipe belongs to john") == &recipe_type)
        .count()
}

#[tokio::test]
async fn test_generate_picks_mains_and_enabled_courses() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let shopping = imkitchen_core::shopping::Module::new(state.clone());
    let types = seed(&state).await?;

    shopping
        .generate(
            GenerateList {
                count: 7,
                household_size: 4,
                randomize: Some(Randomize {
                    cuisine_variety_weight: 1.0,
                    dietary_restrictions: vec![],
                    recipe_types: vec![RecipeType::Appetizer, RecipeType::Dessert],
                }),
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(count(&loaded.recipes, &types, RecipeType::MainCourse), 7);
    // ceil(7 / 2) = 4, capped by the 3 available.
    assert_eq!(count(&loaded.recipes, &types, RecipeType::Appetizer), 3);
    assert_eq!(count(&loaded.recipes, &types, RecipeType::Dessert), 3);
    assert_eq!(count(&loaded.recipes, &types, RecipeType::Accompaniment), 0);
    assert_eq!(loaded.recipes.len(), 13);
    assert!(loaded.statuses.is_empty());
    assert!(loaded.checked.is_empty());
    // One distinct ingredient per recipe.
    assert_eq!(loaded.ingredients.len(), 13);
    assert!(loaded.generated_at > 0);

    // Mains come first, in list order.
    assert!(
        loaded.recipes[..7]
            .iter()
            .all(|id| types[id] == RecipeType::MainCourse)
    );

    Ok(())
}

#[tokio::test]
async fn test_generate_without_optional_types_picks_only_mains() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let shopping = imkitchen_core::shopping::Module::new(state.clone());
    let types = seed(&state).await?;

    shopping
        .generate(
            GenerateList {
                count: 7,
                household_size: 4,
                randomize: Some(Randomize {
                    cuisine_variety_weight: 1.0,
                    dietary_restrictions: vec![],
                    recipe_types: vec![],
                }),
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes.len(), 7);
    assert!(
        loaded
            .recipes
            .iter()
            .all(|id| types[id] == RecipeType::MainCourse)
    );

    Ok(())
}

#[tokio::test]
async fn test_generate_never_repeats_when_pool_is_short() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let shopping = imkitchen_core::shopping::Module::new(state.clone());
    let types = seed(&state).await?;

    shopping
        .generate(
            GenerateList {
                count: 30,
                household_size: 4,
                randomize: Some(Randomize {
                    cuisine_variety_weight: 1.0,
                    dietary_restrictions: vec![],
                    recipe_types: vec![],
                }),
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(count(&loaded.recipes, &types, RecipeType::MainCourse), 10);
    let mut unique = loaded.recipes.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), loaded.recipes.len());

    Ok(())
}

#[tokio::test]
async fn test_generate_accompaniments_only_when_a_main_accepts_one() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let shopping = imkitchen_core::shopping::Module::new(state.clone());
    let types = seed(&state).await?;

    // With all ten mains picked, the two accepting ones are in: sides appear.
    shopping
        .generate(
            GenerateList {
                count: 10,
                household_size: 4,
                randomize: Some(Randomize {
                    cuisine_variety_weight: 1.0,
                    dietary_restrictions: vec![],
                    recipe_types: vec![RecipeType::Accompaniment],
                }),
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(count(&loaded.recipes, &types, RecipeType::MainCourse), 10);
    assert_eq!(count(&loaded.recipes, &types, RecipeType::Accompaniment), 2);

    Ok(())
}

#[tokio::test]
async fn test_generate_rejects_zero_count() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let shopping = imkitchen_core::shopping::Module::new(state.clone());
    seed(&state).await?;

    let err = shopping
        .generate(
            GenerateList {
                count: 0,
                household_size: 4,
                randomize: None,
            },
            "john",
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("count"), "{err}");

    Ok(())
}

#[tokio::test]
async fn test_generate_without_mains_fails() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let err = shopping
        .generate(
            GenerateList {
                count: 7,
                household_size: 4,
                randomize: None,
            },
            "john",
        )
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "No main course found");

    Ok(())
}
