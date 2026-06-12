use evento::{
    Cursor, Executor, Projection, Snapshot,
    cursor::{Args, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::mealplan_recipe::MealPlanRecipe;
use imkitchen_db::recipe_user::{RecipeUser, RecipeUserFts};
use imkitchen_types::recipe::{
    AdvancePrepChanged, BasicInformationChanged, Created, Deleted, DietaryRestriction,
    DietaryRestrictionsChanged, Imported, Ingredient, IngredientsChanged, Instruction,
    InstructionsChanged, MadePrivate, MainCourseOptionsChanged, Recipe, RecipeType,
    RecipeTypeChanged, SharedToCommunity, ThumbnailResized,
};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::{SqlitePool, prelude::FromRow};
use strum::{Display, EnumString};

#[derive(Default, Debug, Deserialize, EnumString, Display, Clone)]
pub enum SortBy {
    #[default]
    RecentlyAdded,
    Easiest,
    Hardest,
}

#[evento::projection(FromRow)]
pub struct UserView {
    pub id: String,
    pub owner_id: String,
    pub owner_name: Option<String>,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub name: String,
    pub slug: String,
    pub origin: Option<String>,
    pub description: String,
    pub household_size: u16,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: evento::sql_types::Bitcode<Vec<Ingredient>>,
    pub instructions: evento::sql_types::Bitcode<Vec<Instruction>>,
    pub dietary_restrictions: sqlx::types::Json<Vec<DietaryRestriction>>,
    pub accepts_accompaniment: bool,
    pub advance_prep: String,
    pub is_shared: bool,
    pub difficulty_score: u16,
    pub created_at: u64,
    pub thumbnail_version: Option<String>,
}

#[derive(Debug, Default, Clone, FromRow, Cursor)]
pub struct UserViewList {
    #[cursor(RecipeUser::Id, 1)]
    #[cursor(by_difficulty, RecipeUser::Id, 1)]
    pub id: String,
    pub owner_id: String,
    pub owner_name: Option<String>,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub dietary_restrictions: sqlx::types::Json<Vec<DietaryRestriction>>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
    #[cursor(by_difficulty, RecipeUser::DifficultyScore, 2)]
    pub difficulty_score: u16,
    #[cursor(RecipeUser::CreatedAt, 2)]
    pub created_at: u64,
    pub thumbnail_version: Option<String>,
}

pub struct RecipesQuery {
    pub exclude_ids: Option<Vec<String>>,
    pub user_id: Option<String>,
    pub recipe_type: Option<RecipeType>,
    pub is_shared: Option<bool>,
    pub has_thumbnail: Option<bool>,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub dietary_where_any: bool,
    pub in_meal_plan: Option<(String, bool)>,
    pub sort_by: SortBy,
    pub search: Option<String>,
    pub args: Args,
}

impl<E: Executor> crate::recipe::Module<E> {
    pub async fn filter_user(
        &self,
        query: RecipesQuery,
    ) -> anyhow::Result<ReadResult<UserViewList>> {
        let mut statement = sea_query::Query::select()
            .columns([
                RecipeUser::Id,
                RecipeUser::OwnerId,
                RecipeUser::OwnerName,
                RecipeUser::RecipeType,
                RecipeUser::Name,
                RecipeUser::Slug,
                RecipeUser::Description,
                RecipeUser::PrepTime,
                RecipeUser::CookTime,
                RecipeUser::DietaryRestrictions,
                RecipeUser::AcceptsAccompaniment,
                RecipeUser::IsShared,
                RecipeUser::DifficultyScore,
                RecipeUser::CreatedAt,
                RecipeUser::ThumbnailVersion,
            ])
            .from(RecipeUser::Table)
            .to_owned();

        if let Some(user_id) = query.user_id {
            statement.and_where(Expr::col(RecipeUser::OwnerId).eq(user_id));
        }

        if let Some(is_shared) = query.is_shared {
            statement.and_where(Expr::col(RecipeUser::IsShared).eq(is_shared));
        }

        match query.has_thumbnail {
            Some(true) => {
                statement.and_where(Expr::col(RecipeUser::ThumbnailVersion).is_not_null());
            }
            Some(false) => {
                statement.and_where(Expr::col(RecipeUser::ThumbnailVersion).is_null());
            }
            None => {}
        }

        if let Some(search) = query.search.filter(|s| !s.is_empty()) {
            statement.and_where(
                Expr::col(RecipeUser::Id).in_subquery(
                    Query::select()
                        .column(RecipeUserFts::Id)
                        .from(RecipeUserFts::Table)
                        .and_where(Expr::cust_with_values(
                            "recipe_user_fts MATCH ?",
                            [format!("{search}*")],
                        ))
                        .order_by(RecipeUserFts::Rank, sea_query::Order::Asc)
                        .limit(20)
                        .take(),
                ),
            );
        }

        if let Some(exclude_ids) = query.exclude_ids {
            statement.and_where(Expr::col(RecipeUser::Id).is_not_in(exclude_ids));
        }

        if let Some(recipe_type) = query.recipe_type {
            statement.and_where(Expr::col(RecipeUser::RecipeType).eq(recipe_type.to_string()));
        }

        if !query.dietary_restrictions.is_empty() {
            let in_clause = query
                .dietary_restrictions
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");

            let mut values: Vec<sea_query::Value> = query
                .dietary_restrictions
                .iter()
                .map(|t| sea_query::Value::String(Some(t.to_string())))
                .collect();

            let sql = if query.dietary_where_any {
                format!(
                    "(SELECT COUNT(*) FROM json_each(dietary_restrictions) WHERE value IN ({})) > 0",
                    in_clause
                )
            } else {
                values.push(sea_query::Value::Int(Some(
                    query.dietary_restrictions.len() as i32,
                )));
                format!(
                    "(SELECT COUNT(*) FROM json_each(dietary_restrictions) WHERE value IN ({})) = ?",
                    in_clause
                )
            };

            statement.and_where(Expr::cust_with_values(sql, values));
        }

        if let Some((meal_plan_user_id, in_plan)) = query.in_meal_plan {
            let subquery = Query::select()
                .expr(Expr::val(1))
                .from(MealPlanRecipe::Table)
                .and_where(
                    Expr::col((MealPlanRecipe::Table, MealPlanRecipe::Id))
                        .equals((RecipeUser::Table, RecipeUser::Id)),
                )
                .and_where(
                    Expr::col((MealPlanRecipe::Table, MealPlanRecipe::UserId))
                        .eq(meal_plan_user_id),
                )
                .take();

            if in_plan {
                statement.and_where(Expr::exists(subquery));
            } else {
                statement.and_where(Expr::exists(subquery).not());
            }
        }

        statement.and_where(Expr::col(RecipeUser::Name).not_equals(""));

        match query.sort_by {
            SortBy::RecentlyAdded => {
                Reader::new(statement)
                    .desc()
                    .args(query.args)
                    .execute(&self.read_db)
                    .await
            }
            SortBy::Easiest => {
                let result = Reader::new(statement)
                    .args(query.args)
                    .execute::<_, UserViewListByDifficulty, _>(&self.read_db)
                    .await?;

                Ok(result.map(|item| item.0))
            }
            SortBy::Hardest => {
                let result = Reader::new(statement)
                    .desc()
                    .args(query.args)
                    .execute::<_, UserViewListByDifficulty, _>(&self.read_db)
                    .await?;

                Ok(result.map(|item| item.0))
            }
        }
    }

    pub async fn find_user(&self, id: impl Into<String>) -> anyhow::Result<Option<UserView>> {
        find_user(&self.read_db, id).await
    }

    /// Resolves a recipe slug to its id. Returns `None` when no recipe carries
    /// the slug. See [`slugify`] for how slugs are derived.
    pub async fn find_id_by_slug(&self, slug: impl Into<String>) -> anyhow::Result<Option<String>> {
        let statement = sea_query::Query::select()
            .columns([RecipeUser::Id])
            .from(RecipeUser::Table)
            .and_where(Expr::col(RecipeUser::Slug).eq(slug.into()))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, (String,), _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_optional(&self.read_db)
                .await?
                .map(|(id,)| id),
        )
    }

    pub async fn find_user_draft(
        &self,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<String>> {
        let statement = sea_query::Query::select()
            .columns([RecipeUser::Id])
            .from(RecipeUser::Table)
            .and_where(Expr::col(RecipeUser::OwnerId).eq(user_id.into()))
            .and_where(Expr::col(RecipeUser::Name).eq(""))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, (String,), _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_optional(&self.read_db)
                .await?
                .map(|(id,)| id),
        )
    }

    pub async fn find_user_to_upsert(
        &self,
        user_id: impl Into<String>,
        origin: Option<&str>,
        name: &str,
    ) -> anyhow::Result<Option<String>> {
        let mut statement = sea_query::Query::select()
            .columns([RecipeUser::Id])
            .from(RecipeUser::Table)
            .and_where(Expr::col(RecipeUser::OwnerId).eq(user_id.into()))
            .to_owned();

        match origin {
            Some(origin) if !origin.is_empty() => {
                statement.and_where(Expr::col(RecipeUser::Origin).eq(origin));
            }
            _ => {
                statement
                    .and_where(Expr::col(RecipeUser::Name).eq(name))
                    .and_where(Expr::col(RecipeUser::Origin).is_null());
            }
        }

        statement
            .order_by(RecipeUser::CreatedAt, sea_query::Order::Desc)
            .limit(1);

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, (String,), _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_optional(&self.read_db)
                .await?
                .map(|(id,)| id),
        )
    }

    /// Maps a batch of recipe ids to their current slugs. Ids without a row are
    /// simply absent from the result, so callers should fall back to the id.
    pub async fn slugs(
        &self,
        ids: Vec<String>,
    ) -> anyhow::Result<std::collections::HashMap<String, String>> {
        if ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let statement = sea_query::Query::select()
            .columns([RecipeUser::Id, RecipeUser::Slug])
            .from(RecipeUser::Table)
            .and_where(Expr::col(RecipeUser::Id).is_in(ids))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(
            sqlx::query_as_with::<_, (String, String), _>(sqlx::AssertSqlSafe(sql), values)
                .fetch_all(&self.read_db)
                .await?
                .into_iter()
                .collect(),
        )
    }
}

