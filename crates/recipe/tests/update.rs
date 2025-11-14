use imkitchen_recipe::{
    AccompanimentType, CuisineType, CuisineTypeChanged, DietaryRestriction, Ingredient,
    Instruction, RecipeType, RecipeTypeChanged, UpdateInput,
};
use imkitchen_shared::Metadata;

mod helpers;

#[tokio::test]
async fn test_update_no_fields() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_preparation: "My first advance_preparation".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accept_accompaniments: false,
        ingredients: vec![Ingredient {
            name: "ingredient 1".to_owned(),
            unit: 1,
            unit_type: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_before_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event_id = loaded.event.id;

    assert_eq!(loaded.item.recipe_type, RecipeType::MainCourse);
    assert_eq!(loaded.item.cuisine_type, CuisineType::Caribbean);

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;
    let loaded = command.load(&recipe).await?;

    assert_eq!(loaded.event.id, event_id);

    Ok(())
}

#[tokio::test]
async fn test_update_only_recipe_type() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_preparation: "My first advance_preparation".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accept_accompaniments: false,
        ingredients: vec![Ingredient {
            name: "ingredient 1".to_owned(),
            unit: 1,
            unit_type: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_before_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    input.recipe_type = RecipeType::Dessert;

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<RecipeTypeChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    assert_eq!(event.unwrap().data.recipe_type, RecipeType::Dessert);

    Ok(())
}

#[tokio::test]
async fn test_update_only_cuisine_type() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_preparation: "My first advance_preparation".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accept_accompaniments: false,
        ingredients: vec![Ingredient {
            name: "ingredient 1".to_owned(),
            unit: 1,
            unit_type: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_before_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    input.cuisine_type = CuisineType::Italian;

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<CuisineTypeChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    assert_eq!(event.unwrap().data.cuisine_type, CuisineType::Italian);

    Ok(())
}
