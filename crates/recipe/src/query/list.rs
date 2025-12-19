use crate::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged, Deleted,
    DietaryRestriction, DietaryRestrictionsChanged, Imported, Ingredient, IngredientsChanged,
    Instruction, InstructionsChanged, MadePrivate, MainCourseOptionsChanged, Recipe, RecipeType,
    RecipeTypeChanged, SharedToCommunity, SortBy,
};
use bincode::{Decode, Encode};
use evento::{
    AggregatorName, Executor, SubscribeBuilder,
    cursor::{Args, ReadResult},
    sql::Reader,
};
use imkitchen_db::table::{RecipeList, User};
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::prelude::FromRow;

#[derive(Debug, Encode, Decode)]
pub struct RecipeQueryCursor {
    pub i: String,
    pub v: u64,
}

#[derive(Default, FromRow)]
pub struct RecipeRow {
    pub id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub cuisine_type: sqlx::types::Text<CuisineType>,
    pub name: String,
    pub description: String,
    pub household_size: u16,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: imkitchen_db::types::Bincode<Vec<Ingredient>>,
    pub instructions: imkitchen_db::types::Bincode<Vec<Instruction>>,
    pub dietary_restrictions: sqlx::types::Json<Vec<DietaryRestriction>>,
    pub accepts_accompaniment: bool,
    pub advance_prep: String,
    pub is_shared: bool,
}

#[derive(Debug, Default, FromRow)]
pub struct RecipeListRow {
    pub id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub cuisine_type: sqlx::types::Text<CuisineType>,
    pub name: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub dietary_restrictions: sqlx::types::Json<Vec<DietaryRestriction>>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
    pub created_at: u64,
}

impl evento::cursor::Cursor for RecipeListRow {
    type T = RecipeQueryCursor;

    fn serialize(&self) -> Self::T {
        Self::T {
            i: self.id.to_owned(),
            v: self.created_at,
        }
    }
}

impl evento::sql::Bind for RecipeListRow {
    type T = (RecipeList, RecipeList);
    type I = [Self::T; 2];
    type V = [Expr; 2];
    type Cursor = Self;

    fn columns() -> Self::I {
        [
            (RecipeList::Table, RecipeList::CreatedAt),
            (RecipeList::Table, RecipeList::Id),
        ]
    }

    fn values(
        cursor: <<Self as evento::sql::Bind>::Cursor as evento::cursor::Cursor>::T,
    ) -> Self::V {
        [cursor.v.into(), cursor.i.into()]
    }
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

impl super::Query {
    pub async fn filter(&self, query: RecipesQuery) -> anyhow::Result<ReadResult<RecipeListRow>> {
        let mut statement = sea_query::Query::select()
            .columns([
                (RecipeList::Table, RecipeList::Id),
                (RecipeList::Table, RecipeList::UserId),
                (RecipeList::Table, RecipeList::RecipeType),
                (RecipeList::Table, RecipeList::CuisineType),
                (RecipeList::Table, RecipeList::Name),
                (RecipeList::Table, RecipeList::Description),
                (RecipeList::Table, RecipeList::PrepTime),
                (RecipeList::Table, RecipeList::CookTime),
                (RecipeList::Table, RecipeList::DietaryRestrictions),
                (RecipeList::Table, RecipeList::AcceptsAccompaniment),
                (RecipeList::Table, RecipeList::IsShared),
                (RecipeList::Table, RecipeList::CreatedAt),
            ])
            .column((User::Table, User::Username))
            .from(RecipeList::Table)
            .join(
                sea_query::JoinType::InnerJoin,
                User::Table,
                Expr::col((RecipeList::Table, RecipeList::UserId)).equals((User::Table, User::Id)),
            )
            .to_owned();

        if let Some(user_id) = query.user_id {
            statement.and_where(Expr::col(RecipeList::UserId).eq(user_id));
        }

        if let Some(is_shared) = query.is_shared {
            statement.and_where(Expr::col(RecipeList::IsShared).eq(is_shared));
        }

        if let Some(exclude_ids) = query.exclude_ids {
            statement
                .and_where(Expr::col((RecipeList::Table, RecipeList::Id)).is_not_in(exclude_ids));
        }

        if let Some(recipe_type) = query.recipe_type {
            statement.and_where(Expr::col(RecipeList::RecipeType).eq(recipe_type.to_string()));
        }

        if let Some(cuisine_type) = query.cuisine_type {
            statement.and_where(Expr::col(RecipeList::CuisineType).eq(cuisine_type.to_string()));
        }

        if !query.dietary_restrictions.is_empty() && !query.dietary_where_any {
            let in_clause = query
                .dietary_restrictions
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");

            statement.and_where(Expr::cust_with_values(
                format!("(SELECT COUNT(*) FROM json_each(dietary_restrictions) WHERE value IN ({})) = ?", in_clause),
            query.dietary_restrictions
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

        Ok(reader
            .args(query.args)
            .execute::<_, RecipeListRow, _>(&self.0)
            .await?)
    }

    pub async fn find(&self, id: impl Into<String>) -> anyhow::Result<Option<RecipeRow>> {
        let statement = sea_query::Query::select()
            .columns([
                RecipeList::UserId,
                RecipeList::RecipeType,
                RecipeList::CuisineType,
                RecipeList::Name,
                RecipeList::Description,
                RecipeList::HouseholdSize,
                RecipeList::PrepTime,
                RecipeList::CookTime,
                RecipeList::Ingredients,
                RecipeList::Instructions,
                RecipeList::DietaryRestrictions,
                RecipeList::AcceptsAccompaniment,
                RecipeList::AdvancePrep,
                RecipeList::IsShared,
                RecipeList::UpdatedAt,
            ])
            .columns([
                (RecipeList::Table, RecipeList::Id),
                (RecipeList::Table, RecipeList::CreatedAt),
            ])
            .column((User::Table, User::Username))
            .from(RecipeList::Table)
            .join(
                sea_query::JoinType::InnerJoin,
                User::Table,
                Expr::col((RecipeList::Table, RecipeList::UserId)).equals((User::Table, User::Id)),
            )
            .and_where(Expr::col((RecipeList::Table, RecipeList::Id)).eq(id.into()))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with::<_, RecipeRow, _>(&sql, values)
            .fetch_optional(&self.0)
            .await?)
    }
}

pub fn subscribe_list<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("recipe-list")
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
        .handler_check_off()
}

#[evento::handler(Recipe)]
async fn handle_created<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = event.aggregator_id.clone();
    let user_id = event.metadata.trigger_by()?;
    let name = event.data.name;
    let config = bincode::config::standard();
    let instructions = bincode::encode_to_vec(Vec::<Instruction>::default(), config)?;
    let ingredients = bincode::encode_to_vec(Vec::<Ingredient>::default(), config)?;

