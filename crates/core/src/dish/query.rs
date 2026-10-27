//! Read side of dishes: the `dish` table (one row per live dish, maintained
//! by [`subscription`]) and the candidate pool a dish is composed from
//! (`meal_plan_recipe`: the user's own recipes plus saved community ones).

use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::dish::Dish;
use imkitchen_db::mealplan_recipe::MealPlanRecipe;
use imkitchen_types::recipe::RecipeType;
use sea_query::{Expr, ExprTrait, Order, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};

/// A dish as listed: its recipe ids in meal order. Recipes deleted since
/// are still listed here and must be resolved (and skipped) by the reader.
#[derive(Clone, Debug, FromRow)]
pub struct DishRow {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub recipe_ids: sqlx::types::Json<Vec<String>>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// A search of the candidate pool.
#[derive(Clone, Debug, Default)]
pub struct CandidateQuery {
    pub recipe_type: Option<RecipeType>,
    pub search: Option<String>,
    pub exclude_ids: Vec<String>,
    /// Page size, 1..=100.
    pub limit: u16,
}

impl CandidateQuery {
    /// Up to `limit` recipes of one course matching `search`.
    pub fn course(recipe_type: RecipeType, search: Option<String>, limit: u16) -> Self {
        Self {
            recipe_type: Some(recipe_type),
            search,
            exclude_ids: vec![],
            limit,
        }
    }
}

/// One page of candidates and the size of the whole match.
#[derive(Clone, Debug, Default)]
pub struct CandidatePage {
    pub items: Vec<Candidate>,
    pub total: u64,
}

/// A recipe the user can put in a dish.
#[derive(Clone, Debug, FromRow)]
pub struct Candidate {
    pub id: String,
    pub name: String,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub accepts_accompaniment: bool,
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("dish-query")
        .handler(handle_created())
        .handler(handle_changed())
        .handler(handle_deleted())
}

fn ids_json(ids: &[String]) -> serde_json::Value {
    serde_json::Value::Array(
        ids.iter()
            .map(|id| serde_json::Value::String(id.to_owned()))
            .collect(),
    )
}

#[evento::subscription]
async fn handle_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::dish::Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();
    let timestamp = event.timestamp;

    let statement = Query::insert()
        .into_table(Dish::Table)
        .columns([
            Dish::Id,
            Dish::UserId,
            Dish::Name,
            Dish::RecipeIds,
            Dish::CreatedAt,
            Dish::UpdatedAt,
        ])
        .values_panic([
            event.aggregate_id.to_owned().into(),
            event.metadata.requested_by()?.into(),
            event.data.name.into(),
            ids_json(&event.data.recipe_ids).into(),
            timestamp.into(),
            timestamp.into(),
        ])
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::dish::Changed>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    let statement = Query::update()
        .table(Dish::Table)
        .value(Dish::Name, &event.data.name)
        .value(Dish::RecipeIds, ids_json(&event.data.recipe_ids))
        .value(Dish::UpdatedAt, event.timestamp)
        .and_where(Expr::col(Dish::Id).eq(&event.aggregate_id))
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::dish::Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    let statement = Query::delete()
        .from_table(Dish::Table)
        .and_where(Expr::col(Dish::Id).eq(&event.aggregate_id))
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Every dish of a user, by name.
pub(crate) async fn list_rows(pool: &SqlitePool, user_id: &str) -> anyhow::Result<Vec<DishRow>> {
    let statement = Query::select()
        .columns([
            Dish::Id,
            Dish::UserId,
            Dish::Name,
            Dish::RecipeIds,
            Dish::CreatedAt,
            Dish::UpdatedAt,
        ])
        .from(Dish::Table)
        .and_where(Expr::col(Dish::UserId).eq(user_id))
        .order_by(Dish::Name, Order::Asc)
        .order_by(Dish::CreatedAt, Order::Asc)
        .to_owned();
    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(
        sqlx::query_as_with::<_, DishRow, _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(pool)
            .await?,
    )
}

impl<E: Executor> super::Module<E> {
    /// The user's dishes, by name.
    pub async fn list(&self, user_id: impl Into<String>) -> anyhow::Result<Vec<DishRow>> {
        list_rows(&self.read_db, &user_id.into()).await
    }

