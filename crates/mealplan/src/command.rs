use evento::{Executor, LoadResult};
use sqlx::SqlitePool;

use crate::MealPlan;

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<MealPlan>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn load_optional(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<MealPlan>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }
}
