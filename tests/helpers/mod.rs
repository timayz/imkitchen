use evento::{
    Sqlite,
    migrator::{Migrate, Plan},
};
use imkitchen_contact::{ContactSubject, SubmitContactFormInput};
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
