use evento::{Action, Executor, Projection, SubscriptionBuilder, metadata::Event};
use imkitchen_db::table::RecipeCommand;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sha3::{Digest, Sha3_224};
use sqlx::{SqlitePool, prelude::FromRow};

use crate::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged, Deleted,
    DietaryRestrictionsChanged, Imported, IngredientsChanged, InstructionsChanged, MadePrivate,
    MainCourseOptionsChanged, Recipe, RecipeType, RecipeTypeChanged, SharedToCommunity,
};

mod create;
mod delete;
mod import;
mod make_private;
mod share_to_community;
mod update;

pub use import::ImportInput;
pub use update::UpdateInput;

#[evento::command]
#[derive(FromRow)]
pub struct Command {
    pub owner_id: String,
    pub recipe_type: sqlx::types::Text<RecipeType>,
    pub cuisine_type: sqlx::types::Text<CuisineType>,
    pub basic_information_hash: Vec<u8>,
    pub ingredients_hash: Vec<u8>,
    pub instructions_hash: Vec<u8>,
    pub dietary_restrictions_hash: Vec<u8>,
    pub advance_prep_hash: Vec<u8>,
    pub accepts_accompaniment: bool,
    pub is_shared: bool,
    pub is_deleted: bool,
}

pub fn create_projection<E: Executor>() -> Projection<CommandData, E> {
    Projection::new("recipe-command")
        .handler(handle_created())
        .handler(handle_deleted())
        .handler(handle_imported())
        .handler(handle_made_private())
        .handler(handle_ingredients_changed())
        .handler(handle_recipe_type_changed())
        .handler(handle_shared_to_community())
        .handler(handle_advance_prep_changed())
        .handler(handle_cuinine_type_changed())
        .handler(handle_instructions_changed())
        .handler(handle_basic_information_changed())
        .handler(handle_main_course_options_changed())
        .handler(handle_dietary_restrictions_changed())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<Command<'a, E>>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .no_safety_check()
        .load::<Recipe>(&id)
        .data(pool.clone())
        .execute_all(executor)
        .await?
        .map(|loaded| Command::new(id, loaded, executor)))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<CommandData, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::snapshot]
