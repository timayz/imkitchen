use std::{path::PathBuf, str::FromStr};

use evento::{
    Sqlite,
    migrator::{Migrate, Plan},
};
use imkitchen_contact::SubmitFormInput;
use imkitchen_shared::{State, contact::Subject};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

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

#[allow(dead_code)]
pub async fn create_submit(
    cmd: &imkitchen_contact::Command<Sqlite>,
    name: impl Into<String>,
) -> anyhow::Result<String> {
    let ids = create_submit_all(cmd, vec![name]).await?;

    Ok(ids.first().unwrap().to_owned())
}

#[allow(dead_code)]
pub async fn create_submit_all(
    cmd: &imkitchen_contact::Command<Sqlite>,
    names: impl IntoIterator<Item = impl Into<String>>,
) -> anyhow::Result<Vec<String>> {
    let mut ids = vec![];
    for name in names.into_iter() {
        let name = name.into();
        let id = cmd
            .submit_form(SubmitFormInput {
                to: "contact@imkitchen.localhost".to_owned(),
                email: format!("{name}@imkitchen.localhost"),
                name: "my name".to_owned(),
                subject: Subject::Other,
                message: "my message".to_owned(),
            })
            .await?;
        ids.push(id);
    }

    Ok(ids)
}
