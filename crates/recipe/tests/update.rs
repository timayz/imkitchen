use imkitchen_recipe::UpdateInput;
use imkitchen_shared::recipe::{
    CuisineType, DietaryRestriction, Ingredient, IngredientUnit, Instruction, RecipeType,
};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_update_no_fields() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;

    let recipe_id =
        imkitchen_recipe::Command::create(&state.evento, "john", "john_doe".to_owned()).await?;

    let input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        accepts_accompaniment: false,
        ingredients: vec![Ingredient {
            name: "ingredient 1".to_owned(),
            quantity: 1,
            unit: Some(IngredientUnit::G),
            category: None,
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        household_size: 4,
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe_id.to_owned(),
    };

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    recipe.update(input.clone(), "john").await?;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();

    assert_eq!(recipe.recipe_type.0, RecipeType::MainCourse);
    assert_eq!(recipe.cuisine_type.0, CuisineType::Caribbean);

    // Update with same values should not change anything
    recipe.update(input.clone(), "john").await?;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();

    assert_eq!(recipe.recipe_type.0, RecipeType::MainCourse);
    assert_eq!(recipe.cuisine_type.0, CuisineType::Caribbean);

    Ok(())
}

#[tokio::test]
async fn test_update_only_recipe_type() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;

    let recipe_id =
        imkitchen_recipe::Command::create(&state.evento, "john", "john_doe".to_owned()).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        accepts_accompaniment: false,
        ingredients: vec![Ingredient {
            name: "ingredient 1".to_owned(),
            quantity: 1,
            unit: Some(IngredientUnit::G),
            category: None,
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        household_size: 4,
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe_id.to_owned(),
    };

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    recipe.update(input.clone(), "john").await?;

    input.recipe_type = RecipeType::Dessert;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    recipe.update(input.clone(), "john").await?;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();

    assert_eq!(recipe.recipe_type.0, RecipeType::Dessert);

    Ok(())
}

#[tokio::test]
async fn test_update_only_cuisine_type() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;

    let recipe_id =
        imkitchen_recipe::Command::create(&state.evento, "john", "john_doe".to_owned()).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        accepts_accompaniment: false,
        ingredients: vec![Ingredient {
            name: "ingredient 1".to_owned(),
            quantity: 1,
            unit: Some(IngredientUnit::G),
            category: None,
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        household_size: 4,
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe_id.to_owned(),
    };

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    recipe.update(input.clone(), "john").await?;

    input.cuisine_type = CuisineType::Italian;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    recipe.update(input.clone(), "john").await?;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();

    assert_eq!(recipe.cuisine_type.0, CuisineType::Italian);

    Ok(())
}

#[tokio::test]
async fn test_update_only_accepts_accompaniment() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;

    let recipe_id =
        imkitchen_recipe::Command::create(&state.evento, "john", "john_doe".to_owned()).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        accepts_accompaniment: false,
        ingredients: vec![Ingredient {
            name: "ingredient 1".to_owned(),
            quantity: 1,
            unit: Some(IngredientUnit::G),
            category: None,
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        household_size: 4,
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe_id.to_owned(),
    };

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    recipe.update(input.clone(), "john").await?;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    assert!(!recipe.accepts_accompaniment);

    input.accepts_accompaniment = true;

    recipe.update(input.clone(), "john").await?;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();

    assert!(recipe.accepts_accompaniment);

    Ok(())
}
