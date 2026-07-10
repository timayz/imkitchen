use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::recipe_thumbnail::RecipeThumbnail;
use imkitchen_types::recipe::Deleted;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

#[derive(Default, FromRow)]
pub struct ThumbnailView {
    pub id: String,
    pub device: String,
    pub data: Vec<u8>,
}

impl<E: Executor> crate::recipe::Module<E> {
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

        Ok(sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
            .fetch_optional(&self.read_db)
            .await?)
    }
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    // Resized variant bytes are now written directly to recipe_thumbnail by the
    // "recipe-command" resize subscription (the authoritative image store), so
    // there is no longer a ThumbnailResized handler here. This view only needs
    // to clean up on deletion.
    SubscriptionBuilder::new("recipe-thumbnail-view").handler(handle_deleted())
}

#[evento::subscription]
async fn handle_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    // No device filter on purpose: this removes every variant for the recipe,
    // including any transient device='original' row left behind if the process
    // died mid-resize.
    let statement = Query::delete()
        .from_table(RecipeThumbnail::Table)
        .and_where(Expr::col(RecipeThumbnail::Id).eq(&event.aggregate_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}
