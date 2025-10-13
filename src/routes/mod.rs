pub mod assets;
pub mod auth;
pub mod health;
pub mod profile;

pub use assets::AssetsService;
pub use auth::{
    get_login, get_password_reset, get_password_reset_complete, get_register, post_login,
    post_password_reset, post_password_reset_complete, post_register, AppState,
};
pub use health::{health, ready};
pub use profile::{
    get_onboarding, get_onboarding_skip, get_profile, post_onboarding_step_1,
    post_onboarding_step_2, post_onboarding_step_3, post_onboarding_step_4, post_profile,
};
