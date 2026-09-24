use evento::Executor;
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
        let last_event = evento::read::<MealPlan>(&input.user_id)
            .backward()
            .limit(1)
            .execute(&self.executor)
            .await?;

        let Some(version) = last_event.first().map(|e| e.version) else {
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
