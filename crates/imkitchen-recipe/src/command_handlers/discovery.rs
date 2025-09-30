use crate::commands::discovery::*;
use crate::domain::discovery::*;
use crate::events::discovery::*;
use uuid::Uuid;
use chrono::Utc;
use validator::Validate;
use std::collections::HashMap;

/// Command handler for recipe search operations
pub struct SearchRecipesCommandHandler {
    search_service: RecipeSearchService,
}

impl SearchRecipesCommandHandler {
    pub fn new() -> Self {
        Self {
            search_service: RecipeSearchService::new(),
        }
    }

    pub async fn handle(&self, command: SearchRecipesCommand) -> Result<Vec<Uuid>, SearchError> {
        // Validate command
        command.validate().map_err(|e| SearchError::InvalidQuery(e.to_string()))?;

        // Perform search using the search service
        let results = self.search_service.search_recipes(&command.search_criteria).await?;

        // Emit search event
        let _event = RecipeSearchedEvent {
            event_id: Uuid::new_v4(),
            user_id: command.user_id,
            session_id: command.session_id,
            query_text: command.search_criteria.query_text.clone(),
            search_type: format!("{:?}", command.search_criteria.search_type),
            results_count: results.len() as u32,
            searched_at: Utc::now(),
        };

        // TODO: Emit event through event store when Evento is configured
        tracing::info!("Recipe search completed: {} results", results.len());

        Ok(results)
    }
}

/// Command handler for filter application operations
pub struct ApplyFiltersCommandHandler {
    search_service: RecipeSearchService,
}

impl ApplyFiltersCommandHandler {
    pub fn new() -> Self {
        Self {
            search_service: RecipeSearchService::new(),
        }
    }

    pub async fn handle(&self, command: ApplyFiltersCommand) -> Result<Vec<Uuid>, SearchError> {
        // Validate command
        command.validate().map_err(|e| SearchError::InvalidQuery(e.to_string()))?;

        // Apply filters to current search
        let results = self.search_service.search_recipes(&command.current_search_criteria).await?;

        // Emit filter applied event
        let _event = FilterAppliedEvent {
            event_id: Uuid::new_v4(),
            user_id: command.user_id,
            session_id: command.session_id,
            rating_threshold: command.filters.rating_threshold,
            difficulty_levels: command.filters.difficulty_levels.iter()
                .map(|d| format!("{:?}", d)).collect(),
            max_prep_time: command.filters.max_prep_time,
            dietary_restrictions: command.filters.dietary_restrictions.iter()
                .map(|d| format!("{:?}", d)).collect(),
            meal_types: command.filters.meal_types.iter()
                .map(|m| format!("{:?}", m)).collect(),
            results_count: results.len() as u32,
            applied_at: Utc::now(),
        };

        // TODO: Emit event through event store when Evento is configured
        tracing::info!("Filters applied: {} results", results.len());

        Ok(results)
    }
}

/// Command handler for random recipe requests
pub struct RequestRandomRecipeCommandHandler {
    random_selector: RandomRecipeSelector,
}

impl RequestRandomRecipeCommandHandler {
    pub fn new() -> Self {
        Self {
            random_selector: RandomRecipeSelector::new(),
        }
    }

    pub async fn handle(&self, command: RequestRandomRecipeCommand) -> Result<Option<Uuid>, SelectionError> {
        // Validate command
        command.validate().map_err(|_e| SelectionError::NoRecipesAvailable)?;

        // Select random recipe with preference filtering
        let selected_recipe = self.random_selector
            .select_random_recipe(command.user_id, &command.preference_filters).await?;

        // Emit random recipe requested event
        let filter_criteria = format!(
            "rating:{:?},difficulty:{:?},time:{:?},dietary:{:?},meal:{:?}",
            command.preference_filters.rating_threshold,
            command.preference_filters.difficulty_levels,
            command.preference_filters.max_prep_time,
            command.preference_filters.dietary_restrictions,
            command.preference_filters.meal_types
        );

        let _event = RandomRecipeRequestedEvent {
            event_id: Uuid::new_v4(),
            user_id: command.user_id,
            session_id: command.session_id,
            filter_criteria,
            selected_recipe_id: selected_recipe,
            requested_at: Utc::now(),
        };

        // TODO: Emit event through event store when Evento is configured
        tracing::info!("Random recipe requested: {:?}", selected_recipe);

        Ok(selected_recipe)
    }
}

