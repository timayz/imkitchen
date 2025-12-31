use evento::{Executor, metadata::Metadata};
use sqlx::{SqlitePool, prelude::FromRow};
use ulid::Ulid;
use validator::Validate;

use crate::rating::{
    CommentAdded, CommentLikeCheked, CommentLikeUnchecked, CommentUnlikeChecked,
    CommentUnlikeUnchecked, LikeChecked, LikeUnchecked, Rating, UnlikeChecked, UnlikeUnchecked,
    Viewed,
};

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
    user_id: impl Into<String>,
) -> Result<Command<'a, E>, anyhow::Error> {
    let id = id.into();
    let user_id = user_id.into();

    Ok(super::create_projection()
        .no_safety_check()
        .load::<Rating>(&id)
        .aggregator_raw("imkitchen-user/User", &user_id)
        .data(pool.clone())
        .execute_all(executor)
        .await?
        .map(|loaded| Command::new(id.to_owned(), loaded, executor))
        .unwrap_or_else(|| {
            Command::new(
                id,
                evento::LoadResult {
                    item: CommandData {
                        user_id,
                        viewed: false,
                        liked: false,
                        unliked: false,
                    },
                    version: 0,
                    routing_key: None,
                },
                executor,
            )
        }))
}

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
