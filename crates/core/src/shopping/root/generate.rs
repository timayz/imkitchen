use evento::{Executor, ProjectionAggregator};
use imkitchen_db::shopping_recipe::ShoppingRecipe;
use imkitchen_db::shopping_slot::ShoppingSlot;
use imkitchen_types::{recipe::Ingredient, shopping::Generated};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use std::collections::{HashMap, HashSet};
use validator::Validate;

#[derive(Validate)]
pub struct Generate {
    pub date: u64,
    #[validate(range(min = 1, max = 30))]
    pub days: u8,
    pub household_size: u16,
}

impl<E: Executor> super::Module<E> {
    pub async fn generate(
        &self,
        input: Generate,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        input.validate()?;
        let request_by = request_by.into();
        let shopping = self
            .load(&request_by)
            .await?
            .unwrap_or_else(|| super::Shopping {
                user_id: request_by.to_owned(),
                checked: Default::default(),
                ingredients: Default::default(),
                cursor: Default::default(),
                generated_at: 0,
            });

        let slots_recipe_ids = self
            .filter_slot_recipe_ids(input.date, &request_by, input.days.into())
            .await?;

        let recipe_ingredients = self
            .filter_recipe_ingredients_by_ids(slots_recipe_ids)
            .await?;

        let mut ingredients = HashMap::new();
        for recipe_ingredients in recipe_ingredients {
            for ingredient in recipe_ingredients {
                let entry = ingredients.entry(ingredient.key()).or_insert(Ingredient {
                    name: ingredient.name,
                    quantity: 0,
                    unit: ingredient.unit,
                    category: ingredient.category,
                });

                entry.quantity += ((input.household_size as u32 * ingredient.quantity
                    / input.household_size as u32) as f64)
                    .ceil() as u32;
            }
        }

        shopping
            .aggregator()?
            .event(&Generated {
                ingredients: ingredients.values().cloned().collect(),
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    async fn filter_recipe_ingredients_by_ids(
        &self,
        ids: Vec<String>,
    ) -> anyhow::Result<Vec<Vec<Ingredient>>> {
        let statement = Query::select()
            .column(ShoppingRecipe::Ingredients)
            .from(ShoppingRecipe::Table)
            .and_where(Expr::col(ShoppingRecipe::Id).is_in(ids))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        Ok(
            sqlx::query_as_with::<_, (evento::sql_types::Bitcode<Vec<Ingredient>>,), _>(
                &sql, values,
            )
            .fetch_all(&self.read_db)
            .await?
            .into_iter()
            .map(|i| i.0.0)
            .collect(),
        )
    }

    async fn filter_slot_recipe_ids(
        &self,
        date: u64,
        user_id: impl Into<String>,
        limit: u64,
    ) -> anyhow::Result<Vec<String>> {
        let user_id = user_id.into();
        let statement = sea_query::Query::select()
            .column(ShoppingSlot::RecipeIds)
            .from(ShoppingSlot::Table)
            .and_where(Expr::col(ShoppingSlot::UserId).eq(&user_id))
            .and_where(Expr::col(ShoppingSlot::Date).gte(date))
            .limit(limit)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, (evento::sql_types::Bitcode<Vec<String>>,), _>(&sql, values)
                .fetch_all(&self.read_db)
                .await?
                .into_iter()
                .flat_map(|ids| ids.0.0)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
        )
    }
}