async fn restore(
    context: &evento::context::RwContext,
    id: String,
    _aggregators: &std::collections::HashMap<String, String>,
) -> anyhow::Result<Option<CommandData>> {
    let pool = context.extract::<SqlitePool>();
    let statement = Query::select()
        .columns([
            RecipeCommand::OwnerId,
            RecipeCommand::RecipeType,
            RecipeCommand::CuisineType,
            RecipeCommand::IsShared,
            RecipeCommand::BasicInformationHash,
            RecipeCommand::IngredientsHash,
            RecipeCommand::InstructionsHash,
            RecipeCommand::DietaryRestrictionsHash,
            RecipeCommand::AdvancePrepHash,
            RecipeCommand::AcceptsAccompaniment,
            RecipeCommand::IsDeleted,
        ])
        .from(RecipeCommand::Table)
        .and_where(Expr::col(RecipeCommand::Id).eq(id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(&pool)
        .await?)
}

#[evento::handler]
async fn handle_created<E: Executor>(
    event: Event<Created>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.owner_id = event.metadata.user()?;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            let statement = Query::insert()
                .into_table(RecipeCommand::Table)
                .columns([
                    RecipeCommand::Id,
                    RecipeCommand::OwnerId,
                    RecipeCommand::RecipeType,
                    RecipeCommand::CuisineType,
                ])
                .values([
                    event.aggregator_id.to_owned().into(),
                    event.metadata.user()?.into(),
                    RecipeType::default().to_string().into(),
                    CuisineType::default().to_string().into(),
                ])?
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
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    let id = event.aggregator_id.to_owned();
    let mut hasher = Sha3_224::default();
    hasher.update(event.data.name);
    hasher.update(event.data.description);
    hasher.update(event.data.household_size.to_string());
    hasher.update(event.data.prep_time.to_string());
    hasher.update(event.data.cook_time.to_string());

    let basic_information_hash = hasher.finalize()[..].to_vec();

    let mut hasher = Sha3_224::default();

    for instruction in event.data.instructions {
        hasher.update(instruction.description);
        hasher.update(instruction.time_next.to_string());
    }

    let instructions_hash = hasher.finalize()[..].to_vec();

    let mut hasher = Sha3_224::default();

    for ingredient in event.data.ingredients {
        hasher.update(ingredient.name);
        hasher.update(ingredient.quantity.to_string());

        if let Some(unit) = ingredient.unit {
            hasher.update(unit.to_string());
        }

        if let Some(category) = ingredient.category {
            hasher.update(category.to_string());
        }
    }

    let ingredients_hash = hasher.finalize()[..].to_vec();

    let mut hasher = Sha3_224::default();
    hasher.update(event.data.advance_prep);

    let advance_prep_hash = hasher.finalize()[..].to_vec();

    match action {
        Action::Apply(data) => {
            data.owner_id = event.metadata.user()?;
            data.recipe_type.0 = event.data.recipe_type;
            data.cuisine_type.0 = event.data.cuisine_type;
            data.basic_information_hash = basic_information_hash;
            data.instructions_hash = instructions_hash;
            data.ingredients_hash = ingredients_hash;
            data.advance_prep_hash = advance_prep_hash;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            let statement = Query::insert()
                .into_table(RecipeCommand::Table)
                .columns([
                    RecipeCommand::Id,
                    RecipeCommand::OwnerId,
                    RecipeCommand::RecipeType,
                    RecipeCommand::CuisineType,
                    RecipeCommand::BasicInformationHash,
                    RecipeCommand::IngredientsHash,
                    RecipeCommand::InstructionsHash,
                    RecipeCommand::AdvancePrepHash,
                ])
                .values([
                    id.into(),
                    event.metadata.user()?.into(),
                    event.data.recipe_type.to_string().into(),
                    event.data.cuisine_type.to_string().into(),
                    basic_information_hash.into(),
                    ingredients_hash.into(),
                    instructions_hash.into(),
                    advance_prep_hash.into(),
                ])?
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
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.recipe_type.0 = event.data.recipe_type;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                &event.aggregator_id,
                RecipeCommand::RecipeType,
                event.data.recipe_type.to_string(),
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_basic_information_changed<E: Executor>(
    event: Event<BasicInformationChanged>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    let id = event.aggregator_id.to_owned();
    let mut hasher = Sha3_224::default();
    hasher.update(event.data.name);
    hasher.update(event.data.description);
    hasher.update(event.data.household_size.to_string());
    hasher.update(event.data.prep_time.to_string());
    hasher.update(event.data.cook_time.to_string());

    let basic_information_hash = hasher.finalize()[..].to_vec();

    match action {
        Action::Apply(data) => {
            data.basic_information_hash = basic_information_hash;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                id,
                RecipeCommand::BasicInformationHash,
                basic_information_hash,
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_instructions_changed<E: Executor>(
    event: Event<InstructionsChanged>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    let id = event.aggregator_id.to_owned();
    let mut hasher = Sha3_224::default();

    for instruction in event.data.instructions {
        hasher.update(instruction.description);
        hasher.update(instruction.time_next.to_string());
    }

    let instructions_hash = hasher.finalize()[..].to_vec();

    match action {
        Action::Apply(data) => {
            data.instructions_hash = instructions_hash;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                id,
                RecipeCommand::InstructionsHash,
                instructions_hash,
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_ingredients_changed<E: Executor>(
    event: Event<IngredientsChanged>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    let id = event.aggregator_id.to_owned();
    let mut hasher = Sha3_224::default();

    for ingredient in event.data.ingredients {
        hasher.update(ingredient.name);
        hasher.update(ingredient.quantity.to_string());

        if let Some(unit) = ingredient.unit {
            hasher.update(unit.to_string());
        }

        if let Some(category) = ingredient.category {
            hasher.update(category.to_string());
        }
    }

    let ingredients_hash = hasher.finalize()[..].to_vec();

    match action {
        Action::Apply(data) => {
            data.ingredients_hash = ingredients_hash;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(&pool, id, RecipeCommand::IngredientsHash, ingredients_hash).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_dietary_restrictions_changed<E: Executor>(
    event: Event<DietaryRestrictionsChanged>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    let id = event.aggregator_id.to_owned();
    let mut hasher = Sha3_224::default();

    for restriction in event.data.dietary_restrictions {
        hasher.update(restriction.to_string());
    }

    let dietary_restrictions_hash = hasher.finalize()[..].to_vec();

    match action {
        Action::Apply(data) => {
            data.dietary_restrictions_hash = dietary_restrictions_hash;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                id,
                RecipeCommand::DietaryRestrictionsHash,
                dietary_restrictions_hash,
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_cuinine_type_changed<E: Executor>(
    event: Event<CuisineTypeChanged>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.cuisine_type.0 = event.data.cuisine_type;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                &event.aggregator_id,
                RecipeCommand::CuisineType,
                event.data.cuisine_type.to_string(),
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_main_course_options_changed<E: Executor>(
    event: Event<MainCourseOptionsChanged>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.accepts_accompaniment = event.data.accepts_accompaniment;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                &event.aggregator_id,
                RecipeCommand::AcceptsAccompaniment,
                event.data.accepts_accompaniment,
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_advance_prep_changed<E: Executor>(
    event: Event<AdvancePrepChanged>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    let id = event.aggregator_id.to_owned();
    let mut hasher = Sha3_224::default();
    hasher.update(event.data.advance_prep);

    let advance_prep_hash = hasher.finalize()[..].to_vec();

    match action {
        Action::Apply(data) => {
            data.advance_prep_hash = advance_prep_hash;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(&pool, id, RecipeCommand::AdvancePrepHash, advance_prep_hash).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_shared_to_community<E: Executor>(
    event: Event<SharedToCommunity>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.is_shared = true;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(&pool, &event.aggregator_id, RecipeCommand::IsShared, true).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_made_private<E: Executor>(
    event: Event<MadePrivate>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.is_shared = false;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(&pool, &event.aggregator_id, RecipeCommand::IsShared, false).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_deleted<E: Executor>(
    event: Event<Deleted>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.is_deleted = true;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(&pool, &event.aggregator_id, RecipeCommand::IsDeleted, true).await?;
        }
    };

    Ok(())
}

async fn update(
    pool: &SqlitePool,
    id: impl Into<Expr>,
    col: RecipeCommand,
    value: impl Into<Expr>,
) -> anyhow::Result<()> {
    let statement = Query::update()
        .table(RecipeCommand::Table)
        .value(col, value)
        .and_where(Expr::col(RecipeCommand::Id).eq(id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}