async fn find_user(pool: &SqlitePool, id: impl Into<String>) -> anyhow::Result<Option<UserView>> {
    let statement = sea_query::Query::select()
        .columns([
            RecipeUser::Id,
            RecipeUser::Cursor,
            RecipeUser::OwnerId,
            RecipeUser::OwnerName,
            RecipeUser::RecipeType,
            RecipeUser::Name,
            RecipeUser::Slug,
            RecipeUser::Origin,
            RecipeUser::Description,
            RecipeUser::HouseholdSize,
            RecipeUser::PrepTime,
            RecipeUser::CookTime,
            RecipeUser::Ingredients,
            RecipeUser::Instructions,
            RecipeUser::DietaryRestrictions,
            RecipeUser::AcceptsAccompaniment,
            RecipeUser::AdvancePrep,
            RecipeUser::IsShared,
            RecipeUser::DifficultyScore,
            RecipeUser::CreatedAt,
            RecipeUser::ThumbnailVersion,
        ])
        .from(RecipeUser::Table)
        .and_where(Expr::col(RecipeUser::Id).eq(id.into()))
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with(sqlx::AssertSqlSafe(sql), values)
        .fetch_optional(pool)
        .await?)
}

/// Turns a recipe name into a URL-friendly slug: lowercased, with every run of
/// non-alphanumeric characters collapsed into a single hyphen and no leading or
/// trailing hyphen (e.g. `"Arroz con Pollo!"` becomes `"arroz-con-pollo"`).
pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut pending_dash = false;

    for ch in name.chars() {
        if ch.is_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.extend(ch.to_lowercase());
        } else {
            pending_dash = true;
        }
    }

    slug
}

