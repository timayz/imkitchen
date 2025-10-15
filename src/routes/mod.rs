pub mod assets;
pub mod auth;
pub mod health;
pub mod profile;
pub mod recipes;

pub use assets::AssetsService;
pub use auth::{
    get_login, get_password_reset, get_password_reset_complete, get_register, post_login,
    post_logout, post_password_reset, post_password_reset_complete, post_register,
    post_stripe_webhook, AppState,
};
pub use health::{health, ready};
pub use profile::{
    get_onboarding, get_onboarding_skip, get_profile, get_subscription, get_subscription_success,
    post_onboarding_step_1, post_onboarding_step_2, post_onboarding_step_3, post_onboarding_step_4,
    post_profile, post_subscription_upgrade,
};
pub use recipes::{
    get_ingredient_row, get_instruction_row, get_recipe_detail, get_recipe_edit_form,
    get_recipe_form, post_create_recipe, post_delete_recipe, post_update_recipe,
};
