use evento::{AggregatorName, Executor, LoadResult, SubscribeBuilder};
use imkitchen_db::table::{RecipeList, RecipeRating as RecipeRatingIden};
use imkitchen_shared::{Event, Metadata};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::{SqlitePool, prelude::FromRow};
use ulid::Ulid;
use validator::Validate;

use crate::{
    Created, Deleted, Imported, Recipe,
    rating::{
        CommentAdded, CommentLikeCheked, CommentLikeUnchecked, CommentUnlikeChecked,
        CommentUnlikeUnchecked, LikeChecked, LikeUnchecked, RecipeRating, UnlikeChecked,
        UnlikeUnchecked, Viewed,
    },
};

#[derive(Deserialize, FromRow, Default)]
pub struct RecipeRatingRow {
    pub viewed: bool,
    pub liked: bool,
    pub unliked: bool,
}

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load_recipe(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<Recipe>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }

    pub async fn find(
        &self,
        recipe_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> imkitchen_shared::Result<Option<RecipeRatingRow>> {
        let recipe_id = recipe_id.into();
        let user_id = user_id.into();
        let statement = Query::select()
            .columns([
                RecipeRatingIden::Viewed,
                RecipeRatingIden::Liked,
                RecipeRatingIden::Unliked,
            ])
            .from(RecipeRatingIden::Table)
            .and_where(Expr::col(RecipeRatingIden::RecipeId).eq(recipe_id))
            .and_where(Expr::col(RecipeRatingIden::UserId).eq(user_id))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, RecipeRatingRow, _>(&sql, values)
            .fetch_optional(&self.1)
            .await?)
    }
}

impl<E: Executor + Clone> Command<E> {
    pub async fn check_like(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };

        let user_id = metadata.trigger_by()?;
        let liked = self
            .find(&id, user_id)
            .await?
            .map(|r| r.liked)
            .unwrap_or(false);

        if liked {
            return Ok(());
        }

        evento::save::<RecipeRating>(id)
            .data(&LikeChecked { liked: true })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn uncheck_like(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };

        let user_id = metadata.trigger_by()?;
        let liked = self
            .find(&id, user_id)
            .await?
            .map(|r| r.liked)
            .unwrap_or(false);

        if !liked {
            return Ok(());
        }

        evento::save::<RecipeRating>(id)
            .data(&LikeUnchecked { liked: false })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn check_unlike(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };

        let user_id = metadata.trigger_by()?;
        let unliked = self
            .find(&id, user_id)
            .await?
            .map(|r| r.unliked)
            .unwrap_or(false);

        if unliked {
            return Ok(());
        }

        evento::save::<RecipeRating>(id)
            .data(&UnlikeChecked { unliked: true })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn uncheck_unlike(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };

        let user_id = metadata.trigger_by()?;
        let unliked = self
            .find(&id, user_id)
            .await?
            .map(|r| r.unliked)
            .unwrap_or(false);

        if !unliked {
            return Ok(());
        }

        evento::save::<RecipeRating>(id)
            .data(&UnlikeUnchecked { unliked: false })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn view(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };

        let user_id = metadata.trigger_by()?;
        let viewed = self
            .find(&id, user_id)
            .await?
            .map(|r| r.viewed)
            .unwrap_or(false);

        if viewed {
            return Ok(());
        }

        evento::save::<RecipeRating>(id)
            .data(&Viewed { viewed: true })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}

#[derive(Validate)]
pub struct AddCommentInput {
    pub id: String,
    pub message: String,
    pub reply_to: Option<String>,
}

impl<E: Executor + Clone> Command<E> {
    pub async fn add_comment(
        &self,
        input: AddCommentInput,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        if self.load_recipe(&input.id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };
        // @TODO: check not already viewd
        // @TODO: check replay to comment exist

        evento::save::<RecipeRating>(input.id)
            .data(&CommentAdded {
                id: Ulid::new().to_string(),
                message: input.message,
                reply_to: input.reply_to,
            })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn check_comment_like(
        &self,
        id: impl Into<String>,
        comment_id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };
        // @TODO: check not already viewd

        let comment_id = comment_id.into();

        // @TODO: check comment exist

        evento::save::<RecipeRating>(id)
            .data(&CommentLikeCheked { comment_id })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn uncheck_comment_like(
        &self,
        id: impl Into<String>,
        comment_id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };
        // @TODO: check not already viewd

        let comment_id = comment_id.into();

        // @TODO: check comment exist

        evento::save::<RecipeRating>(id)
            .data(&CommentLikeUnchecked { comment_id })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn check_comment_unlike(
        &self,
        id: impl Into<String>,
        comment_id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };
        // @TODO: check not already viewd

        let comment_id = comment_id.into();

        // @TODO: check comment exist

        evento::save::<RecipeRating>(id)
            .data(&CommentUnlikeChecked { comment_id })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }

