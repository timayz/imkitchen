use evento::Executor;
use evento::cursor::Args;
use evento::{Aggregator, ReadAggregator};
use imkitchen_db::table::MealPlanRecipe;
use imkitchen_shared::mealplan::{MealPlan, Slot, SlotRecipe, WeekGenerated};
use imkitchen_shared::recipe::{DietaryRestriction, RecipeType};
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
    pub dietary_restrictions: Vec<imkitchen_shared::recipe::DietaryRestriction>,
}

pub struct Generate {
    pub user_id: String,
    pub weeks: Vec<(u64, u64)>,
    pub randomize: Option<Randomize>,
    pub household_size: u16,
}

impl<E: Executor> super::Command<E> {
    pub async fn generate(&self, input: Generate) -> imkitchen_shared::Result<()> {
        let main_course_recipes = match input.randomize.as_ref() {
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
            imkitchen_shared::user!("No main course found");
        }

        let last_event = self
            .executor
            .read(
                Some(vec![ReadAggregator::id(
                    MealPlan::aggregator_type(),
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

        let mut main_course_recipes = main_course_recipes.iter().cycle().take(7 * 4);
        let mut builder = evento::aggregator(&input.user_id)
            .original_version(version)
            .requested_by(&input.user_id)
            .to_owned();

        for (start, end) in input.weeks {
            let mut slots = vec![];

            while let Some(recipe) = main_course_recipes.by_ref().next() {
                let day = OffsetDateTime::from_unix_timestamp(start as i64)?
                    + Duration::days((slots.len()) as i64);

                let appetizer_recipes = match input.randomize.as_ref() {
                    Some(opts) => {
                        self.random(
                            &input.user_id,
                            RecipeType::Appetizer,
                            1.0,
                            opts.dietary_restrictions.to_vec(),
                        )
                        .await?
                    }
                    _ => {
                        self.first_week_recipes(&input.user_id, RecipeType::Appetizer)
                            .await?
                    }
                };

                let mut appetizer_recipes = appetizer_recipes.iter();

                let accompaniment_recipes = match input.randomize.as_ref() {
                    Some(opts) => {
                        self.random(
                            &input.user_id,
                            RecipeType::Accompaniment,
                            1.0,
                            opts.dietary_restrictions.to_vec(),
                        )
                        .await?
                    }
                    _ => {
                        self.first_week_recipes(&input.user_id, RecipeType::Accompaniment)
                            .await?
                    }
                };

                let mut accompaniment_recipes = accompaniment_recipes.iter();

                let dessert_recipes = match input.randomize.as_ref() {
                    Some(opts) => {
                        self.random(
                            &input.user_id,
                            RecipeType::Dessert,
                            1.0,
                            opts.dietary_restrictions.to_vec(),
                        )
                        .await?
                    }
                    _ => {
                        self.first_week_recipes(&input.user_id, RecipeType::Dessert)
                            .await?
                    }
                };
                let mut dessert_recipes = dessert_recipes.iter();

                let accompaniment = if recipe.accepts_accompaniment {
                    accompaniment_recipes.next().map(|r| r.into())
                } else {
                    None
                };

                slots.push(Slot {
                    day: day.unix_timestamp() as u64,
                    appetizer: appetizer_recipes.next().map(|r| r.into()),
                    main_course: recipe.into(),
                    dessert: dessert_recipes.next().map(|r| r.into()),
                    accompaniment,
                });

                if slots.len() == 7 {
                    break;
                }
            }

            if slots.is_empty() {
                break;
            }

            builder.event(&WeekGenerated {
                slots,
                start,
                end,
                household_size: input.household_size,
            });
        }

        builder.commit(&self.executor).await?;

        Ok(())
    }

    pub async fn first_week_recipes(
        &self,
        id: impl Into<String>,
        recipe_type: RecipeType,
    ) -> imkitchen_shared::Result<Vec<Recipe>> {
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

        let row = sqlx::query_as_with::<_, Recipe, _>(&sql, values)
            .fetch_all(&self.read_db)
            .await?;

        let mut recipes = vec![];

        for _ in 0..3 {
            let mut rng = rand::rng();
            let mut r = row.to_vec();
            r.shuffle(&mut rng);
            recipes.extend(r);
        }

        Ok(recipes)
    }

    async fn random(
        &self,
        id: impl Into<String>,
        recipe_type: RecipeType,
        weight: f32,
        dietary_restrictions: Vec<DietaryRestriction>,
    ) -> imkitchen_shared::Result<Vec<Recipe>> {
        if weight < 0.1 {
            imkitchen_shared::user!("weight must be greater than or equal to 0.1");
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
            .limit(7 * 4);

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

        let mut recipes = sqlx::query_as_with::<_, Recipe, _>(&sql, values)
            .fetch_all(&self.read_db)
            .await?;

        let mut rng = rand::rng();
        recipes.shuffle(&mut rng);
        recipes.truncate((recipes.len() as f32 * weight).ceil() as usize);

        Ok(recipes)
    }
}
