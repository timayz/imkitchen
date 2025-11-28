use evento::cursor::Args;
use imkitchen_recipe::{
    Command, CuisineType, DietaryRestriction, Ingredient, Instruction, RecipeType, RecipesQuery,
    SortBy, UpdateInput, subscribe_list, subscribe_user_stat,
};
use imkitchen_shared::Metadata;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
pub async fn test_recipe_query() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let query = imkitchen_recipe::Query(state.pool.clone());
    let john = Metadata::by("john".to_owned());
    let albert = Metadata::by("albert".to_owned());

    let _ = create_recipes(
        &command,
        [
            (
                "john recipe 1",
                RecipeType::MainCourse,
                CuisineType::Caribbean,
            ),
            ("john recipe 2", RecipeType::MainCourse, CuisineType::Thai),
            ("john recipe 3", RecipeType::Dessert, CuisineType::Thai),
        ],
        &john,
    )
    .await?;

    let _ = create_recipes(
        &command,
        [(
            "albert recipe 1",
            RecipeType::MainCourse,
            CuisineType::Caribbean,
        )],
        &albert,
    )
    .await?;

    subscribe_list()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    subscribe_user_stat()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    let stat = query.find_user_stat("john").await?.unwrap();
    assert_eq!(stat.total, 3);

    let result = query
        .filter(RecipesQuery {
            args: Args::default(),
            user_id: Some("john".to_owned()),
            sort_by: SortBy::RecentlyAdded,
            is_shared: None,
            recipe_type: None,
            cuisine_type: None,
        })
        .await?;

    assert_eq!(result.edges.len(), 3);

    let recipe = query.find(&result.edges[0].node.id).await?.unwrap();
    assert_eq!(&recipe.name, "john recipe 3");
    assert_eq!(&recipe.ingredients[0].name, "ingredient 1");

    Ok(())
}

pub async fn create_recipes(
    command: &Command<evento::Sqlite>,
    recipes: impl IntoIterator<Item = (impl Into<String>, RecipeType, CuisineType)>,
    metadata: &Metadata,
) -> anyhow::Result<Vec<String>> {
    let mut ids = vec![];
    for (name, recipe_type, cuisine_type) in recipes.into_iter() {
        let name = name.into();
        let id = command.create(metadata).await?;
        let input = UpdateInput {
            id: id.to_owned(),
            name,
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
            cuisine_type,
            recipe_type,
        };
        command.update(input, metadata).await?;
        ids.push(id);
    }

    Ok(ids)
}
