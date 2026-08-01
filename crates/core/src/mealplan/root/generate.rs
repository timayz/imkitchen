use evento::Executor;
use evento::cursor::Args;
use evento::{Aggregate, EventFilter};
use imkitchen_db::mealplan_recipe::MealPlanRecipe;
use imkitchen_types::mealplan::{DaysGenerated, MealPlan, Slot, SlotRecipe};
use imkitchen_types::recipe::{DietaryRestriction, RecipeType};
use rand::seq::SliceRandom;
use sea_query::{Expr, ExprTrait, Func, IntoColumnRef, Query, SimpleExpr, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;
use time::{Duration, OffsetDateTime};

#[derive(Clone, FromRow)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub accepts_accompaniment: bool,
}

impl From<&Recipe> for SlotRecipe {
    fn from(value: &Recipe) -> Self {
        SlotRecipe {
            id: value.id.to_owned(),
            name: value.name.to_owned(),
        }
    }
}

pub struct Randomize {
    pub cuisine_variety_weight: f32,
    pub dietary_restrictions: Vec<imkitchen_types::recipe::DietaryRestriction>,
    /// Optional courses the user enabled. `MainCourse` is always generated and
    /// is ignored if it appears here.
    pub recipe_types: Vec<RecipeType>,
}

pub struct Generate {
    pub user_id: String,
    pub start: u64,
    pub days: u8,
    pub randomize: Option<Randomize>,
    pub household_size: u16,
}

impl<E: Executor> super::Module<E> {
    pub async fn generate(&self, input: Generate) -> crate::Result<()> {
        let randomize = input.randomize.as_ref();

        let main_course_recipes = match randomize {
            Some(opts) => {
                self.random(
                    &input.user_id,
                    RecipeType::MainCourse,
                    opts.cuisine_variety_weight,
                    opts.dietary_restrictions.to_vec(),
                )
                .await?
            }
            _ => {
                self.first_week_recipes(&input.user_id, RecipeType::MainCourse)
                    .await?
            }
        };

        if main_course_recipes.is_empty() {
            crate::user!("No main course found");
        }

        let last_event = self
            .executor
            .read(
                Some(vec![EventFilter::by_id(
                    MealPlan::aggregate_type(),
                    &input.user_id,
                )]),
                None,
                Args::backward(1, None),
            )
            .await?;

        let version = last_event
            .edges
            .first()
            .map(|e| e.node.version)
            .unwrap_or_default();

        let mut main_course_recipes = main_course_recipes.iter().cycle().take(input.days as usize);
        let mut builder = evento::append(&input.user_id)
            .original_version(version)
            .requested_by(&input.user_id)
            .to_owned();

        // One query per enabled course for the whole plan instead of one per
        // course per day — a 31-day month used to issue 1 + 3 * 31 = 94 queries.
        // Disabled courses cost nothing.
        let appetizers = self
            .optional_pool(&input.user_id, RecipeType::Appetizer, randomize)
            .await?;
        let accompaniments = self
            .optional_pool(&input.user_id, RecipeType::Accompaniment, randomize)
            .await?;
        let desserts = self
            .optional_pool(&input.user_id, RecipeType::Dessert, randomize)
            .await?;
        let beverages = self
            .optional_pool(&input.user_id, RecipeType::Beverage, randomize)
            .await?;
        let condiments = self
            .optional_pool(&input.user_id, RecipeType::Condiment, randomize)
            .await?;

        let mut appetizers = appetizers.iter().cycle();
        let mut accompaniments = accompaniments.iter().cycle();
        let mut desserts = desserts.iter().cycle();
        let mut beverages = beverages.iter().cycle();
        let mut condiments = condiments.iter().cycle();

        let mut slots = vec![];

        while let Some(recipe) = main_course_recipes.by_ref().next() {
            let day = OffsetDateTime::from_unix_timestamp(input.start as i64)?
                + Duration::days((slots.len()) as i64);

            let date = crate::mealplan::date_to_u64(day);

            slots.push(Slot {
                day: day.unix_timestamp() as u64,
                date,
                household_size: input.household_size,
                appetizer: appetizers.next().map(|r| r.into()),
                main_course: recipe.into(),
                dessert: desserts.next().map(|r| r.into()),
                accompaniment: if recipe.accepts_accompaniment {
                    accompaniments.next().map(|r| r.into())
                } else {
                    None
                },
                beverage: beverages.next().map(|r| r.into()),
                condiment: condiments.next().map(|r| r.into()),
            });
        }

        if slots.is_empty() {
            crate::user!("No slots generated");
        }

        builder.event(&DaysGenerated {
            slots,
            start: input.start,
            household_size: input.household_size,
        });

        builder.commit(&self.executor).await?;

        Ok(())
    }

    pub async fn first_week_recipes(
        &self,
        id: impl Into<String>,
        recipe_type: RecipeType,
    ) -> crate::Result<Vec<Recipe>> {
        let id = id.into();

        let statement = Query::select()
            .columns([
                MealPlanRecipe::Id,
                MealPlanRecipe::Name,
                MealPlanRecipe::AcceptsAccompaniment,
            ])
            .from(MealPlanRecipe::Table)
            .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
            .and_where(Expr::col(MealPlanRecipe::RecipeType).eq(recipe_type.to_string()))
            .and_where(Expr::col(MealPlanRecipe::Name).not_equals(""))
            .limit(7)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        let mut recipes = sqlx::query_as_with::<_, Recipe, _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(&self.read_db)
            .await?;

        let mut rng = rand::rng();
        recipes.shuffle(&mut rng);

        Ok(recipes)
    }

    /// Recipe pool for one optional course, or an empty pool when the course is
    /// disabled in the user's preferences or when generating without
    /// randomization. A disabled course issues no query at all.
    async fn optional_pool(
        &self,
        user_id: &str,
        recipe_type: RecipeType,
        randomize: Option<&Randomize>,
    ) -> crate::Result<Vec<Recipe>> {
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

    async fn random(
        &self,
        id: impl Into<String>,
        recipe_type: RecipeType,
        weight: f32,
        dietary_restrictions: Vec<DietaryRestriction>,
    ) -> crate::Result<Vec<Recipe>> {
        if weight < 0.1 {
            crate::user!("weight must be greater than or equal to 0.1");
        }

        let id = id.into();
        let mut sub_statement = Query::select()
            .columns([MealPlanRecipe::Id])
            .from(MealPlanRecipe::Table)
            .and_where(Expr::col(MealPlanRecipe::UserId).eq(id))
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

        let mut recipes = sqlx::query_as_with::<_, Recipe, _>(sqlx::AssertSqlSafe(sql), values)
            .fetch_all(&self.read_db)
            .await?;

        let mut rng = rand::rng();
        recipes.shuffle(&mut rng);
        recipes.truncate((recipes.len() as f32 * weight).ceil() as usize);

        Ok(recipes)
    }
}
