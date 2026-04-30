use evento::{Executor, ProjectionAggregator};
use imkitchen_types::shopping::{Checked, Unchecked};

pub struct ToggleInput {
    pub name: String,
}

impl<E: Executor> super::Module<E> {
    pub async fn toggle(
        &self,
        input: ToggleInput,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let Some(shopping) = self.load(&request_by).await? else {
            crate::not_found!("shopping in toogle");
        };

        if !shopping.ingredients.contains(&input.name) {
            crate::user!("ingredient not found");
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
