use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::RecipeUserFts;
use imkitchen_shared::recipe::{
    BasicInformationChanged, Created, Deleted, Imported, IngredientsChanged,
};
use sea_query::{Expr, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-user-fts-query")
        .handler(handle_created())
        .handler(handle_imported())
        .handler(handle_ingredients_changed())
        .handler(handle_basic_information_changed())
        .handler(handle_deleted())
}

#[evento::subscription]
async fn handle_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let (sql, values) = Query::insert()
        .into_table(RecipeUserFts::Table)
        .columns([
            RecipeUserFts::Id,
            RecipeUserFts::Name,
            RecipeUserFts::Description,
            RecipeUserFts::Ingredients,
        ])
        .values([
            event.aggregator_id.to_owned().into(),
            event.data.name.into(),
            "".into(),
            "".into(),
        ])?
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::subscription]
async fn handle_imported<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Imported>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let ingredients = event
        .data
        .ingredients
        .iter()
        .map(|i| i.name.to_owned())
        .collect::<Vec<_>>()
        .join(" ");

    let (sql, values) = Query::insert()
        .into_table(RecipeUserFts::Table)
        .columns([
            RecipeUserFts::Id,
            RecipeUserFts::Name,
            RecipeUserFts::Description,
            RecipeUserFts::Ingredients,
        ])
        .values([
            event.aggregator_id.to_owned().into(),
            event.data.name.into(),
            event.data.description.into(),
            ingredients.into(),
        ])?
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::subscription]
async fn handle_basic_information_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<BasicInformationChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let (sql, values) = Query::update()
        .from(RecipeUserFts::Table)
        .and_where(Expr::cust_with_values(
            "recipe_user_fts = ?",
            [event.aggregator_id.to_owned()],
        ))
        .values([
            (RecipeUserFts::Name, event.data.name.into()),
            (RecipeUserFts::Description, event.data.description.into()),
        ])
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::subscription]
async fn handle_ingredients_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<IngredientsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let ingredients = event
        .data
        .ingredients
        .iter()
        .map(|i| i.name.to_owned())
        .collect::<Vec<_>>()
        .join(" ");

    let (sql, values) = Query::update()
        .from(RecipeUserFts::Table)
        .and_where(Expr::cust_with_values(
            "recipe_user_fts = ?",
            [event.aggregator_id.to_owned()],
        ))
        .value(RecipeUserFts::Name, ingredients)
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;
    Ok(())
}

#[evento::subscription]
async fn handle_deleted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Deleted>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let (sql, values) = Query::delete()
        .from_table(RecipeUserFts::Table)
        .and_where(Expr::cust_with_values(
            "recipe_user_fts = ?",
            [event.aggregator_id.to_owned()],
        ))
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
