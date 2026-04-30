use evento::Sqlite;
use imkitchen_core::recipe::ImportInput;
use imkitchen_shared::recipe::{CuisineType, RecipeType};
use temp_dir::TempDir;
use time::OffsetDateTime;

mod helpers;

#[tokio::test]
async fn test_random() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
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
        .unretry_execute(&state.executor)
        .await?;

    cmd.generate(imkitchen_core::mealplan::Generate {
        user_id: "john".to_owned(),
        days: 7,
        start: imkitchen_core::mealplan::date_to_u64(OffsetDateTime::now_utc()),
        randomize: Some(imkitchen_core::mealplan::Randomize {
            cuisine_variety_weight: 1.0,
            dietary_restrictions: vec![],
        }),
        household_size: 2,
    })
    .await?;

    Ok(())
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
        description: "my description".to_owned(),
        advance_prep: "".to_owned(),
        ingredients: vec![],
        instructions: vec![],
        household_size: 4,
        cook_time: 25,
        prep_time: 10,
        cuisine_type: CuisineType::Caribbean,
        recipe_type,
    };

    cmd.import(input, user_id, None).await?;

    Ok(())
}
