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
    pub user_id: String,
    #[validate(range(min = 1))]
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    #[validate(range(min = 0.1, max = 1.0))]
    pub cuisine_variety_weight: f32,
}

impl<'a, E: Executor + Clone> Command<'a, E> {
    pub async fn update(executor: &E, input: UpdateInput) -> imkitchen_shared::Result<()> {
        input.validate()?;

        evento::aggregator(&input.user_id)
            .event(&Changed {
                dietary_restrictions: input.dietary_restrictions,
                household_size: input.household_size,
                cuisine_variety_weight: input.cuisine_variety_weight,
            })
            .metadata(&Metadata::new(input.user_id))
            .commit(executor)
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
) -> Result<Option<Command<'a, E>>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .load::<MealPreferences>(&id)
        .filter_events_by_name(false)
        .execute(executor)
        .await?
        .map(|loaded| Command::new(id, loaded, executor)))
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