    pub async fn uncheck_comment_unlike(
        &self,
        id: impl Into<String>,
        comment_id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<()> {
        let id = id.into();
        if self.load_recipe(&id).await?.is_none() {
            imkitchen_shared::bail!("Recipe not found");
        };
        // @TODO: check not already viewd

        let comment_id = comment_id.into();

        // @TODO: check comment exist

        evento::save::<RecipeRating>(id)
            .data(&CommentUnlikeUnchecked { comment_id })?
            .metadata(metadata)?
            .commit(&self.0)
            .await?;

        Ok(())
    }
}

pub fn subscribe_command<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("rating-command")
        .handler(handle_viewed())
        .handler(handle_like_checked())
        .handler(handle_like_unchecked())
        .handler(handle_unlike_checked())
        .handler(handle_unlike_unchecked())
        .handler(handle_recipe_created())
        .handler(handle_recipe_imported())
        .handler(handle_recipe_deleted())
        .handler_check_off()
}

#[evento::handler(RecipeRating)]
async fn handle_viewed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Viewed>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let user_id = event.metadata.trigger_by()?;

    let statement = Query::update()
        .table(RecipeList::Table)
        .value(
            RecipeList::TotalViews,
            Expr::col(RecipeList::TotalViews).add(1),
        )
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    let statement = Query::insert()
        .into_table(RecipeRatingIden::Table)
        .columns([
            RecipeRatingIden::RecipeId,
            RecipeRatingIden::UserId,
            RecipeRatingIden::Viewed,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            user_id.into(),
            true.into(),
        ])
        .on_conflict(
            OnConflict::new()
                .update_column(RecipeRatingIden::Viewed)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeRating)]
async fn handle_like_checked<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<LikeChecked>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let user_id = event.metadata.trigger_by()?;

    let statement = Query::update()
        .table(RecipeList::Table)
        .value(
            RecipeList::TotalLikes,
            Expr::col(RecipeList::TotalLikes).add(1),
        )
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    let statement = Query::insert()
        .into_table(RecipeRatingIden::Table)
        .columns([
            RecipeRatingIden::RecipeId,
            RecipeRatingIden::UserId,
            RecipeRatingIden::Liked,
            RecipeRatingIden::Unliked,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            user_id.into(),
            true.into(),
            false.into(),
        ])
        .on_conflict(
            OnConflict::new()
                .update_columns([RecipeRatingIden::Liked, RecipeRatingIden::Unliked])
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeRating)]
async fn handle_like_unchecked<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<LikeUnchecked>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let user_id = event.metadata.trigger_by()?;

    let statement = Query::update()
        .table(RecipeList::Table)
        .value(
            RecipeList::TotalLikes,
            Expr::col(RecipeList::TotalLikes).sub(1),
        )
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    let statement = Query::insert()
        .into_table(RecipeRatingIden::Table)
        .columns([
            RecipeRatingIden::RecipeId,
            RecipeRatingIden::UserId,
            RecipeRatingIden::Liked,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            user_id.into(),
            false.into(),
        ])
        .on_conflict(
            OnConflict::new()
                .update_column(RecipeRatingIden::Liked)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeRating)]
async fn handle_unlike_checked<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<UnlikeChecked>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let user_id = event.metadata.trigger_by()?;

    let statement = Query::update()
        .table(RecipeList::Table)
        .value(
            RecipeList::TotalLikes,
            Expr::col(RecipeList::TotalLikes).sub(1),
        )
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    let statement = Query::insert()
        .into_table(RecipeRatingIden::Table)
        .columns([
            RecipeRatingIden::RecipeId,
            RecipeRatingIden::UserId,
            RecipeRatingIden::Unliked,
            RecipeRatingIden::Liked,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            user_id.into(),
            true.into(),
            false.into(),
        ])
        .on_conflict(
            OnConflict::new()
                .update_columns([RecipeRatingIden::Unliked, RecipeRatingIden::Liked])
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(RecipeRating)]
async fn handle_unlike_unchecked<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<UnlikeUnchecked>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let user_id = event.metadata.trigger_by()?;

    let statement = Query::update()
        .table(RecipeList::Table)
        .value(
            RecipeList::TotalLikes,
            Expr::col(RecipeList::TotalLikes).add(1),
        )
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    let statement = Query::insert()
        .into_table(RecipeRatingIden::Table)
        .columns([
            RecipeRatingIden::RecipeId,
            RecipeRatingIden::UserId,
            RecipeRatingIden::Unliked,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            user_id.into(),
            false.into(),
        ])
        .on_conflict(
            OnConflict::new()
                .update_column(RecipeRatingIden::Unliked)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_created<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let statement = Query::update()
        .table(RecipeList::Table)
        .value(RecipeList::TotalViews, 0)
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_imported<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let statement = Query::update()
        .table(RecipeList::Table)
        .value(RecipeList::TotalViews, 0)
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_deleted<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let statement = Query::delete()
        .from_table(RecipeRatingIden::Table)
        .and_where(Expr::col(RecipeRatingIden::RecipeId).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
