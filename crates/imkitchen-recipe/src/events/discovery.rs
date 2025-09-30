use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use imkitchen_shared::DomainEvent;

/// Event triggered when a recipe is viewed by a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeViewedEvent {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Uuid,
    pub viewed_at: DateTime<Utc>,
    pub referrer: Option<String>,
}

impl DomainEvent for RecipeViewedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.recipe_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.viewed_at
    }

    fn event_type(&self) -> &'static str {
        "RecipeViewed"
    }
}

/// Event triggered when a recipe search is performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSearchedEvent {
    pub event_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Uuid,
    pub query_text: String,
    pub search_type: String,
    pub results_count: u32,
    pub searched_at: DateTime<Utc>,
}

impl DomainEvent for RecipeSearchedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.searched_at
    }

    fn event_type(&self) -> &'static str {
        "RecipeSearched"
    }
}

/// Event triggered when discovery filters are applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterAppliedEvent {
    pub event_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Uuid,
    pub rating_threshold: Option<f32>,
    pub difficulty_levels: Vec<String>,
    pub max_prep_time: Option<u32>,
    pub dietary_restrictions: Vec<String>,
    pub meal_types: Vec<String>,
    pub results_count: u32,
    pub applied_at: DateTime<Utc>,
}

impl DomainEvent for FilterAppliedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.applied_at
    }

    fn event_type(&self) -> &'static str {
        "FilterApplied"
    }
}

/// Event triggered when a random recipe is requested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomRecipeRequestedEvent {
    pub event_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Uuid,
    pub filter_criteria: String,
    pub selected_recipe_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
}

impl DomainEvent for RandomRecipeRequestedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.requested_at
    }

    fn event_type(&self) -> &'static str {
        "RandomRecipeRequested"
    }
}

/// Event triggered when a discovery session is started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySessionStartedEvent {
    pub event_id: Uuid,
    pub session_id: Uuid,
    pub user_id: Option<Uuid>,
    pub discovery_type: String,
    pub started_at: DateTime<Utc>,
}

impl DomainEvent for DiscoverySessionStartedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.session_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    fn event_type(&self) -> &'static str {
        "DiscoverySessionStarted"
    }
}

/// Event triggered when trending calculations are completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCalculatedEvent {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub popularity_score: f64,
    pub trending_rank: u32,
    pub time_weighted_score: f64,
    pub calculated_at: DateTime<Utc>,
}

impl DomainEvent for TrendingCalculatedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.recipe_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.calculated_at
    }

    fn event_type(&self) -> &'static str {
        "TrendingCalculated"
    }
}

/// Event triggered when recipe popularity is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipePopularityUpdatedEvent {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub view_count: u32,
    pub rating_average: f32,
    pub rating_count: u32,
    pub popularity_score: f64,
    pub updated_at: DateTime<Utc>,
}

impl DomainEvent for RecipePopularityUpdatedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.recipe_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn event_type(&self) -> &'static str {
        "RecipePopularityUpdated"
    }
}

// Placeholder functions for Evento compatibility when it gets added later
impl RecipeViewedEvent {
    pub fn aggregator_name() -> &'static str {
        "Discovery"
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

impl RecipeSearchedEvent {
    pub fn aggregator_name() -> &'static str {
        "Discovery"
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

impl FilterAppliedEvent {
    pub fn aggregator_name() -> &'static str {
        "Discovery"
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

impl RandomRecipeRequestedEvent {
    pub fn aggregator_name() -> &'static str {
        "Discovery"
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

impl DiscoverySessionStartedEvent {
    pub fn aggregator_name() -> &'static str {
        "DiscoverySession"
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

impl TrendingCalculatedEvent {
    pub fn aggregator_name() -> &'static str {
        "Trending"
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

impl RecipePopularityUpdatedEvent {
    pub fn aggregator_name() -> &'static str {
        "Trending"
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}