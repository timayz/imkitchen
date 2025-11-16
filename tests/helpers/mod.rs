use evento::{
    Sqlite,
    migrator::{Migrate, Plan},
};
use imkitchen_contact::{ContactSubject, SubmitContactFormInput};
use imkitchen_recipe::{
    AccompanimentType, CuisineType, DietaryRestriction, Ingredient, Instruction, RecipeType,
    UpdateInput,
};
use imkitchen_shared::Metadata;
use imkitchen_user::{RegisterInput, subscribe_command};
use sqlx::SqlitePool;

pub struct TestState {
    pub evento: Sqlite,
    pub pool: SqlitePool,
}

pub async fn setup_test_state() -> anyhow::Result<TestState> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    let mut conn = pool.acquire().await?;
    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;

    Ok(TestState {
        evento: pool.clone().into(),
        pool,
    })
}

#[allow(dead_code)]
pub async fn create_user(state: &TestState, name: impl Into<String>) -> anyhow::Result<String> {
    let ids = create_users(state, vec![name]).await?;

    Ok(ids.first().unwrap().to_owned())
}

#[allow(dead_code)]
pub async fn create_users(
    state: &TestState,
    names: impl IntoIterator<Item = impl Into<String>>,
) -> anyhow::Result<Vec<String>> {
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());

    let mut ids = vec![];
    for name in names.into_iter() {
        let name = name.into();
        let id = command
            .register(
                RegisterInput {
                    email: format!("{name}@imkitchen.localhost"),
                    password: "my_password".to_owned(),
                },
                Metadata::default(),
            )
            .await?;
        ids.push(id);
    }

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(ids)
}

#[allow(dead_code)]
pub async fn create_submit_contact_form(
    state: &TestState,
    name: impl Into<String>,
) -> anyhow::Result<String> {
    let ids = create_submit_contact_form_all(state, vec![name]).await?;

    Ok(ids.first().unwrap().to_owned())
}

#[allow(dead_code)]
pub async fn create_submit_contact_form_all(
    state: &TestState,
    names: impl IntoIterator<Item = impl Into<String>>,
) -> anyhow::Result<Vec<String>> {
    let command = imkitchen_contact::Command(state.evento.clone(), state.pool.clone());

    let mut ids = vec![];
    for name in names.into_iter() {
        let name = name.into();
        let id = command
            .submit_contact_form(
                SubmitContactFormInput {
                    email: format!("{name}@imkitchen.localhost"),
                    name: "my name".to_owned(),
                    subject: ContactSubject::Other,
                    message: "my message".to_owned(),
                },
                Metadata::default(),
            )
            .await?;
        ids.push(id);
    }

    Ok(ids)
}

#[allow(dead_code)]
pub async fn create_recipes(
    state: &TestState,
    recipes: impl IntoIterator<Item = (impl Into<String>, RecipeType, CuisineType)>,
    metadata: Metadata,
) -> anyhow::Result<Vec<String>> {
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());

    let mut ids = vec![];
    for (name, recipe_type, cuisine_type) in recipes.into_iter() {
        let name = name.into();
        let id = command.create(metadata.clone()).await?;
        let input = UpdateInput {
            id: id.to_owned(),
            name,
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
                quantity: 1,
                unit: "g".to_owned(),
            }],
            instructions: vec![Instruction {
                time_before_next: 15,
                description: "My first instruction".to_owned(),
            }],
            cook_time: 25,
            prep_time: 10,
            cuisine_type,
            recipe_type,
        };
        command.update(input, metadata.clone()).await?;
        ids.push(id);
    }

    Ok(ids)
}
