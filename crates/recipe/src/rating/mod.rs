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

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregator, metadata::Event};
use imkitchen_shared::recipe::rating::{
    self, LikeChecked, LikeUnchecked, UnlikeChecked, UnlikeUnchecked, Viewed,
};

#[derive(Clone)]
pub struct Command<E: Executor>(pub(crate) imkitchen_shared::State<E>);

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

        create_projection::<E>(&id, &user_id)
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| Rating {
                    id: evento::hash_ids(vec![id, user_id]),
                    viewed: false,
                    liked: false,
                    unliked: false,
                    cursor: Default::default(),
                })
            })
    }
}

#[evento::projection(Encode, Decode)]
pub struct Rating {
    pub id: String,
    pub viewed: bool,
    pub liked: bool,
    pub unliked: bool,
}

pub fn create_projection<E: Executor>(
    id: impl Into<String>,
    user_id: impl Into<String>,
) -> Projection<E, Rating> {
    Projection::ids::<rating::Rating>(vec![id.into(), user_id.into()])
        .handler(handle_viewed())
        .handler(handle_like_checked())
        .handler(handle_like_unchecked())
        .handler(handle_unlike_checked())
        .handler(handle_unlike_unchecked())
        .safety_check()
}

impl ProjectionAggregator for Rating {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_viewed(event: Event<Viewed>, data: &mut Rating) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.viewed = true;

    Ok(())
}

#[evento::handler]
async fn handle_like_checked(event: Event<LikeChecked>, data: &mut Rating) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
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
