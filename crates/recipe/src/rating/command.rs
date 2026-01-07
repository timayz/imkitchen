use evento::{
    Executor, Projection, Snapshot,
    metadata::{Event, Metadata},
};
use sqlx::{SqlitePool, prelude::FromRow};
use ulid::Ulid;
use validator::Validate;

use crate::{
    Deleted,
    rating::{
        CommentAdded, CommentLikeCheked, CommentLikeUnchecked, CommentUnlikeChecked,
        CommentUnlikeUnchecked, LikeChecked, LikeUnchecked, Rating, UnlikeChecked, UnlikeUnchecked,
        Viewed,
    },
};

#[evento::command]
#[derive(FromRow)]
pub struct Command {
    pub user_id: String,
    pub viewed: bool,
    pub liked: bool,
    pub unliked: bool,
}

impl<'a, E: Executor + Clone> Command<'a, E> {
    pub async fn check_like(&self) -> imkitchen_shared::Result<()> {
        if !self.liked {
            self.aggregator()
                .event(&LikeChecked)
                .metadata(&Metadata::new(&self.user_id))
                .commit(self.executor)
                .await?;
        }

        Ok(())
    }

    pub async fn uncheck_like(&self) -> imkitchen_shared::Result<()> {
        if self.liked {
            self.aggregator()
                .event(&LikeUnchecked)
                .metadata(&Metadata::new(&self.user_id))
                .commit(self.executor)
                .await?;
        }

        Ok(())
    }

    pub async fn check_unlike(&self) -> imkitchen_shared::Result<()> {
        if !self.unliked {
            self.aggregator()
                .event(&UnlikeChecked)
                .metadata(&Metadata::new(&self.user_id))
                .commit(self.executor)
                .await?;
        }

        Ok(())
    }

    pub async fn uncheck_unlike(&self) -> imkitchen_shared::Result<()> {
        if self.unliked {
            self.aggregator()
                .event(&UnlikeUnchecked)
                .metadata(&Metadata::new(&self.user_id))
                .commit(self.executor)
                .await?;
        }

        Ok(())
    }

    pub async fn view(&self) -> imkitchen_shared::Result<()> {
        if !self.viewed {
            self.aggregator()
                .event(&Viewed)
                .metadata(&Metadata::new(&self.user_id))
                .commit(self.executor)
                .await?;
        }

        Ok(())
    }
}

#[derive(Validate)]
pub struct AddCommentInput {
    pub message: String,
    pub reply_to: Option<String>,
}

impl<'a, E: Executor + Clone> Command<'a, E> {
    pub async fn add_comment(&self, input: AddCommentInput) -> imkitchen_shared::Result<()> {
        //@TODO: check spam
        self.aggregator()
            .event(&CommentAdded {
                id: Ulid::new().to_string(),
                message: input.message,
                reply_to: input.reply_to,
            })
            .metadata(&Metadata::new(&self.user_id))
            .commit(self.executor)
            .await?;

        Ok(())
    }

    pub async fn check_comment_like(
        &self,
        comment_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let comment_id = comment_id.into();

        // @TODO: check comment exist
        // @TODO: skip if already liked

        self.aggregator()
            .event(&CommentLikeCheked { comment_id })
            .metadata(&Metadata::new(&self.user_id))
            .commit(self.executor)
            .await?;

        Ok(())
    }

    pub async fn uncheck_comment_like(
        &self,
        comment_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let comment_id = comment_id.into();

        // @TODO: check comment exist
        // @TODO: skip is already done

        self.aggregator()
            .event(&CommentLikeUnchecked { comment_id })
            .metadata(&Metadata::new(&self.user_id))
            .commit(self.executor)
            .await?;

        Ok(())
    }

    pub async fn check_comment_unlike(
        &self,
        comment_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let comment_id = comment_id.into();

        // @TODO: check comment exist
        // @TODO: skip if already done

        self.aggregator()
            .event(&CommentUnlikeChecked { comment_id })
            .metadata(&Metadata::new(&self.user_id))
            .commit(self.executor)
            .await?;

        Ok(())
    }

    pub async fn uncheck_comment_unlike(
        &self,
        comment_id: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let comment_id = comment_id.into();

        // @TODO: check comment exist
        // @TODO: skip if already done

        self.aggregator()
            .event(&CommentUnlikeUnchecked { comment_id })
            .metadata(&Metadata::new(&self.user_id))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}

pub fn create_projection(
    id: impl Into<String>,
    user_id: impl Into<String>,
) -> Projection<CommandData> {
    Projection::new::<Rating>(id)
        .aggregator_raw("imkitchen-user/User", user_id)
        .handler(handle_viewed())
        .handler(handle_like_checked())
        .handler(handle_like_unchecked())
        .handler(handle_unlike_checked())
        .handler(handle_unlike_unchecked())
        .handler(handle_recipe_deleted())
        .safety_check()
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
    user_id: impl Into<String>,
) -> Result<Command<'a, E>, anyhow::Error> {
    let id = id.into();
    let user_id = user_id.into();

    let result = create_projection(&id, user_id)
        .data(pool.clone())
        .execute(executor)
        .await?;

    let cmd = match result {
        Some(data) => Command::new(id, data.get_cursor_version()?, data, executor),
        _ => Command::new(id, 0, Default::default(), executor),
    };

    Ok(cmd)
}

impl Snapshot for CommandData {}

// #[evento::snapshot]
// async fn restore(
//     context: &evento::context::RwContext,
//     id: String,
//     aggregators: &std::collections::HashMap<String, String>,
// ) -> anyhow::Result<Option<CommandData>> {
//     let user_id = aggregators.get("imkitchen-user/User").unwrap();
//     let pool = context.extract::<SqlitePool>();
//     let statement = Query::select()
//         .columns([
//             RecipeRatingCommand::UserId,
//             RecipeRatingCommand::Viewed,
//             RecipeRatingCommand::Liked,
//             RecipeRatingCommand::Unliked,
//         ])
//         .from(RecipeRatingCommand::Table)
//         .and_where(Expr::col(RecipeRatingCommand::RecipeId).eq(id))
//         .and_where(Expr::col(RecipeRatingCommand::UserId).eq(user_id))
//         .to_owned();
//
//     let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
//
//     Ok(sqlx::query_as_with(&sql, values)
//         .fetch_optional(&pool)
//         .await?)
// }

#[evento::handler]
async fn handle_viewed(event: Event<Viewed>, data: &mut CommandData) -> anyhow::Result<()> {
    data.user_id = event.metadata.user()?;
    data.viewed = true;

    Ok(())
}

#[evento::handler]
async fn handle_like_checked(
    event: Event<LikeChecked>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.user_id = event.metadata.user()?;
    data.liked = true;
    data.unliked = false;

    Ok(())
}

#[evento::handler]
async fn handle_like_unchecked(
    event: Event<LikeUnchecked>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.user_id = event.metadata.user()?;
    data.liked = false;

    Ok(())
}

#[evento::handler]
async fn handle_unlike_checked(
    event: Event<UnlikeChecked>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.user_id = event.metadata.user()?;
    data.unliked = true;
    data.liked = false;

    Ok(())
}

#[evento::handler]
async fn handle_unlike_unchecked(
    event: Event<UnlikeUnchecked>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.user_id = event.metadata.user()?;
    data.unliked = false;

    Ok(())
}

#[evento::handler]
async fn handle_recipe_deleted(
    event: Event<Deleted>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.unliked = false;
    data.liked = false;
    data.viewed = false;
    data.user_id = event.metadata.user()?;

    Ok(())
}
