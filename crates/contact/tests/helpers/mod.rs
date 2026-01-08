use std::{path::PathBuf, str::FromStr};

use evento::{
    Sqlite,
    migrator::{Migrate, Plan},
};
use imkitchen_contact::SubmitFormInput;
use imkitchen_shared::contact::Subject;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

pub struct TestState {
    pub evento: Sqlite,
    pub pool: SqlitePool,
}

pub async fn setup_test_state(path: PathBuf) -> anyhow::Result<TestState> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
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
pub async fn create_submit(state: &TestState, name: impl Into<String>) -> anyhow::Result<String> {
    let ids = create_submit_all(state, vec![name]).await?;

    Ok(ids.first().unwrap().to_owned())
}

#[allow(dead_code)]
pub async fn create_submit_all(
    state: &TestState,
    names: impl IntoIterator<Item = impl Into<String>>,
) -> anyhow::Result<Vec<String>> {
    let mut ids = vec![];
    for name in names.into_iter() {
        let name = name.into();
        let id = imkitchen_contact::Command::submit_form(
            &state.evento,
            SubmitFormInput {
                to: "contact@imkitchen.localhost".to_owned(),
                email: format!("{name}@imkitchen.localhost"),
                name: "my name".to_owned(),
                subject: Subject::Other,
                message: "my message".to_owned(),
            },
        )
        .await?;
        ids.push(id);
    }

    Ok(ids)
}
