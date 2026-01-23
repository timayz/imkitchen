mod add;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregator, metadata::Event};
use imkitchen_shared::recipe::comment::{self, Added};
use std::ops::Deref;

pub use add::AddCommentInput;

#[derive(Clone)]
pub struct Command<E: Executor>(pub(crate) imkitchen_shared::State<E>);

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Command<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Comment>> {
        let id = id.into();

        create_projection::<E>(&id).execute(&self.executor).await
    }
}

#[evento::projection(Encode, Decode)]
pub struct Comment {
    pub id: String,
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Comment> {
    Projection::new::<comment::Comment>(id)
        .handler(handle_added())
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
