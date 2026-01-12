use evento::{
    Cursor, Executor, Projection, Snapshot,
    cursor::{Args, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::table::RecipeUser;
use imkitchen_shared::recipe::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged, Deleted,
    DietaryRestriction, DietaryRestrictionsChanged, Imported, Ingredient, IngredientsChanged,
    Instruction, InstructionsChanged, MadePrivate, MainCourseOptionsChanged, Recipe, RecipeType,
    RecipeTypeChanged, SharedToCommunity,
    rating::{LikeChecked, LikeUnchecked, UnlikeChecked, UnlikeUnchecked, Viewed},
};
use sea_query::{Expr, ExprTrait, OnConflict, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::{SqlitePool, prelude::FromRow};
use strum::{Display, EnumString};

#[derive(Default, Debug, Deserialize, EnumString, Display, Clone)]
pub enum SortBy {
    #[default]
    RecentlyAdded,
}

#[evento::projection(FromRow)]
pub struct UserView {
    pub id: String,
    pub owner_id: String,
    pub owner_name: Option<String>,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub cuisine_type: sqlx::types::Text<CuisineType>,
    pub name: String,
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
    pub total_views: u64,
    pub total_likes: i64,
    pub total_comments: u64,
    pub created_at: u64,
}

impl UserView {
    pub fn total_ulikes(&self) -> u64 {
        self.total_likes.try_into().unwrap_or(0)
    }
}

#[derive(Debug, Default, FromRow, Cursor)]
pub struct UserViewList {
    #[cursor(RecipeUser::Id, 1)]
    pub id: String,
    pub owner_id: String,
    pub owner_name: Option<String>,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub cuisine_type: sqlx::types::Text<CuisineType>,
    pub name: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub dietary_restrictions: sqlx::types::Json<Vec<DietaryRestriction>>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
    pub total_views: u64,
    #[cursor(RecipeUser::CreatedAt, 2)]
    pub created_at: u64,
}

pub struct RecipesQuery {
    pub exclude_ids: Option<Vec<String>>,
    pub user_id: Option<String>,
    pub recipe_type: Option<RecipeType>,
    pub cuisine_type: Option<CuisineType>,
    pub is_shared: Option<bool>,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub dietary_where_any: bool,
    pub sort_by: SortBy,
    pub args: Args,
}

impl<E: Executor> super::Query<E> {
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
                RecipeUser::CuisineType,
                RecipeUser::Name,
                RecipeUser::Description,
                RecipeUser::PrepTime,
                RecipeUser::CookTime,
                RecipeUser::DietaryRestrictions,
                RecipeUser::AcceptsAccompaniment,
                RecipeUser::IsShared,
                RecipeUser::TotalViews,
                RecipeUser::CreatedAt,
            ])
            .from(RecipeUser::Table)
            .to_owned();

        if let Some(user_id) = query.user_id {
            statement.and_where(Expr::col(RecipeUser::OwnerId).eq(user_id));
        }

        if let Some(is_shared) = query.is_shared {
            statement.and_where(Expr::col(RecipeUser::IsShared).eq(is_shared));
        }

        if let Some(exclude_ids) = query.exclude_ids {
            statement.and_where(Expr::col(RecipeUser::Id).is_not_in(exclude_ids));
        }

        if let Some(recipe_type) = query.recipe_type {
            statement.and_where(Expr::col(RecipeUser::RecipeType).eq(recipe_type.to_string()));
        }

        if let Some(cuisine_type) = query.cuisine_type {
            statement.and_where(Expr::col(RecipeUser::CuisineType).eq(cuisine_type.to_string()));
        }

        if !query.dietary_restrictions.is_empty() && !query.dietary_where_any {
            let in_clause = query
                .dietary_restrictions
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");

            statement.and_where(Expr::cust_with_values(
            format!(
                "(SELECT COUNT(*) FROM json_each(dietary_restrictions) WHERE value IN ({})) = ?",
                in_clause
            ),
            query
                .dietary_restrictions
                .iter()
                .map(|t| sea_query::Value::String(Some(*Box::new(t.to_string()))))
                .chain(std::iter::once(sea_query::Value::Int(Some(
                    query.dietary_restrictions.len() as i32,
                ))))
                .collect::<Vec<_>>(),
        ));
        }

        if !query.dietary_restrictions.is_empty() && query.dietary_where_any {
            let in_clause = query
                .dietary_restrictions
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");

            statement.and_where(Expr::cust_with_values(
                format!(
                    "(SELECT COUNT(*) FROM json_each(dietary_restrictions) WHERE value IN ({}))",
                    in_clause
                ),
                query
                    .dietary_restrictions
                    .iter()
                    .map(|t| sea_query::Value::String(Some(*Box::new(t.to_string()))))
                    .collect::<Vec<_>>(),
            ));
        }

        let mut reader = Reader::new(statement);

        if matches!(query.sort_by, SortBy::RecentlyAdded) {
            reader.desc();
        }

        reader.args(query.args).execute(&self.read_db).await
    }

    pub async fn find_user(&self, id: impl Into<String>) -> anyhow::Result<Option<UserView>> {
        find_user(&self.read_db, id).await
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
            RecipeUser::CuisineType,
            RecipeUser::Name,
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
            RecipeUser::TotalViews,
            RecipeUser::TotalLikes,
            RecipeUser::TotalComments,
            RecipeUser::CreatedAt,
        ])
        .from(RecipeUser::Table)
        .and_where(Expr::col(RecipeUser::Id).eq(id.into()))
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, UserView> {
    Projection::new::<Recipe>(id)
        .handler(handle_created())
        .handler(handle_imported())
        .handler(handle_recipe_type_changed())
        .handler(handle_basic_information_changed())
        .handler(handle_ingredients_changed())
        .handler(handle_instructions_changed())
        .handler(handle_dietary_restrictions_changed())
        .handler(handle_cuisine_type_changed())
        .handler(handle_main_course_options_changed())
        .handler(handle_advance_prep_changed())
        .handler(handle_shared_to_community())
        .handler(handle_made_private())
        .handler(handle_deleted())
        .handler(handle_viewed())
        .handler(handle_like_checked())
        .handler(handle_like_unchecked())
        .handler(handle_unlike_checked())
        .handler(handle_unlike_unchecked())
}

