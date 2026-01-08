use evento::{
    Executor, Projection, Snapshot,
    metadata::{Event, Metadata},
};
use imkitchen_shared::shopping::{Checked, Generated, Resetted, Shopping, Unchecked};
use std::collections::{HashMap, HashSet};

#[evento::command]
pub struct Command {
    pub user_id: String,
    pub checked: HashMap<u64, HashSet<String>>,
    pub ingredients: HashMap<u64, HashSet<String>>,
}

pub struct ToggleInput {
    pub week: u64,
    pub name: String,
}

impl<'a, E: Executor> Command<'a, E> {
    pub async fn toggle(
        &self,
        input: ToggleInput,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        if request_by != self.user_id {
            imkitchen_shared::forbidden!("not owner");
        }

        let Some(ingredients) = self.ingredients.get(&input.week) else {
            imkitchen_shared::user!("ingredient not found");
        };

        if !ingredients.contains(&input.name) {
            imkitchen_shared::user!("ingredient not found");
        }

        let checked = self
            .checked
            .get(&input.week)
            .and_then(|v| v.get(&input.name))
            .is_some();

        if checked {
            self.aggregator()
                .event(&Unchecked {
                    week: input.week,
                    ingredient: input.name,
                })
                .metadata(&Metadata::new(request_by))
                .commit(self.executor)
                .await?;
        } else {
            self.aggregator()
                .event(&Checked {
                    week: input.week,
                    ingredient: input.name,
                })
                .metadata(&Metadata::new(request_by))
                .commit(self.executor)
                .await?;
        }

        Ok(())
    }
}

impl<'a, E: Executor> Command<'a, E> {
    pub async fn reset(
        &self,
        week: u64,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let request_by = request_by.into();
        if request_by != self.user_id {
            imkitchen_shared::forbidden!("not owner");
        }

        self.aggregator()
            .event(&Resetted { week })
            .metadata(&Metadata::new(request_by))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}

impl Snapshot for CommandData {}

pub fn create_projection(id: impl Into<String>) -> Projection<CommandData> {
    Projection::new::<Shopping>(id)
        .handler(handle_checked())
        .handler(handle_resetted())
        .handler(handle_generated())
        .handler(handle_unchecked())
        .safety_check()
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    id: impl Into<String>,
) -> anyhow::Result<Option<Command<'a, E>>> {
    let id = id.into();
    let Some(data) = create_projection(&id).execute(executor).await? else {
        return Ok(None);
    };

    Ok(Some(Command::new(
        id,
        data.get_cursor_version()?,
        data,
        executor,
    )))
}

#[evento::handler]
async fn handle_generated(event: Event<Generated>, data: &mut CommandData) -> anyhow::Result<()> {
    data.user_id = event.metadata.user()?;

    let ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();

    data.ingredients.insert(event.data.week, ingredients);
    data.checked.remove(&event.data.week);

    if data.ingredients.len() <= 5 {
        return Ok(());
    }

    let mut keys = data.ingredients.keys().cloned().collect::<Vec<_>>();
    keys.sort();

    if let Some(key) = keys.first() {
        data.ingredients.remove(key);
        data.checked.remove(key);
    }

    Ok(())
}

#[evento::handler]
async fn handle_checked(event: Event<Checked>, data: &mut CommandData) -> anyhow::Result<()> {
    let entry = data.checked.entry(event.data.week).or_default();
    entry.insert(event.data.ingredient);

    Ok(())
}

#[evento::handler]
async fn handle_unchecked(event: Event<Unchecked>, data: &mut CommandData) -> anyhow::Result<()> {
    let entry = data.checked.entry(event.data.week).or_default();
    entry.remove(&event.data.ingredient);
    if entry.is_empty() {
        data.checked.remove(&event.data.week);
    }

    Ok(())
}

#[evento::handler]
async fn handle_resetted(event: Event<Resetted>, data: &mut CommandData) -> anyhow::Result<()> {
    data.checked.remove(&event.data.week);

    Ok(())
}
