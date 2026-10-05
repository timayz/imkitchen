//! Random recipe picking from the user's candidate pool (`meal_plan_recipe`:
//! their own recipes plus saved community ones, maintained by
//! [`super::super::pool::subscription`]).

use evento::Executor;
use imkitchen_db::mealplan_recipe::MealPlanRecipe;
use imkitchen_types::recipe::{DietaryRestriction, RecipeType};
use rand::seq::SliceRandom;
use sea_query::{Expr, ExprTrait, Func, IntoColumnRef, Query, SimpleExpr, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

#[derive(Clone, Debug, FromRow)]
pub struct PoolRecipe {
    pub id: String,
    pub name: String,
    pub accepts_accompaniment: bool,
}

pub struct Randomize {
    pub cuisine_variety_weight: f32,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    /// Optional courses the user enabled. `MainCourse` is always generated and
    /// is ignored if it appears here.
    pub recipe_types: Vec<RecipeType>,
}

impl<E: Executor> super::Module<E> {
    /// Up to seven pool recipes of a type, shuffled. Used for the onboarding
    /// probes ("does the user have any main courses yet?") and for generating
    /// without randomization options.
    pub async fn sample_recipes(
        &self,
        user_id: impl Into<String>,
        recipe_type: RecipeType,
    ) -> crate::Result<Vec<PoolRecipe>> {
        let user_id = user_id.into();

        let statement = Query::select()
            .columns([
                MealPlanRecipe::Id,
                MealPlanRecipe::Name,
                MealPlanRecipe::AcceptsAccompaniment,
            ])
            .from(MealPlanRecipe::Table)
            .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id))
            .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
            .and_where(Expr::col(MealPlanRecipe::Name).not_equals(""))
            .limit(7)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        let mut recipes = sqlx::query_as_with::<_, PoolRecipe, _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(&self.read_db)
            .await?;

        let mut rng = rand::rng();
        recipes.shuffle(&mut rng);

        Ok(recipes)
    }

    /// Recipe pool for one optional course, or an empty pool when the course is
    /// disabled in the user's preferences or when generating without
    /// randomization. A disabled course issues no query at all.
    pub(crate) async fn optional_pool(
        &self,
        user_id: &str,
        recipe_type: RecipeType,
        randomize: Option<&Randomize>,
    ) -> crate::Result<Vec<PoolRecipe>> {
        let Some(opts) = randomize else {
            return Ok(vec![]);
        };

        if !opts.recipe_types.contains(&recipe_type) {
            return Ok(vec![]);
        }

        self.random(
            user_id,
            recipe_type,
            1.0,
            opts.dietary_restrictions.to_vec(),
        )
        .await
    }

    /// Up to 35 random pool recipes of a type matching every requested dietary
    /// restriction, then truncated to `weight` (0.1..=1.0) of that: a smaller
    /// weight means a smaller, more repetitive pool.
    pub(crate) async fn random(
        &self,
        user_id: impl Into<String>,
        recipe_type: RecipeType,
        weight: f32,
        dietary_restrictions: Vec<DietaryRestriction>,
    ) -> crate::Result<Vec<PoolRecipe>> {
        if weight < 0.1 {
            crate::user!("weight must be greater than or equal to 0.1");
        }

        let user_id = user_id.into();
        let mut sub_statement = Query::select()
            .columns([MealPlanRecipe::Id])
            .from(MealPlanRecipe::Table)
            .and_where(Expr::col(MealPlanRecipe::UserId).eq(user_id))
            .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
            .and_where(Expr::col(MealPlanRecipe::Name).not_equals(""))
            .to_owned();

        if !dietary_restrictions.is_empty() {
            let in_clause = dietary_restrictions
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");

            sub_statement.and_where(Expr::cust_with_values(
                format!(
                    "(SELECT COUNT(*) FROM json_each(dietary_restrictions) WHERE value IN ({})) = ?",
                    in_clause
                ),
                dietary_restrictions
                    .iter()
                    .map(|t| sea_query::Value::String(Some(*Box::new(t.to_string()))))
                    .chain(std::iter::once(sea_query::Value::Int(Some(
                        dietary_restrictions.len() as i32,
                    ))))
                    .collect::<Vec<_>>(),
            ));
        }

        sub_statement
            .order_by_expr(
                SimpleExpr::FunctionCall(Func::random()),
                sea_query::Order::Asc,
            )
            .limit(7 * 5);

        let statement = Query::select()
            .columns([
                MealPlanRecipe::Id,
                MealPlanRecipe::Name,
                MealPlanRecipe::AcceptsAccompaniment,
            ])
            .from(MealPlanRecipe::Table)
            .and_where(
                MealPlanRecipe::Id
                    .into_column_ref()
                    .in_subquery(sub_statement),
            )
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        let mut recipes = sqlx::query_as_with::<_, PoolRecipe, _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(&self.read_db)
            .await?;

        let mut rng = rand::rng();
        recipes.shuffle(&mut rng);
        recipes.truncate((recipes.len() as f32 * weight).ceil() as usize);

        Ok(recipes)
    }
}
