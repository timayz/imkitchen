use evento::{Executor, ProjectionAggregator};
use imkitchen_shared::shopping::{Checked, Unchecked};

pub struct ToggleInput {
    pub name: String,
}

impl<E: Executor> super::Command<E> {
    pub async fn toggle(
        &self,
        input: ToggleInput,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        let Some(shopping) = self.load(&request_by).await? else {
            imkitchen_shared::not_found!("shopping in toogle");
        };

        if !shopping.ingredients.contains(&input.name) {
            imkitchen_shared::user!("ingredient not found");
        }

        let checked = shopping.checked.contains(&input.name);

        if checked {
            shopping
                .aggregator()?
                .event(&Unchecked {
                    ingredient: input.name,
                })
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
        } else {
            shopping
                .aggregator()?
                .event(&Checked {
                    ingredient: input.name,
                })
                .requested_by(request_by)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
