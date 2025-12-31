use crate::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged, Deleted,
    DietaryRestriction, DietaryRestrictionsChanged, Imported, Ingredient, IngredientsChanged,
    Instruction, InstructionsChanged, MadePrivate, MainCourseOptionsChanged, Recipe, RecipeType,
    RecipeTypeChanged, SharedToCommunity, SortBy,
    rating::{LikeChecked, LikeUnchecked, UnlikeChecked, UnlikeUnchecked, Viewed},
};
use evento::{
    Action, Cursor, Executor, LoadResult, Projection, SubscriptionBuilder,
    cursor::{Args, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::table::RecipeUser;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};

#[derive(Default, FromRow)]
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

pub async fn filter(
    pool: &SqlitePool,
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

    reader.args(query.args).execute(pool).await
}

pub async fn find(pool: &SqlitePool, id: impl Into<String>) -> anyhow::Result<Option<UserView>> {
    let statement = sea_query::Query::select()
        .columns([
            RecipeUser::Id,
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
            RecipeUser::UpdatedAt,
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

pub fn create_projection<E: Executor>() -> Projection<UserView, E> {
    Projection::new("recipe-user-view")
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

pub async fn load<E: Executor>(
    executor: &E,
    pool: &SqlitePool,
    id: impl Into<String>,
) -> Result<Option<LoadResult<UserView>>, anyhow::Error> {
    let id = id.into();

    create_projection()
        .no_safety_check()
        .load::<Recipe>(&id)
        .data(pool.clone())
        .execute(executor)
        .await
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<UserView, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::snapshot]
async fn restore(
    context: &evento::context::RwContext,
    id: String,
    _aggregators: &std::collections::HashMap<String, String>,
) -> anyhow::Result<Option<UserView>> {
    let pool = context.extract::<SqlitePool>();

    Ok(Some(find(&pool, id).await?.unwrap_or_default()))
}

#[evento::handler]
async fn handle_created<E: Executor>(
    event: Event<Created>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.owner_id = event.metadata.user()?;
            data.owner_name = event.data.owner_name.to_owned();
            data.created_at = event.timestamp;
            data.id = event.aggregator_id.to_owned();
            data.name = event.data.name;
            data.household_size = 4;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let timestamp = event.timestamp;
            let aggregator_id = event.aggregator_id.clone();
            let user_id = event.metadata.user()?;
            let name = event.data.name;
            let instructions = bitcode::encode(&Vec::<Instruction>::default());
            let ingredients = bitcode::encode(&Vec::<Ingredient>::default());

            let statement = Query::insert()
                .into_table(RecipeUser::Table)
                .columns([
                    RecipeUser::Id,
                    RecipeUser::OwnerId,
                    RecipeUser::OwnerName,
                    RecipeUser::RecipeType,
                    RecipeUser::CuisineType,
                    RecipeUser::Name,
                    RecipeUser::Ingredients,
                    RecipeUser::Instructions,
                    RecipeUser::DietaryRestrictions,
                    RecipeUser::CreatedAt,
                ])
                .values_panic([
                    aggregator_id.into(),
                    user_id.into(),
                    event.data.owner_name.to_owned().into(),
                    RecipeType::default().to_string().into(),
                    CuisineType::default().to_string().into(),
                    name.into(),
                    ingredients.into(),
                    instructions.into(),
                    serde_json::Value::Array(vec![]).into(),
                    timestamp.into(),
                ])
                .to_owned();
            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_imported<E: Executor>(
    event: Event<Imported>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    let instructions = bitcode::encode(&event.data.instructions);
    let ingredients = bitcode::encode(&event.data.ingredients);

    match action {
        Action::Apply(data) => {
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
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let timestamp = event.timestamp;
            let aggregator_id = event.aggregator_id.clone();
            let user_id = event.metadata.user()?;
            let name = event.data.name;

            let statement = Query::insert()
                .into_table(RecipeUser::Table)
                .columns([
                    RecipeUser::Id,
                    RecipeUser::OwnerId,
                    RecipeUser::OwnerName,
                    RecipeUser::Name,
                    RecipeUser::Description,
                    RecipeUser::RecipeType,
                    RecipeUser::CuisineType,
                    RecipeUser::HouseholdSize,
                    RecipeUser::PrepTime,
                    RecipeUser::CookTime,
                    RecipeUser::Ingredients,
                    RecipeUser::Instructions,
                    RecipeUser::AdvancePrep,
                    RecipeUser::DietaryRestrictions,
                    RecipeUser::CreatedAt,
                ])
                .values_panic([
                    aggregator_id.into(),
                    user_id.into(),
                    event.data.owner_name.to_owned().into(),
                    name.into(),
                    event.data.description.into(),
                    event.data.recipe_type.to_string().into(),
                    event.data.cuisine_type.to_string().into(),
                    event.data.household_size.into(),
                    event.data.prep_time.into(),
                    event.data.cook_time.into(),
                    ingredients.into(),
                    instructions.into(),
                    event.data.advance_prep.into(),
                    serde_json::Value::Array(vec![]).into(),
                    timestamp.into(),
                ])
                .to_owned();
            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_recipe_type_changed<E: Executor>(
    event: Event<RecipeTypeChanged>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.recipe_type.0 = event.data.recipe_type;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (
                        RecipeUser::RecipeType,
                        event.data.recipe_type.to_string().into(),
                    ),
                    (RecipeUser::UpdatedAt, event.timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_basic_information_changed<E: Executor>(
    event: Event<BasicInformationChanged>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.name = event.data.name;
            data.description = event.data.description;
            data.household_size = event.data.household_size;
            data.prep_time = event.data.prep_time;
            data.cook_time = event.data.cook_time;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let timestamp = event.timestamp;
            let aggregator_id = event.aggregator_id.clone();
            let name = event.data.name;
            let description = event.data.description;
            let household_size = event.data.household_size;
            let prep_time = event.data.prep_time;
            let cook_time = event.data.cook_time;

            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (RecipeUser::Name, name.into()),
                    (RecipeUser::Description, description.into()),
                    (RecipeUser::HouseholdSize, household_size.into()),
                    (RecipeUser::PrepTime, prep_time.into()),
                    (RecipeUser::CookTime, cook_time.into()),
                    (RecipeUser::UpdatedAt, timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_ingredients_changed<E: Executor>(
    event: Event<IngredientsChanged>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.ingredients.0 = event.data.ingredients;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let ingredients = bitcode::encode(&event.data.ingredients);
            let timestamp = event.timestamp;
            let aggregator_id = &event.aggregator_id;
            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (RecipeUser::Ingredients, ingredients.into()),
                    (RecipeUser::UpdatedAt, timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_instructions_changed<E: Executor>(
    event: Event<InstructionsChanged>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.instructions.0 = event.data.instructions;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let instructions = bitcode::encode(&event.data.instructions);
            let timestamp = event.timestamp;
            let aggregator_id = &event.aggregator_id;
            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (RecipeUser::Instructions, instructions.into()),
                    (RecipeUser::UpdatedAt, timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_dietary_restrictions_changed<E: Executor>(
    event: Event<DietaryRestrictionsChanged>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.dietary_restrictions.0 = event.data.dietary_restrictions;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let dietary_restrictions = event
                .data
                .dietary_restrictions
                .iter()
                .map(|d| serde_json::Value::String(d.to_string()))
                .collect::<Vec<_>>();
            let timestamp = event.timestamp;
            let aggregator_id = &event.aggregator_id;
            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (
                        RecipeUser::DietaryRestrictions,
                        serde_json::Value::Array(dietary_restrictions).into(),
                    ),
                    (RecipeUser::UpdatedAt, timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_cuisine_type_changed<E: Executor>(
    event: Event<CuisineTypeChanged>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.cuisine_type.0 = event.data.cuisine_type;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (
                        RecipeUser::CuisineType,
                        event.data.cuisine_type.to_string().into(),
                    ),
                    (RecipeUser::UpdatedAt, event.timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_main_course_options_changed<E: Executor>(
    event: Event<MainCourseOptionsChanged>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.accepts_accompaniment = event.data.accepts_accompaniment;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let timestamp = event.timestamp;
            let aggregator_id = &event.aggregator_id;
            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (
                        RecipeUser::AcceptsAccompaniment,
                        event.data.accepts_accompaniment.into(),
                    ),
                    (RecipeUser::UpdatedAt, timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_advance_prep_changed<E: Executor>(
    event: Event<AdvancePrepChanged>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.advance_prep = event.data.advance_prep;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let timestamp = event.timestamp;
            let aggregator_id = event.aggregator_id.clone();
            let description = event.data.advance_prep;

            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (RecipeUser::AdvancePrep, description.into()),
                    (RecipeUser::UpdatedAt, timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_shared_to_community<E: Executor>(
    event: Event<SharedToCommunity>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.is_shared = true;
            data.owner_name = Some(event.data.owner_name);
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (RecipeUser::IsShared, true.into()),
                    (RecipeUser::UpdatedAt, event.timestamp.into()),
                    (
                        RecipeUser::OwnerName,
                        event.data.owner_name.to_owned().into(),
                    ),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };
    Ok(())
}

#[evento::handler]
async fn handle_made_private<E: Executor>(
    event: Event<MadePrivate>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.is_shared = false;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let statement = Query::update()
                .table(RecipeUser::Table)
                .values([
                    (RecipeUser::IsShared, false.into()),
                    (RecipeUser::UpdatedAt, event.timestamp.into()),
                ])
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };
    Ok(())
}

#[evento::handler]
async fn handle_deleted<E: Executor>(
    event: Event<Deleted>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.created_at = 0;
        }
        Action::Handle(context) => {
            let pool = context.extract::<sqlx::SqlitePool>();
            let statement = Query::delete()
                .from_table(RecipeUser::Table)
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };
    Ok(())
}

#[evento::handler]
async fn handle_viewed<E: Executor>(
    event: Event<Viewed>,
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.total_views += 1;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();
            let statement = Query::update()
                .table(RecipeUser::Table)
                .value(
                    RecipeUser::TotalViews,
                    Expr::col(RecipeUser::TotalViews).add(1),
                )
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
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
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.total_likes += 1;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();

            let statement = Query::update()
                .table(RecipeUser::Table)
                .value(
                    RecipeUser::TotalLikes,
                    Expr::col(RecipeUser::TotalLikes).add(1),
                )
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
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
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.total_likes -= 1;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();
            let statement = Query::update()
                .table(RecipeUser::Table)
                .value(
                    RecipeUser::TotalLikes,
                    Expr::col(RecipeUser::TotalLikes).sub(1),
                )
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
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
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.total_likes -= 1;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();

            let statement = Query::update()
                .table(RecipeUser::Table)
                .value(
                    RecipeUser::TotalLikes,
                    Expr::col(RecipeUser::TotalLikes).sub(1),
                )
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
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
    action: Action<'_, UserView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.total_likes += 1;
        }
        Action::Handle(context) => {
            let pool: SqlitePool = context.extract();

            let statement = Query::update()
                .table(RecipeUser::Table)
                .value(
                    RecipeUser::TotalLikes,
                    Expr::col(RecipeUser::TotalLikes).add(1),
                )
                .and_where(Expr::col(RecipeUser::Id).eq(&event.aggregator_id))
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}
