use evento::cursor::Args;
use imkitchen::{RecipeInput, RecipeSortBy, subscribe_global_stat, subscribe_recipe};
use imkitchen_recipe::{CuisineType, RecipeType};
use imkitchen_shared::Metadata;

mod helpers;

#[tokio::test]
pub async fn test_recipe_query() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let users = helpers::create_users(&state, vec!["john", "albert"]).await?;

    let _ = helpers::create_recipes(
        &state,
        [
            (
                "john recipe 1",
                RecipeType::MainCourse,
                CuisineType::Caribbean,
            ),
            ("john recipe 2", RecipeType::MainCourse, CuisineType::Thai),
            ("john recipe 3", RecipeType::Dessert, CuisineType::Thai),
        ],
        Metadata::by(users[0].to_owned()),
    )
    .await?;

    let _ = helpers::create_recipes(
        &state,
        [(
            "albert recipe 1",
            RecipeType::MainCourse,
            CuisineType::Caribbean,
        )],
        Metadata::by(users[1].to_owned()),
    )
    .await?;

    subscribe_recipe()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    subscribe_global_stat()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    let stats = imkitchen::query_user_global_stats(&state.pool, users[0].to_owned()).await?;
    assert_eq!(stats.total, 3);

    Ok(())
}
