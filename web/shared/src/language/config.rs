use super::UserLanguageSource;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct UserLanguageConfig {
    pub fallback_language: String,
    pub sources: Vec<Arc<dyn UserLanguageSource>>,
}
