mod add_comment;
mod check_comment_like;
mod check_comment_unlike;
mod check_like;
mod check_unlike;
mod uncheck_comment_like;
mod uncheck_comment_unlike;
mod uncheck_like;
mod uncheck_unlike;
mod view;

use std::ops::Deref;

pub use add_comment::*;

use evento::{Executor, Projection, ProjectionAggregator, Snapshot, metadata::Event};
use imkitchen_shared::{
    recipe::{
        Deleted,
        rating::{self, LikeChecked, LikeUnchecked, UnlikeChecked, UnlikeUnchecked, Viewed},
    },
    user::User,
};
use sqlx::prelude::FromRow;

pub struct Command<E: Executor> {
    state: imkitchen_shared::State<E>,
}

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Rating> {
        let id = id.into();
        let user_id = user_id.into();

        create_projection(&id, &user_id)
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| Rating {
                    id,
                    user_id,
                    viewed: false,
                    liked: false,
                    unliked: false,
                    cursor: Default::default(),
                })
            })
    }
}

#[evento::projection(FromRow)]
pub struct Rating {
    pub id: String,
    pub user_id: String,
    pub viewed: bool,
    pub liked: bool,
    pub unliked: bool,
}

pub fn create_projection(id: impl Into<String>, user_id: impl Into<String>) -> Projection<Rating> {
    Projection::new::<rating::Rating>(id)
        .aggregator::<User>(user_id)
        .handler(handle_viewed())
        .handler(handle_like_checked())
        .handler(handle_like_unchecked())
        .handler(handle_unlike_checked())
        .handler(handle_unlike_unchecked())
        .handler(handle_recipe_deleted())
        .safety_check()
}

impl ProjectionAggregator for Rating {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}
impl Snapshot for Rating {}

#[evento::handler]
async fn handle_viewed(event: Event<Viewed>, data: &mut Rating) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.user_id = event.metadata.user()?;
    data.viewed = true;

    Ok(())
}

#[evento::handler]
async fn handle_like_checked(event: Event<LikeChecked>, data: &mut Rating) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.user_id = event.metadata.user()?;
    data.liked = true;
    data.unliked = false;

    Ok(())
}

#[evento::handler]
async fn handle_like_unchecked(
    _event: Event<LikeUnchecked>,
    data: &mut Rating,
) -> anyhow::Result<()> {
    data.liked = false;

    Ok(())
}

#[evento::handler]
async fn handle_unlike_checked(
    event: Event<UnlikeChecked>,
    data: &mut Rating,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.user_id = event.metadata.user()?;
    data.unliked = true;
    data.liked = false;

    Ok(())
}

#[evento::handler]
async fn handle_unlike_unchecked(
    _event: Event<UnlikeUnchecked>,
    data: &mut Rating,
) -> anyhow::Result<()> {
    data.unliked = false;

    Ok(())
}

#[evento::handler]
async fn handle_recipe_deleted(_event: Event<Deleted>, data: &mut Rating) -> anyhow::Result<()> {
    data.unliked = false;
    data.liked = false;
    data.viewed = false;

    Ok(())
}
