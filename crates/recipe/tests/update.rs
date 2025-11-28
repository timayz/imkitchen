use imkitchen_recipe::{
    AdvancePrepChanged, BasicInformationChanged, CuisineType, CuisineTypeChanged,
    DietaryRestriction, DietaryRestrictionsChanged, Ingredient, IngredientsChanged, Instruction,
    InstructionsChanged, MainCourseOptionsChanged, RecipeType, RecipeTypeChanged, UpdateInput,
};
use imkitchen_shared::Metadata;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_update_no_fields() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    let loaded = command.load(&recipe).await?;
    let event_id = loaded.event.id;

    assert_eq!(loaded.item.recipe_type, RecipeType::MainCourse);
    assert_eq!(loaded.item.cuisine_type, CuisineType::Caribbean);

    command.update(input.clone(), &john).await?;
    let loaded = command.load(&recipe).await?;

    assert_eq!(loaded.event.id, event_id);

    Ok(())
}

#[tokio::test]
async fn test_update_only_recipe_type() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.recipe_type = RecipeType::Dessert;

    command.update(input.clone(), &john).await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<RecipeTypeChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    assert_eq!(event.unwrap().data.recipe_type, RecipeType::Dessert);

    Ok(())
}

#[tokio::test]
async fn test_update_only_cuisine_type() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.cuisine_type = CuisineType::Italian;

    command.update(input.clone(), &john).await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<CuisineTypeChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    assert_eq!(event.unwrap().data.cuisine_type, CuisineType::Italian);

    Ok(())
}

#[tokio::test]
async fn test_update_only_name() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.name = "Updated Recipe Name".to_owned();

    command.update(input.clone(), &john).await?;

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
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.description = "Updated description".to_owned();

    command.update(input.clone(), &john).await?;

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
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.prep_time = 15;

    command.update(input.clone(), &john).await?;

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
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.cook_time = 30;

    command.update(input.clone(), &john).await?;

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
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.ingredients = vec![
        Ingredient {
            name: "ingredient 1".to_owned(),
            quantity: 2,
            unit: "g".to_owned(),
        },
        Ingredient {
            name: "ingredient 2".to_owned(),
            quantity: 100,
            unit: "ml".to_owned(),
        },
    ];

    command.update(input.clone(), &john).await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<IngredientsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.ingredients.len(), 2);
    assert_eq!(event_data.data.ingredients[0].name, "ingredient 1");
    assert_eq!(event_data.data.ingredients[0].quantity, 2);
    assert_eq!(event_data.data.ingredients[1].name, "ingredient 2");
    assert_eq!(event_data.data.ingredients[1].quantity, 100);

    Ok(())
}

#[tokio::test]
async fn test_update_only_ingredients_empty() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.ingredients = vec![];

    command.update(input.clone(), &john).await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<IngredientsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.ingredients.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_update_only_instructions() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.instructions = vec![
        Instruction {
            time_next: 20,
            description: "Updated first instruction".to_owned(),
        },
        Instruction {
            time_next: 10,
            description: "New second instruction".to_owned(),
        },
    ];

    command.update(input.clone(), &john).await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<InstructionsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(event_data.data.instructions.len(), 2);
    assert_eq!(
        event_data.data.instructions[0].description,
        "Updated first instruction"
    );
    assert_eq!(event_data.data.instructions[0].time_next, 20);
    assert_eq!(
        event_data.data.instructions[1].description,
        "New second instruction"
    );

    Ok(())
}

#[tokio::test]
async fn test_update_only_dietary_restrictions() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.dietary_restrictions = vec![
        DietaryRestriction::Vegan,
        DietaryRestriction::NutFree,
        DietaryRestriction::LowCarb,
    ];

    command.update(input.clone(), &john).await?;

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
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.accepts_accompaniment = true;

    command.update(input.clone(), &john).await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded
        .event
        .to_details::<MainCourseOptionsChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert!(event_data.data.accepts_accompaniment);

    Ok(())
}

#[tokio::test]
async fn test_update_only_advance_prep() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;

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
            unit: "g".to_owned(),
        }],
        instructions: vec![Instruction {
            time_next: 15,
            description: "My first instruction".to_owned(),
        }],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type: RecipeType::MainCourse,
        id: recipe.to_owned(),
    };

    command.update(input.clone(), &john).await?;

    input.advance_prep = "Updated advance preparation instructions".to_owned();

    command.update(input.clone(), &john).await?;

    let loaded = command.load(&recipe).await?;
    let event = loaded.event.to_details::<AdvancePrepChanged, Metadata>()?;

    assert_eq!(loaded.event.version, 8);
    let event_data = event.unwrap();
    assert_eq!(
        event_data.data.advance_prep,
        "Updated advance preparation instructions"
    );

    Ok(())
}