    /// One dish by id, from the read model.
    pub async fn find(&self, id: impl Into<String>) -> anyhow::Result<Option<DishRow>> {
        let statement = Query::select()
            .columns([
                Dish::Id,
                Dish::UserId,
                Dish::Name,
                Dish::RecipeIds,
                Dish::CreatedAt,
                Dish::UpdatedAt,
            ])
            .from(Dish::Table)
            .and_where(Expr::col(Dish::Id).eq(id.into()))
            .limit(1)
            .to_owned();
        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, DishRow, _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_optional(&self.read_db)
                .await?,
        )
    }

    /// Search the candidate pool: a page of `query.limit` recipes by name,
    /// optionally of one course and matching `query.search` (case-insensitive
    /// substring of the name), plus how many match in all. Drafts (unnamed
    /// recipes) and `query.exclude_ids` are left out. A library of hundreds
    /// of mains stays usable because the composer only ever shows a page.
    pub async fn candidates(
        &self,
        user_id: impl Into<String>,
        query: CandidateQuery,
    ) -> anyhow::Result<CandidatePage> {
        let user_id = user_id.into();
        let filter = |statement: &mut sea_query::SelectStatement| {
            statement
                .from(MealPlanRecipe::Table)
                .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id.as_str()))
                .and_where(Expr::col(MealPlanRecipe::Name).not_equals(""));
            if let Some(recipe_type) = &query.recipe_type {
                statement
                    .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()));
            }
            if let Some(search) = query
                .search
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let escaped = search
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_");
                statement.and_where(
                    Expr::col(MealPlanRecipe::Name)
                        .like(sea_query::LikeExpr::new(format!("%{escaped}%")).escape('\\')),
                );
            }
            if !query.exclude_ids.is_empty() {
                statement
                    .and_where(Expr::col(MealPlanRecipe::Id).is_not_in(query.exclude_ids.clone()));
            }
        };

        let mut count = Query::select();
        count.expr(Expr::col(MealPlanRecipe::Id).count());
        filter(&mut count);
        let (sql, values) = count.build_sqlx(SqliteQueryBuilder);
        let total = sqlx::query_scalar_with::<_, i64, _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_one(&self.read_db)
            .await?;

        let mut statement = Query::select();
        statement.columns([
            MealPlanRecipe::Id,
            MealPlanRecipe::Name,
            MealPlanRecipe::RecipeType,
            MealPlanRecipe::AcceptsAccompaniment,
        ]);
        filter(&mut statement);
        statement
            .order_by(MealPlanRecipe::RecipeType, Order::Asc)
            .order_by(MealPlanRecipe::Name, Order::Asc)
            .limit(query.limit.clamp(1, 100) as u64);
        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        let items = sqlx::query_as_with::<_, Candidate, _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(&self.read_db)
            .await?;

        Ok(CandidatePage {
            items,
            total: u64::try_from(total).unwrap_or(0),
        })
    }

    /// The candidates among `ids` (ids outside the user's pool are absent).
    pub async fn candidates_by_ids(
        &self,
        user_id: impl Into<String>,
        ids: Vec<String>,
    ) -> anyhow::Result<Vec<Candidate>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let statement = Query::select()
            .columns([
                MealPlanRecipe::Id,
                MealPlanRecipe::Name,
                MealPlanRecipe::RecipeType,
                MealPlanRecipe::AcceptsAccompaniment,
            ])
            .from(MealPlanRecipe::Table)
            .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id.into()))
            .and_where(Expr::col(MealPlanRecipe::Id).is_in(ids))
            .to_owned();
        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, Candidate, _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_all(&self.read_db)
                .await?,
        )
    }
}
