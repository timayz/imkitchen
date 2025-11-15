use imkitchen_recipe::{
    AccompanimentType, AdvancePrepChanged, BasicInformationChanged, CuisineType,
    CuisineTypeChanged, DietaryRestriction, DietaryRestrictionsChanged, Ingredient,
    IngredientsChanged, Instruction, InstructionsChanged, MainCourseOptionsChanged, RecipeType,
    RecipeTypeChanged, UpdateInput,
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
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

#[tokio::test]
async fn test_update_only_name() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.name = "Updated Recipe Name".to_owned();

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded
        .event
        .to_details::<BasicInformationChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.name, "Updated Recipe Name");
    assert_eq!(event_data.data.description, "My first description");
    assert_eq!(event_data.data.prep_time, 10);
    assert_eq!(event_data.data.cook_time, 25);

    Ok(())
}

#[tokio::test]
async fn test_update_only_description() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.description = "Updated description".to_owned();

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded
        .event
        .to_details::<BasicInformationChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.name, "My first Recipe");
    assert_eq!(event_data.data.description, "Updated description");
    assert_eq!(event_data.data.prep_time, 10);
    assert_eq!(event_data.data.cook_time, 25);

    Ok(())
}

#[tokio::test]
async fn test_update_only_prep_time() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.prep_time = 15;

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded
        .event
        .to_details::<BasicInformationChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.name, "My first Recipe");
    assert_eq!(event_data.data.description, "My first description");
    assert_eq!(event_data.data.prep_time, 15);
    assert_eq!(event_data.data.cook_time, 25);

    Ok(())
}

#[tokio::test]
async fn test_update_only_cook_time() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.cook_time = 30;

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded
        .event
        .to_details::<BasicInformationChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.name, "My first Recipe");
    assert_eq!(event_data.data.description, "My first description");
    assert_eq!(event_data.data.prep_time, 10);
    assert_eq!(event_data.data.cook_time, 30);

    Ok(())
}

#[tokio::test]
async fn test_update_only_ingredients() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.ingredients = vec![
        Ingredient {
            name: "ingredient 1".to_owned(),
            unit: 2,
            unit_type: "g".to_owned(),
        },
        Ingredient {
            name: "ingredient 2".to_owned(),
            unit: 100,
            unit_type: "ml".to_owned(),
        },
    ];

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<IngredientsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.ingredients.len(), 2);
    assert_eq!(event_data.data.ingredients[0].name, "ingredient 1");
    assert_eq!(event_data.data.ingredients[0].unit, 2);
    assert_eq!(event_data.data.ingredients[1].name, "ingredient 2");
    assert_eq!(event_data.data.ingredients[1].unit, 100);

    Ok(())
}

#[tokio::test]
async fn test_update_only_ingredients_empty() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.ingredients = vec![];

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<IngredientsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.ingredients.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_update_only_instructions() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.instructions = vec![
        Instruction {
            time_before_next: 20,
            description: "Updated first instruction".to_owned(),
        },
        Instruction {
            time_before_next: 10,
            description: "New second instruction".to_owned(),
        },
    ];

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<InstructionsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.instructions.len(), 2);
    assert_eq!(
        event_data.data.instructions[0].description,
        "Updated first instruction"
    );
    assert_eq!(event_data.data.instructions[0].time_before_next, 20);
    assert_eq!(
        event_data.data.instructions[1].description,
        "New second instruction"
    );

    Ok(())
}

#[tokio::test]
async fn test_update_only_dietary_restrictions() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.dietary_restrictions = vec![
        DietaryRestriction::Vegan,
        DietaryRestriction::NutFree,
        DietaryRestriction::LowCarb,
    ];

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded
        .event
        .to_details::<DietaryRestrictionsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.dietary_restrictions.len(), 3);
    assert_eq!(
        event_data.data.dietary_restrictions[0],
        DietaryRestriction::Vegan
    );
    assert_eq!(
        event_data.data.dietary_restrictions[1],
        DietaryRestriction::NutFree
    );
    assert_eq!(
        event_data.data.dietary_restrictions[2],
        DietaryRestriction::LowCarb
    );

    Ok(())
}

#[tokio::test]
async fn test_update_only_accepts_accompaniment() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.accepts_accompaniment = true;

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded
        .event
        .to_details::<MainCourseOptionsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert!(event_data.data.accepts_accompaniment);
    assert_eq!(event_data.data.preferred_accompaniment_types.len(), 1);
    assert_eq!(
        event_data.data.preferred_accompaniment_types[0],
        AccompanimentType::Fries
    );

    Ok(())
}

#[tokio::test]
async fn test_update_only_preferred_accompaniment_types() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.preferred_accompaniment_types = vec![AccompanimentType::Rice, AccompanimentType::Salad];

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded
        .event
        .to_details::<MainCourseOptionsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert!(!event_data.data.accepts_accompaniment);
    assert_eq!(event_data.data.preferred_accompaniment_types.len(), 2);
    assert_eq!(
        event_data.data.preferred_accompaniment_types[0],
        AccompanimentType::Rice
    );
    assert_eq!(
        event_data.data.preferred_accompaniment_types[1],
        AccompanimentType::Salad
    );

    Ok(())
}

#[tokio::test]
async fn test_update_only_advance_prep() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = helpers::create_user(&state, "john").await?;

    let recipe = command.create(Metadata::by(john.to_owned())).await?;

    let mut input = UpdateInput {
        name: "My first Recipe".to_owned(),
        description: "My first description".to_owned(),
        advance_prep: "My first advance prep".to_owned(),
        dietary_restrictions: vec![
            DietaryRestriction::DairyFree,
            DietaryRestriction::GlutenFree,
        ],
        preferred_accompaniment_types: vec![AccompanimentType::Fries],
        accepts_accompaniment: false,
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

    input.advance_prep = "Updated advance preparation instructions".to_owned();

    command
        .update(input.clone(), Metadata::by(john.to_owned()))
        .await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<AdvancePrepChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(
        event_data.data.description,
        "Updated advance preparation instructions"
    );

    Ok(())
}
