use imkitchen_recipe::{CuisineType, ImportInput, RecipeType};
use temp_dir::TempDir;

use crate::helpers::TestState;

mod helpers;

#[tokio::test]
async fn test_random() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;

    for i in 0..200 {
        import_recipe(&state, i.to_string(), RecipeType::MainCourse, "john").await?;
    }

    for i in 0..4 {
        import_recipe(&state, i.to_string(), RecipeType::Appetizer, "john").await?;
    }

    for i in 0..5 {
        import_recipe(&state, i.to_string(), RecipeType::Appetizer, "albert").await?;
    }

    imkitchen_mealplan::subscription()
        .data(state.pool.clone())
        .unretry_execute(&state.evento)
        .await?;

    let weeks = imkitchen_mealplan::next_four_mondays_from_now()
        .iter()
        .map(|w| {
            (
                w.start.unix_timestamp() as u64,
                w.end.unix_timestamp() as u64,
            )
        })
        .collect::<Vec<_>>();

    imkitchen_mealplan::Command::generate(
        &state.evento,
        &state.pool,
        imkitchen_mealplan::Generate {
            user_id: "john".to_owned(),
            weeks: weeks.to_vec(),
            randomize: Some(imkitchen_mealplan::Randomize {
                cuisine_variety_weight: 1.0,
                dietary_restrictions: vec![],
            }),
            household_size: 2,
        },
    )
    .await?;

    let last = imkitchen_mealplan::last_week::load(&state.evento, "john").await?;
    assert_eq!(weeks.last().unwrap().0, last.unwrap().week);

    Ok(())
}

async fn import_recipe(
    state: &TestState,
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

    imkitchen_recipe::Command::import(&state.evento, input, user_id, None).await?;

    Ok(())
}
