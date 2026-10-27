use crate::helpers;
use evento::Sqlite;
use imkitchen_core::dish::ComposeInput;
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
    Ok(seed_with_accepting(state).await?.0)
}

/// Like [`seed`], also returning the ids of the two mains that accept an
/// accompaniment.
async fn seed_with_accepting(
    state: &imkitchen_core::State<Sqlite>,
) -> anyhow::Result<(HashMap<String, RecipeType>, Vec<String>)> {
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let mut types = HashMap::new();
    let mut accepting = vec![];

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
        if i < 2 {
            accepting.push(id.clone());
        }
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

    Ok((types, accepting))
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
    // Every meal gets one of each enabled course until that pool runs out.
    assert_eq!(count(&loaded.recipes, &types, RecipeType::Appetizer), 3);
    assert_eq!(count(&loaded.recipes, &types, RecipeType::Dessert), 3);
    assert_eq!(count(&loaded.recipes, &types, RecipeType::Accompaniment), 0);
    assert_eq!(loaded.recipes.len(), 13);
    assert!(loaded.statuses.is_empty());
    assert!(loaded.checked.is_empty());
    // One distinct ingredient per recipe.
    assert_eq!(loaded.ingredients.len(), 13);
    assert!(loaded.generated_at > 0);

    // Meal order: the first three meals are starter, main, dessert; the
    // remaining four are mains on their own.
    let kinds: Vec<&RecipeType> = loaded.recipes.iter().map(|id| &types[id]).collect();
    for meal in 0..3 {
        assert_eq!(kinds[meal * 3], &RecipeType::Appetizer, "meal {meal}");
        assert_eq!(kinds[meal * 3 + 1], &RecipeType::MainCourse, "meal {meal}");
        assert_eq!(kinds[meal * 3 + 2], &RecipeType::Dessert, "meal {meal}");
    }
    assert!(kinds[9..].iter().all(|k| *k == &RecipeType::MainCourse));

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
    let (types, accepting) = seed_with_accepting(&state).await?;

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
    // Each side directly follows the main it was paired with, and that main
    // accepts accompaniments.
    for (i, id) in loaded.recipes.iter().enumerate() {
        if types[id] == RecipeType::Accompaniment {
            let main = &loaded.recipes[i - 1];
            assert_eq!(types[main], RecipeType::MainCourse);
            assert!(
                accepting.contains(main),
                "side paired with a non-accepting main"
            );
        }
    }

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

/// Two dishes (one with two sides and a sauce, one a bare main) among the
/// seeded pool: they come first, keep their composition, and the random
/// pools never hand out a recipe a dish already placed.
#[tokio::test]
async fn test_generate_uses_dishes_first_and_fills_their_missing_courses() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());
    let dish = imkitchen_core::dish::Module::new(state.clone());
    let (types, accepting) = seed_with_accepting(&state).await?;

    let of = |t: RecipeType| {
        types
            .iter()
            .filter(|(_, rt)| **rt == t)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>()
    };
    let sides = of(RecipeType::Accompaniment);
    let sauce = helpers::import_typed_recipe(
        &recipe_cmd,
        "sauce",
        "sauce-ingredient",
        10,
        4,
        RecipeType::Condiment,
        false,
        "john",
    )
    .await?;
    helpers::run_shopping_subscription(&state).await?;
    helpers::run_pool_subscription(&state).await?;

    // Dish A: an accepting main with both sides and the sauce.
    let dish_a = dish
        .create(
            ComposeInput {
                name: "Full plate".to_owned(),
                recipe_ids: vec![
                    accepting[0].clone(),
                    sides[0].clone(),
                    sides[1].clone(),
                    sauce.clone(),
                ],
            },
            "john",
        )
        .await?;
    // Dish B: the other accepting main alone.
    let dish_b = dish
        .create(
            ComposeInput {
                name: "Bare main".to_owned(),
                recipe_ids: vec![accepting[1].clone()],
            },
            "john",
        )
        .await?;
    helpers::run_dish_subscription(&state).await?;

    shopping
        .generate(
            GenerateList {
                count: 3,
                household_size: 4,
                randomize: Some(Randomize {
                    cuisine_variety_weight: 1.0,
                    dietary_restrictions: vec![],
                    recipe_types: vec![
                        RecipeType::Accompaniment,
                        RecipeType::Dessert,
                        RecipeType::Condiment,
                    ],
                }),
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    let ids = &loaded.recipes;

    // Both dishes are in, one of them first, then one random main.
    let dish_a_ids = dish.load(&dish_a).await?.unwrap().recipe_ids;
    let dish_b_ids = dish.load(&dish_b).await?.unwrap().recipe_ids;
    let pos = |id: &String| ids.iter().position(|x| x == id).expect("recipe in list");
    let a_main = pos(&accepting[0]);
    let b_main = pos(&accepting[1]);
    // Dish A reads main, side, side, dessert, sauce; nothing else in between.
    assert_eq!(ids[a_main + 1], sides[0]);
    assert_eq!(ids[a_main + 2], sides[1]);
    assert_eq!(types[&ids[a_main + 3]], RecipeType::Dessert);
    assert_eq!(ids[a_main + 4], sauce);
    assert_eq!(dish_a_ids.len(), 4);
    // Dish B is filled: its main accepts a side, but both sides went to dish
    // A, so it gets a dessert and nothing else beside the main.
    assert_eq!(types[&ids[b_main + 1]], RecipeType::Dessert);
    assert_eq!(dish_b_ids, vec![accepting[1].clone()]);

    let mains: Vec<&String> = ids
        .iter()
        .filter(|id| types.get(*id) == Some(&RecipeType::MainCourse))
        .collect();
    assert_eq!(mains.len(), 3);
    // Sides were placed by dish A and never drawn again.
    assert_eq!(ids.iter().filter(|id| sides.contains(id)).count(), 2);
    assert_eq!(ids.iter().filter(|id| *id == &sauce).count(), 1);
    // Three desserts, one per meal.
    assert_eq!(
        ids.iter()
            .filter(|id| types.get(*id) == Some(&RecipeType::Dessert))
            .count(),
        3
    );
    assert_eq!(ids.len(), 3 + 2 + 1 + 3);

    Ok(())
}

/// A dish whose main was deleted is skipped; one whose side was deleted
/// keeps going without it.
#[tokio::test]
async fn test_generate_skips_dishes_without_their_main() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());
    let dish = imkitchen_core::dish::Module::new(state.clone());
    let (types, accepting) = seed_with_accepting(&state).await?;
    let sides: Vec<String> = types
        .iter()
        .filter(|(_, rt)| **rt == RecipeType::Accompaniment)
        .map(|(id, _)| id.clone())
        .collect();

    let gone = dish
        .create(
            ComposeInput {
                name: "Gone".to_owned(),
                recipe_ids: vec![accepting[0].clone(), sides[0].clone()],
            },
            "john",
        )
        .await?;
    let kept = dish
        .create(
            ComposeInput {
                name: "Kept".to_owned(),
                recipe_ids: vec![accepting[1].clone(), sides[1].clone()],
            },
            "john",
        )
        .await?;
    helpers::run_dish_subscription(&state).await?;

    recipe_cmd.delete(&accepting[0], "john").await?;
    recipe_cmd.delete(&sides[1], "john").await?;
    helpers::run_shopping_subscription(&state).await?;
    helpers::run_pool_subscription(&state).await?;

    shopping
        .generate(
            GenerateList {
                count: 1,
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
    assert_eq!(loaded.recipes, vec![accepting[1].clone()]);
    assert!(
        dish.load(&gone).await?.is_some(),
        "the dish itself survives"
    );
    assert!(dish.load(&kept).await?.is_some());

    Ok(())
}
