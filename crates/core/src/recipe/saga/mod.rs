//! Saga that translates the bulk "share all" / "make all private" commands into
//! per-recipe `SharedToCommunity` / `MadePrivate` events.
//!
//! Keeping the effect in each recipe's own event stream makes the recipe-query
//! projection the single, reload- and rebuild-safe owner of `is_shared`: a
//! bulk-shared recipe survives every reprojection because its shared state lives
//! in its own history, not in a one-off SQL overlay. The bulk
//! `AllSharedToCommunity` / `AllMadePrivate` aggregate event stays the cheap,
//! O(1) command; this saga does the per-recipe fan-out.
//!
//! To know which recipes an owner has, the saga maintains its own tiny
//! `recipe_owner` index from `Created` / `Imported` / `Deleted` events. Because a
//! subscription processes the log in order, that index is always complete by the
//! time an `AllSharedToCommunity` is reached — so the fan-out never reads the
//! recipe-query read model, which may be mid-rebuild (e.g. right after the m0005
//! truncate). That is what keeps the historical backfill race-free.

pub mod embeddable;

use evento::{
    Executor, ProjectionAggregate,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::recipe_owner::RecipeOwner;
use imkitchen_types::recipe::{Created, Deleted, Imported, MadePrivate, SharedToCommunity};
use imkitchen_types::recipe_share::{AllMadePrivate, AllSharedToCommunity};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-saga-share")
        .handler(handle_created())
        .handler(handle_imported())
        .handler(handle_deleted())
        .handler(handle_all_shared_to_community())
        .handler(handle_all_made_private())
}

/// Records `recipe_id → owner_id` in the saga's own index.
async fn index_owner(db: &SqlitePool, recipe_id: &str, owner_id: &str) -> anyhow::Result<()> {
    let (sql, values) = Query::insert()
        .into_table(RecipeOwner::Table)
        .columns([RecipeOwner::RecipeId, RecipeOwner::OwnerId])
        .values([recipe_id.to_owned().into(), owner_id.to_owned().into()])?
        .on_conflict(
            OnConflict::column(RecipeOwner::RecipeId)
                .update_column(RecipeOwner::OwnerId)
                .to_owned(),
        )
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(db)
        .await?;

    Ok(())
}

/// Recipe ids owned by `owner_id`, read from the saga's index (never the
/// possibly-rebuilding recipe-query read model).
async fn owner_recipe_ids(db: &SqlitePool, owner_id: &str) -> anyhow::Result<Vec<String>> {
    let (sql, values) = Query::select()
        .column(RecipeOwner::RecipeId)
        .from(RecipeOwner::Table)
        .and_where(Expr::col(RecipeOwner::OwnerId).eq(owner_id))
        .build_sqlx(SqliteQueryBuilder);

    Ok(
        sqlx::query_as_with::<_, (String,), _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|(id,)| id)
            .collect(),
    )
}

#[evento::subscription]
async fn handle_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    index_owner(
        &write_db,
        &event.aggregate_id,
        &event.metadata.requested_by()?,
    )
    .await
}

#[evento::subscription]
async fn handle_imported<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    index_owner(
        &write_db,
        &event.aggregate_id,
        &event.metadata.requested_by()?,
    )
    .await
}

#[evento::subscription]
async fn handle_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    let (sql, values) = Query::delete()
        .from_table(RecipeOwner::Table)
        .and_where(Expr::col(RecipeOwner::RecipeId).eq(&event.aggregate_id))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&write_db)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_all_shared_to_community<E: Executor>(
    context: &Context<'_, E>,
    event: Event<AllSharedToCommunity>,
) -> anyhow::Result<()> {
    let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    let owner_id = event.metadata.requested_by()?;

    for id in owner_recipe_ids(&write_db, &owner_id).await? {
        let Some(recipe) = crate::recipe::create_projection()
            .load(&id)
            .execute(context.executor)
            .await?
        else {
            continue;
        };

        // Idempotent on replay/retry: the aggregate guard skips recipes the
        // saga already shared.
        if recipe.is_shared {
            continue;
        }

        recipe
            .write()?
            .event(&SharedToCommunity {
                owner_name: event.data.owner_name.to_owned(),
            })
            .requested_by(owner_id.as_str())
            .commit(context.executor)
            .await?;
    }

    Ok(())
}

#[evento::subscription]
async fn handle_all_made_private<E: Executor>(
    context: &Context<'_, E>,
    event: Event<AllMadePrivate>,
) -> anyhow::Result<()> {
    let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    let owner_id = event.metadata.requested_by()?;

    for id in owner_recipe_ids(&write_db, &owner_id).await? {
        let Some(recipe) = crate::recipe::create_projection()
            .load(&id)
            .execute(context.executor)
            .await?
        else {
            continue;
        };

        if !recipe.is_shared {
            continue;
        }

        recipe
            .write()?
            .event(&MadePrivate)
            .requested_by(owner_id.as_str())
            .commit(context.executor)
            .await?;
    }

    Ok(())
}