/// Builds the slug stored for a recipe. The base slug comes from the recipe
/// name; if another recipe already owns that slug, the last six characters of
/// this recipe's id are appended to keep it unique. Nameless drafts fall back to
/// the id so the column stays unique and non-empty.
async fn build_slug(write_db: &SqlitePool, id: &str, name: &str) -> anyhow::Result<String> {
    let base = slugify(name);
    if base.is_empty() {
        return Ok(id.to_lowercase());
    }

    let taken = sqlx::query_scalar::<_, String>(
        "SELECT id FROM recipe_user WHERE slug = ? AND id <> ? LIMIT 1",
    )
    .bind(&base)
    .bind(id)
    .fetch_optional(write_db)
    .await?
    .is_some();

    if taken {
        let suffix = &id[id.len().saturating_sub(6)..];
        Ok(format!("{base}-{}", suffix.to_lowercase()))
    } else {
        Ok(base)
    }
}

pub fn create_projection<E: Executor>() -> Projection<E, UserView> {
    Projection::new::<Recipe>()
        .tombstone::<Deleted>()
        .handler(handle_created())
        .handler(handle_imported())
        .handler(handle_recipe_type_changed())
        .handler(handle_basic_information_changed())
        .handler(handle_ingredients_changed())
        .handler(handle_instructions_changed())
        .handler(handle_dietary_restrictions_changed())
        .handler(handle_main_course_options_changed())
        .handler(handle_advance_prep_changed())
        .handler(handle_shared_to_community())
        .handler(handle_made_private())
        .handler(handle_thumbnail_resized())
}

