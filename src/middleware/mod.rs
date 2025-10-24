pub mod auth;
pub mod cache;
pub mod minify;

pub use auth::{auth_middleware, Auth};
pub use cache::cache_control_middleware;
pub use minify::minify_html_middleware;
