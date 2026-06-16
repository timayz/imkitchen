use evento::Executor;
use evento::cursor::Args;
use evento::{Aggregate, EventFilter};
use imkitchen_types::mealplan::{DaySlotStatus, MealPlan, SlotRecipeStatusChanged};

pub struct ChangeSlotRecipeStatus {
    pub user_id: String,
    pub date: u64,
    pub recipe_id: String,
    pub status: DaySlotStatus,
}

impl<E: Executor> super::Module<E> {
    pub async fn change_slot_recipe_status(
        &self,
        input: ChangeSlotRecipeStatus,
    ) -> crate::Result<()> {
        let last_event = self
            .executor
            .read(
                Some(vec![EventFilter::by_id(
                    MealPlan::aggregate_type(),
                    &input.user_id,
                )]),
                None,
                Args::backward(1, None),
            )
            .await?;

        let Some(version) = last_event.edges.first().map(|e| e.node.version) else {
            crate::not_found!("mealplan not found");
        };

        evento::append(&input.user_id)
            .event(&SlotRecipeStatusChanged {
                date: input.date,
                recipe_id: input.recipe_id,
                status: input.status,
            })
            .original_version(version)
            .requested_by(&input.user_id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
