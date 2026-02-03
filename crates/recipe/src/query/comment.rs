use evento::{
    Cursor, Executor, Projection, Snapshot,
    cursor::{Args, ReadResult},
    metadata::Event,
    sql::Reader,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::RecipeComment;
use imkitchen_shared::recipe::{
    comment::{self, Added, Replied},
    comment_rating::{LikeChecked, LikeUnchecked, UnlikeChecked, UnlikeUnchecked},
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

#[evento::projection(FromRow, Cursor)]
pub struct CommentView {
    #[cursor(RecipeComment::Id, 1)]
    pub id: String,
    pub recipe_id: String,
    pub owner_id: String,
    pub owner_name: String,
    pub reply_to: Option<String>,
    pub body: String,
    pub total_likes: i64,
    pub total_replies: u64,
    #[cursor(RecipeComment::CreatedAt, 2)]
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

impl CommentView {
    pub fn total_ulikes(&self) -> u64 {
        self.total_likes.try_into().unwrap_or(0)
    }
}

pub struct CommentsQuery {
    pub recipe_id: String,
    pub reply_to: Option<String>,
    pub exclude_owner: Option<String>,
    pub args: Args,
}

impl<E: Executor> super::Query<E> {
    pub async fn filter_comment(
        &self,
        query: CommentsQuery,
    ) -> anyhow::Result<ReadResult<CommentView>> {
        let mut statement = sea_query::Query::select()
            .columns([
                RecipeComment::Id,
                RecipeComment::Cursor,
                RecipeComment::RecipeId,
                RecipeComment::OwnerId,
                RecipeComment::ReplyTo,
                RecipeComment::OwnerName,
                RecipeComment::Body,
                RecipeComment::TotalLikes,
                RecipeComment::TotalReplies,
                RecipeComment::CreatedAt,
                RecipeComment::UpdatedAt,
            ])
            .from(RecipeComment::Table)
            .and_where(Expr::col(RecipeComment::RecipeId).eq(query.recipe_id))
            .to_owned();

        if let Some(reply_to) = query.reply_to {
            statement.and_where(Expr::col(RecipeComment::ReplyTo).eq(reply_to));
        }

        if let Some(owner_id) = query.exclude_owner {
            statement.and_where(Expr::col(RecipeComment::OwnerId).not_equals(owner_id));
        }

        Reader::new(statement)
            .args(query.args)
            .desc()
            .execute(&self.read_db)
            .await
    }
}

async fn find_comment(
    pool: &SqlitePool,
    id: impl Into<String>,
) -> anyhow::Result<Option<CommentView>> {
    let statement = sea_query::Query::select()
        .columns([
            RecipeComment::Id,
            RecipeComment::Cursor,
            RecipeComment::RecipeId,
            RecipeComment::ReplyTo,
            RecipeComment::OwnerId,
            RecipeComment::OwnerName,
            RecipeComment::Body,
            RecipeComment::TotalLikes,
            RecipeComment::TotalReplies,
            RecipeComment::CreatedAt,
            RecipeComment::UpdatedAt,
        ])
        .from(RecipeComment::Table)
        .and_where(Expr::col(RecipeComment::Id).eq(id.into()))
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(pool)
        .await?)
}

pub fn create_projection<E: Executor>(ids: Vec<impl Into<String>>) -> Projection<E, CommentView> {
    Projection::ids::<comment::Comment>(ids).handler(handle_added())
}

impl<E: Executor> super::Query<E> {
    pub async fn comment(
        &self,
        recipe_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Result<Option<CommentView>, anyhow::Error> {
        load(
            &self.executor,
            &self.read_db,
            &self.write_db,
            recipe_id,
            user_id,
        )
        .await
    }
}

pub(crate) async fn load<E: Executor>(
    executor: &E,
    read_db: &SqlitePool,
    write_db: &SqlitePool,
    recipe_id: impl Into<String>,
    user_id: impl Into<String>,
) -> Result<Option<CommentView>, anyhow::Error> {
    // let id = id.into();
    //
    // if executor.has_event::<Deleted>(&id).await? {
    //     return Ok(None);
    // }

    create_projection(vec![recipe_id.into(), user_id.into()])
        .data((read_db.clone(), write_db.clone()))
        .execute(executor)
        .await
}

impl<E: Executor> Snapshot<E> for CommentView {
    async fn restore(context: &evento::projection::Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let (read_db, _) = context.extract::<(SqlitePool, SqlitePool)>();
        find_comment(&read_db, &context.id).await
    }

    async fn take_snapshot(
        &self,
        context: &evento::projection::Context<'_, E>,
    ) -> anyhow::Result<()> {
        let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();

        let statement = sea_query::Query::insert()
            .into_table(RecipeComment::Table)
            .columns([
                RecipeComment::Id,
                RecipeComment::RecipeId,
                RecipeComment::ReplyTo,
                RecipeComment::Cursor,
                RecipeComment::OwnerId,
                RecipeComment::OwnerName,
                RecipeComment::Body,
                RecipeComment::TotalLikes,
                RecipeComment::TotalReplies,
                RecipeComment::CreatedAt,
                RecipeComment::UpdatedAt,
            ])
            .values([
                self.id.to_owned().into(),
                self.recipe_id.to_owned().into(),
                self.reply_to.to_owned().into(),
                self.cursor.to_owned().into(),
                self.owner_id.to_owned().into(),
                self.owner_name.to_owned().into(),
                self.body.to_owned().into(),
                self.total_likes.into(),
                self.total_replies.into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])?
            .on_conflict(
                OnConflict::column(RecipeComment::Id)
                    .update_columns([
                        RecipeComment::RecipeId,
                        RecipeComment::ReplyTo,
                        RecipeComment::Cursor,
                        RecipeComment::OwnerId,
                        RecipeComment::OwnerName,
                        RecipeComment::Body,
                        RecipeComment::TotalLikes,
                        RecipeComment::TotalReplies,
                        RecipeComment::CreatedAt,
                        RecipeComment::UpdatedAt,
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
async fn handle_added(event: Event<Added>, data: &mut CommentView) -> anyhow::Result<()> {
    data.owner_id = event.metadata.requested_by()?;
    data.owner_name = event.data.owner_name.to_owned();
    data.recipe_id = event.data.recipe_id.to_owned();
    data.created_at = event.timestamp;
    data.id = event.aggregator_id.to_owned();
    data.body = event.data.body;

    Ok(())
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("recipe-comment-query")
        .handler(handle_replied())
        .handler(handle_like_checked())
        .handler(handle_like_unchecked())
        .handler(handle_unlike_checked())
        .handler(handle_unlike_unchecked())
}

#[evento::subscription]
async fn handle_replied<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Replied>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = sea_query::Query::insert()
        .into_table(RecipeComment::Table)
        .columns([
            RecipeComment::Id,
            RecipeComment::RecipeId,
            RecipeComment::ReplyTo,
            RecipeComment::Cursor,
            RecipeComment::OwnerId,
            RecipeComment::OwnerName,
            RecipeComment::Body,
            RecipeComment::CreatedAt,
        ])
        .values([
            event.id.to_string().into(),
            event.data.recipe_id.to_owned().into(),
            event.aggregator_id.to_owned().into(),
            "".into(),
            event.metadata.requested_by()?.into(),
            event.data.owner_name.to_owned().into(),
            event.data.body.to_owned().into(),
            event.timestamp.into(),
        ])?
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    update(
        &pool,
        RecipeComment::TotalReplies,
        true,
        event.aggregator_id.to_owned(),
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_like_checked<E: Executor>(
    context: &Context<'_, E>,
    event: Event<LikeChecked>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update(
        &pool,
        RecipeComment::TotalLikes,
        true,
        event.data.comment_id,
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_like_unchecked<E: Executor>(
    context: &Context<'_, E>,
    event: Event<LikeUnchecked>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update(
        &pool,
        RecipeComment::TotalLikes,
        false,
        event.data.comment_id,
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_unlike_checked<E: Executor>(
    context: &Context<'_, E>,
    event: Event<UnlikeChecked>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update(
        &pool,
        RecipeComment::TotalLikes,
        false,
        event.data.comment_id,
    )
    .await?;

    Ok(())
}

#[evento::subscription]
async fn handle_unlike_unchecked<E: Executor>(
    context: &Context<'_, E>,
    event: Event<UnlikeUnchecked>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    update(
        &pool,
        RecipeComment::TotalLikes,
        true,
        event.data.comment_id,
    )
    .await?;

    Ok(())
}

async fn update(
    pool: &SqlitePool,
    col: RecipeComment,
    add: bool,
    id: String,
) -> anyhow::Result<()> {
    let expr = if add {
        Expr::col(col.clone()).add(1)
    } else {
        Expr::col(col.clone()).sub(1)
    };
    let statement = sea_query::Query::update()
        .table(RecipeComment::Table)
        .value(col, expr)
        .and_where(Expr::col(RecipeComment::Id).eq(id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}
