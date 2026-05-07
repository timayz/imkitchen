use std::ops::Deref;

use evento::{Executor, Projection, Snapshot, metadata::Event};
use imkitchen_db::notification_recipient::NotificationRecipient;
use imkitchen_identity::types::user::{self, LoggedIn, Registered};
use sea_query::{Expr, ExprTrait, OnConflict, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;
use sqlx::prelude::FromRow;

#[derive(Clone)]
pub struct Module<E: Executor>(pub(crate) imkitchen_core::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = imkitchen_core::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Recipient>> {
        load(&self.executor, &self.read_db, &self.write_db, id).await
    }
}

#[evento::projection(FromRow)]
pub struct Recipient {
    pub id: String,
    pub email: String,
    pub lang: String,
    pub timezone: String,
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Recipient> {
    Projection::new::<user::User>(id)
        .handler(handle_registered())
        .handler(handle_logged_in())
}

impl evento::ProjectionAggregator for Recipient {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

pub(crate) async fn load<E: Executor>(
    executor: &E,
    read_db: &SqlitePool,
    write_db: &SqlitePool,
    id: impl Into<String>,
) -> anyhow::Result<Option<Recipient>> {
    create_projection(id)
        .data((read_db.clone(), write_db.clone()))
        .execute(executor)
        .await
}

async fn find(pool: &SqlitePool, id: &str) -> anyhow::Result<Option<Recipient>> {
    let statement = sea_query::Query::select()
        .columns([
            NotificationRecipient::Id,
            NotificationRecipient::Cursor,
            NotificationRecipient::Email,
            NotificationRecipient::Lang,
            NotificationRecipient::Timezone,
        ])
        .from(NotificationRecipient::Table)
        .and_where(Expr::col(NotificationRecipient::Id).eq(id))
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(pool)
        .await?)
}

impl<E: Executor> Snapshot<E> for Recipient {
    async fn restore(context: &evento::projection::Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let (read_db, _) = context.extract::<(SqlitePool, SqlitePool)>();
        find(&read_db, &context.id).await
    }

    async fn take_snapshot(
        &self,
        context: &evento::projection::Context<'_, E>,
    ) -> anyhow::Result<()> {
        let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();

        let statement = sea_query::Query::insert()
            .into_table(NotificationRecipient::Table)
            .columns([
                NotificationRecipient::Id,
                NotificationRecipient::Cursor,
                NotificationRecipient::Email,
                NotificationRecipient::Lang,
                NotificationRecipient::Timezone,
            ])
            .values([
                self.id.to_owned().into(),
                self.cursor.to_owned().into(),
                self.email.to_owned().into(),
                self.lang.to_owned().into(),
                self.timezone.to_owned().into(),
            ])?
            .on_conflict(
                OnConflict::column(NotificationRecipient::Id)
                    .update_columns([
                        NotificationRecipient::Cursor,
                        NotificationRecipient::Email,
                        NotificationRecipient::Lang,
                        NotificationRecipient::Timezone,
                    ])
                    .to_owned(),
            )
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(&write_db).await?;

        Ok(())
    }
}

#[evento::handler]
async fn handle_registered(event: Event<Registered>, data: &mut Recipient) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.email = event.data.email.to_owned();
    data.lang = event.data.lang.to_owned();
    data.timezone = event.data.timezone.to_owned();

    Ok(())
}

#[evento::handler]
async fn handle_logged_in(event: Event<LoggedIn>, data: &mut Recipient) -> anyhow::Result<()> {
    data.lang = event.data.lang.to_owned();
    data.timezone = event.data.timezone.to_owned();

    Ok(())
}
