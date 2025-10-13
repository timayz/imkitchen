pub mod assets;
pub mod auth;
pub mod health;

pub use assets::AssetsService;
pub use auth::{get_register, post_register, AppState};
pub use health::{health, ready};