    let statement = Query::insert()
        .into_table(RecipeList::Table)
        .columns([
            RecipeList::Id,
            RecipeList::UserId,
            RecipeList::RecipeType,
            RecipeList::CuisineType,
            RecipeList::Name,
            RecipeList::Ingredients,
            RecipeList::Instructions,
            RecipeList::DietaryRestrictions,
            RecipeList::CreatedAt,
        ])
        .values_panic([
            aggregator_id.into(),
            user_id.into(),
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

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_imported<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = event.aggregator_id.clone();
    let user_id = event.metadata.trigger_by()?;
    let name = event.data.name;
    let config = bincode::config::standard();
    let instructions = bincode::encode_to_vec(event.data.instructions, config)?;
    let ingredients = bincode::encode_to_vec(event.data.ingredients, config)?;

    let statement = Query::insert()
        .into_table(RecipeList::Table)
        .columns([
            RecipeList::Id,
            RecipeList::UserId,
            RecipeList::Name,
            RecipeList::Description,
            RecipeList::RecipeType,
            RecipeList::CuisineType,
            RecipeList::HouseholdSize,
            RecipeList::PrepTime,
            RecipeList::CookTime,
            RecipeList::Ingredients,
            RecipeList::Instructions,
            RecipeList::AdvancePrep,
            RecipeList::DietaryRestrictions,
            RecipeList::CreatedAt,
        ])
        .values_panic([
            aggregator_id.into(),
            user_id.into(),
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

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_recipe_type_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<RecipeTypeChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (
                RecipeList::RecipeType,
                event.data.recipe_type.to_string().into(),
            ),
            (RecipeList::UpdatedAt, event.timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_basic_information_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<BasicInformationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = event.aggregator_id.clone();
    let name = event.data.name;
    let description = event.data.description;
    let household_size = event.data.household_size;
    let prep_time = event.data.prep_time;
    let cook_time = event.data.cook_time;

    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (RecipeList::Name, name.into()),
            (RecipeList::Description, description.into()),
            (RecipeList::HouseholdSize, household_size.into()),
            (RecipeList::PrepTime, prep_time.into()),
            (RecipeList::CookTime, cook_time.into()),
            (RecipeList::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_ingredients_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<IngredientsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let ingredients = bincode::encode_to_vec(&event.data.ingredients, config)?;
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (RecipeList::Ingredients, ingredients.into()),
            (RecipeList::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_instructions_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<InstructionsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let config = bincode::config::standard();
    let instructions = bincode::encode_to_vec(&event.data.instructions, config)?;
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (RecipeList::Instructions, instructions.into()),
            (RecipeList::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_dietary_restrictions_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<DietaryRestrictionsChanged>,
) -> anyhow::Result<()> {
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
        .table(RecipeList::Table)
        .values([
            (
                RecipeList::DietaryRestrictions,
                serde_json::Value::Array(dietary_restrictions).into(),
            ),
            (RecipeList::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_cuisine_type_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<CuisineTypeChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (
                RecipeList::CuisineType,
                event.data.cuisine_type.to_string().into(),
            ),
            (RecipeList::UpdatedAt, event.timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_main_course_options_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MainCourseOptionsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = &event.aggregator_id;
    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (
                RecipeList::AcceptsAccompaniment,
                event.data.accepts_accompaniment.into(),
            ),
            (RecipeList::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_advance_prep_changed<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<AdvancePrepChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let timestamp = event.timestamp;
    let aggregator_id = event.aggregator_id.clone();
    let description = event.data.advance_prep;

    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (RecipeList::AdvancePrep, description.into()),
            (RecipeList::UpdatedAt, timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_shared_to_community<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<SharedToCommunity>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (RecipeList::IsShared, event.data.shared.into()),
            (RecipeList::UpdatedAt, event.timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_made_private<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MadePrivate>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (RecipeList::IsShared, event.data.shared.into()),
            (RecipeList::UpdatedAt, event.timestamp.into()),
        ])
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(Recipe)]
async fn handle_deleted<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::delete()
        .from_table(RecipeList::Table)
        .and_where(Expr::col(RecipeList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
