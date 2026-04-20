pub mod cache;
#[cfg(not(debug_assertions))]
pub mod minify;

pub use cache::cache_control_middleware;
#[cfg(not(debug_assertions))]
pub use minify::minify_html_middleware;
