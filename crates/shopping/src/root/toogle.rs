use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use imkitchen_shared::shopping::{Checked, Unchecked};

pub struct ToggleInput {
    pub week: u64,
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

        let Some(ingredients) = shopping.ingredients.get(&input.week) else {
            imkitchen_shared::user!("ingredient not found");
        };

        if !ingredients.contains(&input.name) {
            imkitchen_shared::user!("ingredient not found");
        }

        let checked = shopping
            .checked
            .get(&input.week)
            .and_then(|v| v.get(&input.name))
            .is_some();

        if checked {
            shopping
                .aggregator()?
                .event(&Unchecked {
                    week: input.week,
                    ingredient: input.name,
                })
                .metadata(&Metadata::new(request_by))
                .commit(&self.executor)
                .await?;
        } else {
            shopping
                .aggregator()?
                .event(&Checked {
                    week: input.week,
                    ingredient: input.name,
                })
                .metadata(&Metadata::new(request_by))
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
