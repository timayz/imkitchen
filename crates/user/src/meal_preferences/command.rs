use evento::{
    Executor, Projection, Snapshot,
    metadata::{Event, Metadata},
};
use imkitchen_shared::{
    recipe::DietaryRestriction,
    user::meal_preferences::{Changed, MealPreferences},
};
use validator::Validate;

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

fn create_projection(id: impl Into<String>) -> Projection<CommandData> {
    Projection::new::<MealPreferences>(id)
        .handler(handle_updated())
        .safety_check()
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    id: impl Into<String>,
) -> Result<Command<'a, E>, anyhow::Error> {
    let id = id.into();

    let result = create_projection(&id).execute(executor).await?;

    let cmd = match result {
        Some(data) => Command::new(id, data.get_cursor_version()?, data, executor),
        _ => Command::new(
            id,
            0,
            CommandData {
                household_size: 4,
                dietary_restrictions: vec![],
                cuisine_variety_weight: 1.0,
            },
            executor,
        ),
    };

    Ok(cmd)
}

impl evento::Snapshot for CommandData {}

#[evento::handler]
async fn handle_updated(event: Event<Changed>, data: &mut CommandData) -> anyhow::Result<()> {
    data.household_size = event.data.household_size;
    data.dietary_restrictions = event.data.dietary_restrictions;
    data.cuisine_variety_weight = event.data.cuisine_variety_weight;

    Ok(())
}
