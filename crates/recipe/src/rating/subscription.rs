use evento::{Action, Executor, Projection, SubscriptionBuilder, metadata::Event};
use imkitchen_db::table::RecipeRatingCommand;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

use crate::{
    Deleted,
    rating::{LikeChecked, LikeUnchecked, UnlikeChecked, UnlikeUnchecked, Viewed},
};

use super::CommandData;

pub fn create_projection<E: Executor>() -> Projection<CommandData, E> {
    Projection::new("rating-command")
        .handler(handle_viewed())
        .handler(handle_like_checked())
        .handler(handle_like_unchecked())
        .handler(handle_unlike_checked())
        .handler(handle_unlike_unchecked())
        .handler(handle_recipe_deleted())
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<CommandData, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::snapshot]
async fn restore(
    context: &evento::context::RwContext,
    id: String,
    aggregators: &std::collections::HashMap<String, String>,
) -> anyhow::Result<Option<CommandData>> {
    let user_id = aggregators.get("imkitchen-user/User").unwrap();
    let pool = context.extract::<SqlitePool>();
    let statement = Query::select()
        .columns([
            RecipeRatingCommand::UserId,
            RecipeRatingCommand::Viewed,
            RecipeRatingCommand::Liked,
            RecipeRatingCommand::Unliked,
        ])
        .from(RecipeRatingCommand::Table)
        .and_where(Expr::col(RecipeRatingCommand::RecipeId).eq(id))
        .and_where(Expr::col(RecipeRatingCommand::UserId).eq(user_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(&pool)
        .await?)
}

#[evento::handler]
async fn handle_viewed<E: Executor>(
    event: Event<Viewed>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.user_id = event.metadata.user()?;
            data.viewed = true;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();
            let user_id = event.metadata.user()?;

            let statement = Query::insert()
                .into_table(RecipeRatingCommand::Table)
                .columns([
                    RecipeRatingCommand::RecipeId,
                    RecipeRatingCommand::UserId,
                    RecipeRatingCommand::Viewed,
                ])
                .values_panic([
                    event.aggregator_id.to_owned().into(),
                    user_id.into(),
                    true.into(),
                ])
                .on_conflict(
                    OnConflict::new()
                        .update_column(RecipeRatingCommand::Viewed)
                        .to_owned(),
                )
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_like_checked<E: Executor>(
    event: Event<LikeChecked>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.user_id = event.metadata.user()?;
            data.liked = true;
            data.unliked = false;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();
            let user_id = event.metadata.user()?;

            let statement = Query::insert()
                .into_table(RecipeRatingCommand::Table)
                .columns([
                    RecipeRatingCommand::RecipeId,
                    RecipeRatingCommand::UserId,
                    RecipeRatingCommand::Liked,
                    RecipeRatingCommand::Unliked,
                ])
                .values_panic([
                    event.aggregator_id.to_owned().into(),
                    user_id.into(),
                    true.into(),
                    false.into(),
                ])
                .on_conflict(
                    OnConflict::new()
                        .update_columns([RecipeRatingCommand::Liked, RecipeRatingCommand::Unliked])
                        .to_owned(),
                )
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_like_unchecked<E: Executor>(
    event: Event<LikeUnchecked>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.user_id = event.metadata.user()?;
            data.liked = false;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();
            let user_id = event.metadata.user()?;

            let statement = Query::insert()
                .into_table(RecipeRatingCommand::Table)
                .columns([
                    RecipeRatingCommand::RecipeId,
                    RecipeRatingCommand::UserId,
                    RecipeRatingCommand::Liked,
                ])
                .values_panic([
                    event.aggregator_id.to_owned().into(),
                    user_id.into(),
                    false.into(),
                ])
                .on_conflict(
                    OnConflict::new()
                        .update_column(RecipeRatingCommand::Liked)
                        .to_owned(),
                )
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_unlike_checked<E: Executor>(
    event: Event<UnlikeChecked>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.user_id = event.metadata.user()?;
            data.unliked = true;
            data.liked = false;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();
            let user_id = event.metadata.user()?;

            let statement = Query::insert()
                .into_table(RecipeRatingCommand::Table)
                .columns([
                    RecipeRatingCommand::RecipeId,
                    RecipeRatingCommand::UserId,
                    RecipeRatingCommand::Unliked,
                    RecipeRatingCommand::Liked,
                ])
                .values_panic([
                    event.aggregator_id.to_owned().into(),
                    user_id.into(),
                    true.into(),
                    false.into(),
                ])
                .on_conflict(
                    OnConflict::new()
                        .update_columns([RecipeRatingCommand::Unliked, RecipeRatingCommand::Liked])
                        .to_owned(),
                )
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_unlike_unchecked<E: Executor>(
    event: Event<UnlikeUnchecked>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.user_id = event.metadata.user()?;
            data.unliked = false;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();
            let user_id = event.metadata.user()?;

            let statement = Query::insert()
                .into_table(RecipeRatingCommand::Table)
                .columns([
                    RecipeRatingCommand::RecipeId,
                    RecipeRatingCommand::UserId,
                    RecipeRatingCommand::Unliked,
                ])
                .values_panic([
                    event.aggregator_id.to_owned().into(),
                    user_id.into(),
                    false.into(),
                ])
                .on_conflict(
                    OnConflict::new()
                        .update_column(RecipeRatingCommand::Unliked)
                        .to_owned(),
                )
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_recipe_deleted<E: Executor>(
    event: Event<Deleted>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.unliked = false;
            data.liked = false;
            data.viewed = false;
            data.user_id = event.metadata.user()?;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();

            let statement = Query::delete()
                .from_table(RecipeRatingCommand::Table)
                .and_where(Expr::col(RecipeRatingCommand::RecipeId).eq(&event.aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}
