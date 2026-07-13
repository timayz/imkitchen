use evento::{
    Sqlite,
    migrator::{Migrate, Plan},
};
use imkitchen_core::State;
use imkitchen_core::recipe::ImportInput;
use imkitchen_types::recipe::{Ingredient, IngredientCategory, IngredientUnit, RecipeType};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{path::PathBuf, str::FromStr};

pub async fn setup_test_state(path: PathBuf) -> anyhow::Result<State<Sqlite>> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    let mut conn = pool.acquire().await?;
    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;

    Ok(State {
        executor: pool.clone().into(),
        read_db: pool.clone(),
        write_db: pool,
    })
}

/// Import a recipe with a single ingredient and return its id.
pub async fn import_recipe(
    cmd: &imkitchen_core::recipe::Module<Sqlite>,
    name: &str,
    ingredient_name: &str,
    quantity: u32,
    household_size: u16,
    user_id: &str,
) -> anyhow::Result<String> {
    let input = ImportInput {
        name: name.to_owned(),
        origin: None,
        description: "desc".to_owned(),
        advance_prep: "".to_owned(),
        ingredients: vec![Ingredient {
            name: ingredient_name.to_owned(),
            quantity,
            unit: Some(IngredientUnit::G),
            category: Some(IngredientCategory::Grocery),
        }],
        instructions: vec![],
        household_size,
        cook_time: 25,
        prep_time: 10,
        recipe_type: RecipeType::MainCourse,
        accepts_accompaniment: false,
        dietary_restrictions: vec![],
    };

    cmd.import(input, user_id, None).await.map_err(Into::into)
}

/// Drain the shopping subscription (maintains `shopping_recipe` / `shopping_slot`).
pub async fn run_shopping_subscription(state: &State<Sqlite>) -> anyhow::Result<()> {
    imkitchen_core::shopping::subscription()
        .data(state.write_db.clone())
        .no_retry()
        .run_once(&state.executor)
        .await?;
    Ok(())
}

/// Drain the shopping-list read-model subscription (maintains `shopping_list`).
pub async fn run_shopping_list_subscription(state: &State<Sqlite>) -> anyhow::Result<()> {
    imkitchen_core::shopping::list::subscription()
        .data(state.write_db.clone())
        .no_retry()
        .run_once(&state.executor)
        .await?;
    Ok(())
}