impl<E: Executor> crate::recipe::Module<E> {
    pub async fn user(&self, id: impl Into<String>) -> Result<Option<UserView>, anyhow::Error> {
        load(&self.executor, &self.read_db, &self.write_db, id).await
    }
}

pub(crate) async fn load<E: Executor>(
    executor: &E,
    read_db: &SqlitePool,
    write_db: &SqlitePool,
    id: impl Into<String>,
) -> Result<Option<UserView>, anyhow::Error> {
    create_projection()
        .data((read_db.clone(), write_db.clone()))
        .load(id)
        .execute(executor)
        .await
}

impl<E: Executor> Snapshot<E> for UserView {
    async fn restore(context: &evento::projection::Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let (read_db, _) = context.extract::<(SqlitePool, SqlitePool)>();
        find_user(&read_db, &context.id).await
    }

    async fn take_snapshot(
        &self,
        context: &evento::projection::Context<'_, E>,
    ) -> anyhow::Result<()> {
        let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();

        let slug = build_slug(&write_db, &self.id, &self.name).await?;

        let ingredients = bitcode::encode(&self.ingredients.0);
        let instructions = bitcode::encode(&self.instructions.0);
        let difficulty_score: u16 =
            self.prep_time + self.cook_time + (self.instructions.0.len() as u16) * 3;
        let dietary_restrictions = self
            .dietary_restrictions
            .iter()
            .map(|d| serde_json::Value::String(d.to_string()))
            .collect::<Vec<_>>();

        let statement = sea_query::Query::insert()
            .into_table(RecipeUser::Table)
            .columns([
                RecipeUser::Id,
                RecipeUser::Cursor,
                RecipeUser::OwnerId,
                RecipeUser::OwnerName,
                RecipeUser::RecipeType,
                RecipeUser::Name,
                RecipeUser::Slug,
                RecipeUser::Origin,
                RecipeUser::Description,
                RecipeUser::HouseholdSize,
                RecipeUser::PrepTime,
                RecipeUser::CookTime,
                RecipeUser::Ingredients,
                RecipeUser::Instructions,
                RecipeUser::DietaryRestrictions,
                RecipeUser::AcceptsAccompaniment,
                RecipeUser::AdvancePrep,
                RecipeUser::IsShared,
                RecipeUser::DifficultyScore,
                RecipeUser::CreatedAt,
                RecipeUser::ThumbnailVersion,
            ])
            .values([
                self.id.to_owned().into(),
                self.cursor.to_owned().into(),
                self.owner_id.to_owned().into(),
                self.owner_name.to_owned().into(),
                self.recipe_type.to_string().into(),
                self.name.to_owned().into(),
                slug.to_owned().into(),
                self.origin.to_owned().into(),
                self.description.to_owned().into(),
                self.household_size.into(),
                self.prep_time.into(),
                self.cook_time.into(),
                ingredients.into(),
                instructions.into(),
                serde_json::Value::Array(dietary_restrictions).into(),
                self.accepts_accompaniment.into(),
                self.advance_prep.to_owned().into(),
                self.is_shared.into(),
                difficulty_score.into(),
                self.created_at.into(),
                self.thumbnail_version.to_owned().into(),
            ])?
            .on_conflict(
                OnConflict::column(RecipeUser::Id)
                    .update_columns([
                        RecipeUser::Cursor,
                        RecipeUser::OwnerId,
                        RecipeUser::OwnerName,
                        RecipeUser::RecipeType,
                        RecipeUser::Name,
                        RecipeUser::Slug,
                        RecipeUser::Origin,
                        RecipeUser::Description,
                        RecipeUser::HouseholdSize,
                        RecipeUser::PrepTime,
                        RecipeUser::CookTime,
                        RecipeUser::Ingredients,
                        RecipeUser::Instructions,
                        RecipeUser::DietaryRestrictions,
                        RecipeUser::AcceptsAccompaniment,
                        RecipeUser::AdvancePrep,
                        RecipeUser::IsShared,
                        RecipeUser::DifficultyScore,
                        RecipeUser::CreatedAt,
                        RecipeUser::ThumbnailVersion,
                    ])
                    .to_owned(),
            )
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
            .execute(&write_db)
            .await?;

        Ok(())
    }

