use evento::{
    Action, Executor, Projection,
    metadata::{Event, Metadata},
};
use imkitchen_recipe::DietaryRestriction;
use validator::Validate;

use super::{Changed, MealPreferences};

#[evento::command]
pub struct Command {
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
}

#[derive(Validate)]
pub struct UpdateInput {
    #[validate(range(min = 1))]
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    #[validate(range(min = 0.1, max = 1.0))]
    pub cuisine_variety_weight: f32,
}

impl<'a, E: Executor + Clone> Command<'a, E> {
    pub async fn update(&self, input: UpdateInput) -> imkitchen_shared::Result<()> {
        input.validate()?;

        self.aggregator()
            .event(&Changed {
                dietary_restrictions: input.dietary_restrictions,
                household_size: input.household_size,
                cuisine_variety_weight: input.cuisine_variety_weight,
            })
            .metadata(&Metadata::new(self.aggregator_id.to_owned()))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}

fn create_projection<E: Executor>() -> Projection<CommandData, E> {
    Projection::new("user-meal-preferences-command").handler(handle_updated())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    id: impl Into<String>,
) -> Result<Command<'a, E>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .no_safety_check()
        .load::<MealPreferences>(&id)
        .execute_all(executor)
        .await?
        .map(|loaded| Command::new(&id, loaded, executor))
        .unwrap_or_else(|| {
            Command::new(
                &id,
                evento::LoadResult {
                    item: CommandData {
                        household_size: 4,
                        dietary_restrictions: vec![],
                        cuisine_variety_weight: 1.0,
                    },
                    version: 0,
                    routing_key: None,
                },
                executor,
            )
        }))
}

impl evento::Snapshot for CommandData {}

#[evento::handler]
async fn handle_updated<E: Executor>(
    event: Event<Changed>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.household_size = event.data.household_size;
            data.dietary_restrictions = event.data.dietary_restrictions;
            data.cuisine_variety_weight = event.data.cuisine_variety_weight;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}
