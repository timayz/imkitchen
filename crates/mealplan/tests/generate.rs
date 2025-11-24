use imkitchen_mealplan::Status;
use imkitchen_recipe::{CuisineType, ImportInput, RecipeType};
use imkitchen_shared::Metadata;
use temp_dir::TempDir;
use time::OffsetDateTime;

mod helpers;

#[tokio::test]
async fn test_random() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_mealplan::Command(state.evento.clone(), state.pool.clone());
    let query = imkitchen_mealplan::Query(state.pool.clone());
    let recipe_command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());
    let albert = Metadata::by("albert".to_owned());

    for i in 0..200 {
        import_recipe(
            &recipe_command,
            i.to_string(),
            RecipeType::MainCourse,
            &john,
        )
        .await?;
    }

    for i in 0..4 {
        import_recipe(&recipe_command, i.to_string(), RecipeType::Appetizer, &john).await?;
    }

    for i in 0..5 {
        import_recipe(
            &recipe_command,
            i.to_string(),
            RecipeType::Appetizer,
            &albert,
        )
        .await?;
    }

    command.generate(&john).await?;
    let result = command.generate(&john).await;
    assert_eq!(
        &result.unwrap_err().to_string(),
        "Meal plan status is processing"
    );

    imkitchen_mealplan::subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    let ids1 = imkitchen_mealplan::random(
        &state.pool,
        "john",
        imkitchen_recipe::RecipeType::MainCourse,
    )
    .await?;

    let ids2 = imkitchen_mealplan::random(
        &state.pool,
        "john",
        imkitchen_recipe::RecipeType::MainCourse,
    )
    .await?;

    assert_ne!(ids1, ids2);
    assert_eq!(ids1.len(), 28);

    imkitchen_mealplan::subscribe_week()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    command.generate(&john).await?;

    imkitchen_mealplan::subscribe_week()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    let now = OffsetDateTime::now_utc();
    let weeks = imkitchen_mealplan::next_four_mondays(now.unix_timestamp())?;
    for week in weeks {
        let row = query.find(week as u64, "john").await?.unwrap();
        assert!(row.slots.is_empty());
        assert_eq!(row.status.0, Status::Processing);
    }

    imkitchen_mealplan::subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    imkitchen_mealplan::subscribe_week()
        .data(state.pool)
        .unretry_oneshot(&state.evento)
        .await?;

    for week in weeks {
        let row = query.find(week as u64, "john").await?.unwrap();
        assert!(!row.slots.is_empty());
        assert_eq!(row.status.0, Status::Idle);
    }

    Ok(())
}

async fn import_recipe(
    cmd: &imkitchen_recipe::Command<evento::Sqlite>,
    id: impl Into<String>,
    recipe_type: RecipeType,
    metadata: &Metadata,
) -> anyhow::Result<()> {
    let id = id.into();
    let input = ImportInput {
        name: format!("recipe {id}"),
        description: "my description".to_owned(),
        advance_prep: "".to_owned(),
        ingredients: vec![],
        instructions: vec![],
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type,
    };

    cmd.import(input.clone(), metadata).await?;

    Ok(())
}
