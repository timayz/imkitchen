use crate::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged, Deleted,
    DietaryRestrictionsChanged, Imported, Ingredient, IngredientsChanged, Instruction,
    InstructionsChanged, MadePrivate, MainCourseOptionsChanged, Recipe, RecipeType,
    RecipeTypeChanged, SharedToCommunity,
};
use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::RecipeList;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

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
    let prep_time = event.data.prep_time;
    let cook_time = event.data.cook_time;

    let statement = Query::update()
        .table(RecipeList::Table)
        .values([
            (RecipeList::Name, name.into()),
            (RecipeList::Description, description.into()),
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
