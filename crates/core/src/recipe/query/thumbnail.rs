use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::RecipeThumbnail;
use imkitchen_shared::recipe::{Deleted, ThumbnailResized};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

#[derive(Default, FromRow)]
pub struct ThumbnailView {
    pub id: String,
    pub device: String,
    pub data: Vec<u8>,
}

impl<E: Executor> super::Query<E> {
    pub async fn find_thumbnail(
        &self,
        id: impl Into<String>,
        device: impl Into<String>,
    ) -> anyhow::Result<Option<ThumbnailView>> {
        let id = id.into();
        let device = device.into();
        let statement = sea_query::Query::select()
            .columns([
                RecipeThumbnail::Id,
                RecipeThumbnail::Device,
                RecipeThumbnail::Data,
            ])
            .from(RecipeThumbnail::Table)
            .and_where(Expr::col(RecipeThumbnail::Id).eq(id))
            .and_where(Expr::col(RecipeThumbnail::Device).eq(device))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with(&sql, values)
            .fetch_optional(&self.read_db)
            .await?)
    }
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-thumbnail-view")
        .handler(handle_resized())
        .handler(handle_deleted())
}

#[evento::subscription]
async fn handle_resized<E: Executor>(
    context: &Context<'_, E>,
    event: Event<ThumbnailResized>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let statement = Query::insert()
        .into_table(RecipeThumbnail::Table)
        .columns([
            RecipeThumbnail::Id,
            RecipeThumbnail::Device,
            RecipeThumbnail::Data,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.device.to_owned().into(),
            event.data.data.into(),
        ])
        .on_conflict(
            OnConflict::columns([RecipeThumbnail::Id, RecipeThumbnail::Device])
                .update_column(RecipeThumbnail::Data)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::subscription]
async fn handle_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let statement = Query::delete()
        .from_table(RecipeThumbnail::Table)
        .and_where(Expr::col(RecipeThumbnail::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
