use imkitchen_identity::login::Login;
use imkitchen_types::recipe::{DietaryRestriction, RecipeType};
use imkitchen_web_shared::services::settings::General as GeneralView;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct General {
    pub email: String,
    pub description: String,
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    /// Optional courses to generate besides the main course.
    pub recipe_types: Vec<RecipeType>,
    /// 0.1 – 1.0
    pub cuisine_variety_weight: f32,
}

impl From<GeneralView> for General {
    fn from(g: GeneralView) -> Self {
        Self {
            email: g.email,
            description: g.description,
            household_size: g.preferences.household_size,
            dietary_restrictions: g.preferences.dietary_restrictions,
            recipe_types: g.preferences.recipe_types,
            cuisine_variety_weight: g.preferences.cuisine_variety_weight,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PreferencesRequest {
    pub household_size: u16,
    #[serde(default)]
    pub dietary_restrictions: Vec<DietaryRestriction>,
    #[serde(default)]
    pub recipe_types: Vec<RecipeType>,
    pub cuisine_variety_weight: f32,
}

#[derive(Deserialize, Debug)]
pub struct ProfileRequest {
    pub description: String,
}

#[derive(Deserialize, Debug)]
pub struct UsernameRequest {
    pub username: String,
}

/// A signed-in device.
#[derive(Serialize, Debug)]
pub struct Session {
    /// The `acc` claim; `DELETE /settings/sessions/{id}` signs it out.
    pub id: String,
    /// The client identity string the device signed in with.
    pub user_agent: String,
    pub tz: String,
    /// The session making this request.
    pub current: bool,
}

impl Session {
    pub fn new(login: &Login, current_acc: &str) -> Self {
        Self {
            id: login.id.to_owned(),
            user_agent: login.user_agent.to_owned(),
            tz: login.tz.to_owned(),
            current: login.id == current_acc,
        }
    }
}