impl<E: Executor> super::Query<E> {
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
    create_projection(id)
        .data((read_db.clone(), write_db.clone()))
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
        let ingredients = bitcode::encode(&self.ingredients.0);
        let instructions = bitcode::encode(&self.instructions.0);
        let dietary_restrictions = self
            .dietary_restrictions
            .iter()
            .map(|d| serde_json::Value::String(d.to_string()))
            .collect::<Vec<_>>();

        let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();

        let statement = sea_query::Query::insert()
            .into_table(RecipeUser::Table)
            .columns([
                RecipeUser::Id,
                RecipeUser::Cursor,
                RecipeUser::OwnerId,
                RecipeUser::OwnerName,
                RecipeUser::RecipeType,
                RecipeUser::CuisineType,
                RecipeUser::Name,
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
                RecipeUser::TotalViews,
                RecipeUser::TotalLikes,
                RecipeUser::TotalComments,
                RecipeUser::CreatedAt,
            ])
            .values([
                self.id.to_owned().into(),
                self.cursor.to_owned().into(),
                self.owner_id.to_owned().into(),
                self.owner_name.to_owned().into(),
                self.recipe_type.to_string().into(),
                self.cuisine_type.to_string().into(),
                self.name.to_owned().into(),
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
                self.total_views.into(),
                self.total_likes.into(),
                self.total_comments.into(),
                self.created_at.into(),
            ])?
            .on_conflict(
                OnConflict::column(RecipeUser::Id)
                    .update_columns([
                        RecipeUser::Cursor,
                        RecipeUser::OwnerId,
                        RecipeUser::OwnerName,
                        RecipeUser::RecipeType,
                        RecipeUser::CuisineType,
                        RecipeUser::Name,
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
                        RecipeUser::TotalViews,
                        RecipeUser::TotalLikes,
                        RecipeUser::TotalComments,
                        RecipeUser::CreatedAt,
                    ])
                    .to_owned(),
            )
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(&write_db).await?;

        Ok(())
    }
}

#[evento::handler]
async fn handle_created(event: Event<Created>, data: &mut UserView) -> anyhow::Result<()> {
    data.owner_id = event.metadata.user()?;
    data.owner_name = event.data.owner_name.to_owned();
    data.created_at = event.timestamp;
    data.id = event.aggregator_id.to_owned();
    data.name = event.data.name;
    data.household_size = 4;

    Ok(())
}

#[evento::handler]
async fn handle_imported(event: Event<Imported>, data: &mut UserView) -> anyhow::Result<()> {
    data.owner_id = event.metadata.user()?;
    data.owner_name = event.data.owner_name.to_owned();
    data.id = event.aggregator_id.to_owned();
    data.name = event.data.name;
    data.description = event.data.description;
    data.recipe_type.0 = event.data.recipe_type;
    data.cuisine_type.0 = event.data.cuisine_type;
    data.prep_time = event.data.prep_time;
    data.cook_time = event.data.cook_time;
    data.advance_prep = event.data.advance_prep;
    data.ingredients.0 = event.data.ingredients;
    data.instructions.0 = event.data.instructions;
    data.household_size = event.data.household_size;

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
async fn handle_cuisine_type_changed(
    event: Event<CuisineTypeChanged>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.cuisine_type.0 = event.data.cuisine_type;

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
async fn handle_deleted(_event: Event<Deleted>, data: &mut UserView) -> anyhow::Result<()> {
    data.created_at = 0;

    Ok(())
}

#[evento::handler]
async fn handle_viewed(_event: Event<Viewed>, data: &mut UserView) -> anyhow::Result<()> {
    data.total_views += 1;

    Ok(())
}

#[evento::handler]
async fn handle_like_checked(
    _event: Event<LikeChecked>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.total_likes += 1;

    Ok(())
}

#[evento::handler]
async fn handle_like_unchecked(
    _event: Event<LikeUnchecked>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.total_likes -= 1;

    Ok(())
}

#[evento::handler]
async fn handle_unlike_checked(
    _event: Event<UnlikeChecked>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.total_likes -= 1;

    Ok(())
}

#[evento::handler]
async fn handle_unlike_unchecked(
    _event: Event<UnlikeUnchecked>,
    data: &mut UserView,
) -> anyhow::Result<()> {
    data.total_likes += 1;

    Ok(())
}