/// Discovery session aggregate for tracking user interactions and learning preferences
#[derive(Debug, Clone)]
pub struct DiscoverySession {
    pub session_id: Uuid,
    pub user_id: Option<Uuid>,
    pub discovery_type: String,
    pub started_at: chrono::DateTime<Utc>,
    pub search_count: u32,
    pub filter_applications: u32,
    pub recipes_viewed: u32,
    pub viewed_recipe_ids: Vec<Uuid>,
    pub learned_preferences: HashMap<String, String>,
}

impl DiscoverySession {
    /// Start a new discovery session
    pub fn start(session_id: Uuid, user_id: Option<Uuid>, discovery_type: &str) -> Self {
        let session = Self {
            session_id,
            user_id,
            discovery_type: discovery_type.to_string(),
            started_at: Utc::now(),
            search_count: 0,
            filter_applications: 0,
            recipes_viewed: 0,
            viewed_recipe_ids: Vec::new(),
            learned_preferences: HashMap::new(),
        };

        // Emit session started event
        let _event = DiscoverySessionStartedEvent {
            event_id: Uuid::new_v4(),
            session_id,
            user_id,
            discovery_type: discovery_type.to_string(),
            started_at: session.started_at,
        };

        // TODO: Emit event through event store when Evento is configured
        tracing::info!("Discovery session started: {} ({})", session_id, discovery_type);

        session
    }

    /// Record a search performed in this session
    pub fn record_search(&mut self, query: &str, search_type: &str, _results_count: u32) {
        self.search_count += 1;
        
        // Learn preferences from search patterns
        if query.contains("vegetarian") || query.contains("vegan") {
            self.learn_preference("dietary", "plant_based");
        }
        if query.contains("quick") || query.contains("easy") {
            self.learn_preference("difficulty", "Easy");
        }

        tracing::debug!("Search recorded in session {}: {} ({})", self.session_id, query, search_type);
    }

    /// Record filter application in this session
    pub fn record_filter_application(&mut self, results_count: u32) {
        self.filter_applications += 1;
        
        tracing::debug!("Filter application recorded in session {}: {} results", self.session_id, results_count);
    }

    /// Record a recipe view in this session
    pub fn record_recipe_view(&mut self, recipe_id: Uuid) {
        if !self.viewed_recipe_ids.contains(&recipe_id) {
            self.viewed_recipe_ids.push(recipe_id);
            self.recipes_viewed += 1;

            // Emit recipe viewed event
            let _event = RecipeViewedEvent {
                event_id: Uuid::new_v4(),
                recipe_id,
                user_id: self.user_id,
                session_id: self.session_id,
                viewed_at: Utc::now(),
                referrer: Some(self.discovery_type.clone()),
            };

            // TODO: Emit event through event store when Evento is configured
            tracing::debug!("Recipe view recorded in session {}: {}", self.session_id, recipe_id);
        }
    }

    /// Learn a user preference from interactions
    pub fn learn_preference(&mut self, preference_type: &str, preference_value: &str) {
        self.learned_preferences.insert(
            preference_type.to_string(),
            preference_value.to_string()
        );
        
        tracing::debug!("Preference learned in session {}: {} = {}", 
                       self.session_id, preference_type, preference_value);
    }

    /// Get learned preferences for this session
    pub fn get_learned_preferences(&self) -> &HashMap<String, String> {
        &self.learned_preferences
    }

    /// Check if session is active (has recent activity)
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        let inactive_threshold = chrono::Duration::minutes(30);
        now.signed_duration_since(self.started_at) < inactive_threshold
    }
}