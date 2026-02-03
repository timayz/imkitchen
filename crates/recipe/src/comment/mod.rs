mod add;
mod reply;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregator, metadata::Event};
use imkitchen_shared::recipe::comment::{self, Added, Replied};
use std::ops::Deref;

pub use add::AddCommentInput;
pub use reply::ReplyCommentInput;

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
        recipe_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Option<Comment>> {
        create_projection::<E>(recipe_id, user_id)
            .execute(&self.executor)
            .await
    }

    pub async fn load_from(&self, id: impl Into<String>) -> anyhow::Result<Option<Comment>> {
        Projection::new::<comment::Comment>(id)
            .handler(handle_added())
            .skip::<Replied>()
            .safety_check()
            .execute(&self.executor)
            .await
    }
}

#[evento::projection(Encode, Decode)]
pub struct Comment {
    pub id: String,
}

fn create_projection<E: Executor>(
    recipe_id: impl Into<String>,
    user_id: impl Into<String>,
) -> Projection<E, Comment> {
    Projection::ids::<comment::Comment>(vec![recipe_id.into(), user_id.into()])
        .handler(handle_added())
        .skip::<Replied>()
        .safety_check()
}

impl ProjectionAggregator for Comment {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_added(event: Event<Added>, data: &mut Comment) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();

    Ok(())
}
