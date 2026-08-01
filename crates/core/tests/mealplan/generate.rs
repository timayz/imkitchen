use evento::Sqlite;
use imkitchen_core::recipe::ImportInput;
use imkitchen_types::recipe::RecipeType;
use temp_dir::TempDir;
use time::OffsetDateTime;

#[tokio::test]
async fn test_random() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = crate::helpers::setup_test_state(path).await?;
    let cmd = imkitchen_core::mealplan::Module::new(state.clone());
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());

    for i in 0..200 {
        import_recipe(&recipe_cmd, i.to_string(), RecipeType::MainCourse, "john").await?;
    }

    for i in 0..4 {
        import_recipe(&recipe_cmd, i.to_string(), RecipeType::Appetizer, "john").await?;
    }

    for i in 0..5 {
        import_recipe(&recipe_cmd, i.to_string(), RecipeType::Appetizer, "albert").await?;
    }

    imkitchen_core::mealplan::subscription()
        .data(state.write_db.clone())
        .no_retry()
        .run_once(&state.executor)
        .await?;

    cmd.generate(imkitchen_core::mealplan::Generate {
        user_id: "john".to_owned(),
        days: 7,
        start: imkitchen_core::mealplan::date_to_u64(OffsetDateTime::now_utc()),
        randomize: Some(imkitchen_core::mealplan::Randomize {
            cuisine_variety_weight: 1.0,
            dietary_restrictions: vec![],
            recipe_types: RecipeType::default_meal_plan_types(),
        }),
        household_size: 2,
    })
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_generate_respects_recipe_types() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = crate::helpers::setup_test_state(path).await?;
    let cmd = imkitchen_core::mealplan::Module::new(state.clone());

    seed_recipes(&state).await?;

    let start = OffsetDateTime::now_utc();

    cmd.generate(imkitchen_core::mealplan::Generate {
        user_id: "john".to_owned(),
        days: 7,
        start: start.unix_timestamp() as u64,
        randomize: Some(imkitchen_core::mealplan::Randomize {
            cuisine_variety_weight: 1.0,
            dietary_restrictions: vec![],
            recipe_types: vec![
                RecipeType::Appetizer,
                RecipeType::Dessert,
                RecipeType::Beverage,
                RecipeType::Condiment,
            ],
        }),
        household_size: 2,
    })
    .await?;

    let slots = materialize_slots(&state, &cmd, start).await?;

    assert_eq!(slots.len(), 7);

    for slot in &slots {
        assert!(slot.appetizer.is_some());
        assert!(slot.dessert.is_some());
        assert!(slot.beverage.is_some());
        assert!(slot.condiment.is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_generate_without_optional_recipe_types() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = crate::helpers::setup_test_state(path).await?;
    let cmd = imkitchen_core::mealplan::Module::new(state.clone());

    seed_recipes(&state).await?;

    let start = OffsetDateTime::now_utc();

    cmd.generate(imkitchen_core::mealplan::Generate {
        user_id: "john".to_owned(),
        days: 7,
        start: start.unix_timestamp() as u64,
        randomize: Some(imkitchen_core::mealplan::Randomize {
            cuisine_variety_weight: 1.0,
            dietary_restrictions: vec![],
            recipe_types: vec![],
        }),
        household_size: 2,
    })
    .await?;

    let slots = materialize_slots(&state, &cmd, start).await?;

    assert_eq!(slots.len(), 7);

    for slot in &slots {
        assert!(slot.appetizer.is_none());
        assert!(slot.accompaniment.is_none());
        assert!(slot.dessert.is_none());
        assert!(slot.beverage.is_none());
        assert!(slot.condiment.is_none());
    }

    Ok(())
}

/// Imports one recipe of every type for `john` and runs the subscription that
/// fills the `mealplan_recipe` candidate pool.
async fn seed_recipes(state: &imkitchen_core::State<Sqlite>) -> anyhow::Result<()> {
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());

    for recipe_type in [
        RecipeType::MainCourse,
        RecipeType::Appetizer,
        RecipeType::Accompaniment,
        RecipeType::Dessert,
        RecipeType::Beverage,
        RecipeType::Condiment,
    ] {
        for i in 0..4 {
            import_recipe(
                &recipe_cmd,
                format!("{recipe_type}-{i}"),
                recipe_type.clone(),
                "john",
            )
            .await?;
        }
    }

    imkitchen_core::mealplan::subscription()
        .data(state.write_db.clone())
        .no_retry()
        .run_once(&state.executor)
        .await?;

    Ok(())
}

/// Runs the read-model subscription so `DaysGenerated` lands in `meal_plan_slot`,
/// then reads the generated week back.
async fn materialize_slots(
    state: &imkitchen_core::State<Sqlite>,
    cmd: &imkitchen_core::mealplan::Module<Sqlite>,
    start: OffsetDateTime,
) -> anyhow::Result<Vec<imkitchen_core::mealplan::slot::SlotRow>> {
    imkitchen_core::mealplan::slot::subscription()
        .data(state.write_db.clone())
        .no_retry()
        .run_once(&state.executor)
        .await?;

    cmd.range("john", start, start + time::Duration::days(6))
        .await
}

async fn import_recipe(
    cmd: &imkitchen_core::recipe::Module<Sqlite>,
    id: impl Into<String>,
    recipe_type: RecipeType,
    user_id: impl Into<String>,
) -> anyhow::Result<()> {
    let id = id.into();
    let input = ImportInput {
        name: format!("recipe {id}"),
        origin: None,
        description: "my description".to_owned(),
        advance_prep: "".to_owned(),
        ingredients: vec![],
        instructions: vec![],
        household_size: 4,
        cook_time: 25,
        prep_time: 10,
        recipe_type,
        accepts_accompaniment: false,
        dietary_restrictions: vec![],
    };

    cmd.import(input, user_id, None).await?;

    Ok(())
}