    async fn drop_snapshot(context: &evento::projection::Context<'_, E>) -> anyhow::Result<()> {
        let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
        let (sql, values) = Query::delete()
            .from_table(RecipeUser::Table)
            .and_where(Expr::col(RecipeUser::Id).eq(&context.id))
            .build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(sqlx::AssertSqlSafe(sql), values)
            .execute(&write_db)
            .await?;
        Ok(())
    }
}

#[evento::handler]
async fn handle_created(event: Event<Created>, data: &mut UserView) -> anyhow::Result<()> {
    data.owner_id = event.metadata.requested_by()?;
    data.owner_name = event.data.owner_name.to_owned();
    data.created_at = event.timestamp;
    data.id = event.aggregator_id.to_owned();
    data.name = event.data.name;
    data.household_size = 4;

    Ok(())
}

#[evento::handler]
async fn handle_imported(event: Event<Imported>, data: &mut UserView) -> anyhow::Result<()> {
    data.created_at = event.timestamp;
    data.owner_id = event.metadata.requested_by()?;
    data.owner_name = event.data.owner_name.to_owned();
    data.id = event.aggregator_id.to_owned();
    data.name = event.data.name;
    data.origin = event.data.origin;
    data.description = event.data.description;
    data.recipe_type.0 = event.data.recipe_type;
    data.prep_time = event.data.prep_time;
    data.cook_time = event.data.cook_time;
    data.advance_prep = event.data.advance_prep;
    data.ingredients.0 = event.data.ingredients;
    data.instructions.0 = event.data.instructions;
    data.household_size = event.data.household_size;
    data.accepts_accompaniment = event.data.accepts_accompaniment;
    data.dietary_restrictions.0 = event.data.dietary_restrictions;

    Ok(())
}

#[evento::handler]
async fn handle_recipe_type_changed(
    event: Event<RecipeTypeChanged>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.recipe_type.0 = event.data.recipe_type;

    Ok(())
}

#[evento::handler]
async fn handle_basic_information_changed(
    event: Event<BasicInformationChanged>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.name = event.data.name;
    data.origin = event.data.origin;
    data.description = event.data.description;
    data.household_size = event.data.household_size;
    data.prep_time = event.data.prep_time;
    data.cook_time = event.data.cook_time;

    Ok(())
}

#[evento::handler]
async fn handle_ingredients_changed(
    event: Event<IngredientsChanged>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.ingredients.0 = event.data.ingredients;

    Ok(())
}

#[evento::handler]
async fn handle_instructions_changed(
    event: Event<InstructionsChanged>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.instructions.0 = event.data.instructions;

    Ok(())
}

#[evento::handler]
async fn handle_dietary_restrictions_changed(
    event: Event<DietaryRestrictionsChanged>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.dietary_restrictions.0 = event.data.dietary_restrictions;

    Ok(())
}

#[evento::handler]
async fn handle_main_course_options_changed(
    event: Event<MainCourseOptionsChanged>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.accepts_accompaniment = event.data.accepts_accompaniment;

    Ok(())
}

#[evento::handler]
async fn handle_advance_prep_changed(
    event: Event<AdvancePrepChanged>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.advance_prep = event.data.advance_prep;

    Ok(())
}

#[evento::handler]
async fn handle_shared_to_community(
    event: Event<SharedToCommunity>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.is_shared = true;
    data.owner_name = Some(event.data.owner_name);

    Ok(())
}

#[evento::handler]
async fn handle_made_private(
    _event: Event<MadePrivate>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.is_shared = false;

    Ok(())
}

#[evento::handler]
async fn handle_thumbnail_resized(
    event: Event<ThumbnailResized>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.thumbnail_version = Some(event.id.to_string());

    Ok(())
}
