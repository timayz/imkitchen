use async_trait::async_trait;
use axum::http::request::Parts;
use std::fmt::Debug;

#[async_trait]
pub trait UserLanguageSource: Send + Sync + Debug {
    async fn languages_from_parts(&self, parts: &mut Parts) -> Vec<String>;
}
