use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use crate::domain::discovery::{DiscoveryFilters, SearchCriteria, SortingCriteria};

/// Command to search for recipes with criteria and filters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SearchRecipesCommand {
    pub command_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Uuid,
    #[validate(nested)]
    pub search_criteria: SearchCriteria,
    #[validate(nested)]
    pub filters: DiscoveryFilters,
    pub sort_order: SortingCriteria,
    #[validate(range(min = 1))]
    pub page: u32,
    #[validate(range(min = 1, max = 100))]
    pub page_size: u32,
}

/// Command to apply discovery filters to current search
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApplyFiltersCommand {
    pub command_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Uuid,
    #[validate(nested)]
    pub filters: DiscoveryFilters,
    #[validate(nested)]
    pub current_search_criteria: SearchCriteria,
}

/// Command to request a random recipe with preference filtering
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RequestRandomRecipeCommand {
    pub command_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Uuid,
    #[validate(nested)]
    pub preference_filters: DiscoveryFilters,
}